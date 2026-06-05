use docnav_protocol::{
    ensure_supported_protocol, extract_request_context_from_value, validate_manifest_value,
    validate_probe_result_value, validate_protocol_request_value, validate_protocol_response_value,
    FailureResponse, FindArguments, FindResult, InfoArguments, InfoResult, Manifest, Operation,
    OperationArguments, OperationResult, OutlineArguments, OutlineResult, ProbeResult,
    ProtocolRange, ProtocolResponse, ReadArguments, ReadResult, RequestEnvelope, StableError,
    StableErrorCode, UNKNOWN_REQUEST_ID,
};
use serde::Serialize;
use serde_json::Value;
use std::io::{Read, Write};

pub type AdapterResult<T> = Result<T, AdapterError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterExitCode {
    Success = 0,
    ProtocolError = 2,
    HandlerError = 3,
    IoError = 4,
}

impl AdapterExitCode {
    pub const fn code(self) -> i32 {
        self as i32
    }
}

pub fn exit_code_for_error(code: StableErrorCode) -> AdapterExitCode {
    match code {
        StableErrorCode::InvalidRequest
        | StableErrorCode::ProtocolIncompatible
        | StableErrorCode::CapabilityUnsupported => AdapterExitCode::ProtocolError,
        StableErrorCode::InternalError
        | StableErrorCode::AdapterUnavailable
        | StableErrorCode::AdapterInvokeFailed => AdapterExitCode::IoError,
        StableErrorCode::DocumentNotFound
        | StableErrorCode::DocumentPathInvalid
        | StableErrorCode::DocumentEncodingUnsupported
        | StableErrorCode::FormatUnknown
        | StableErrorCode::FormatAmbiguous
        | StableErrorCode::RefNotFound
        | StableErrorCode::RefAmbiguous => AdapterExitCode::HandlerError,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterError {
    pub error: StableError,
    pub exit_code: AdapterExitCode,
}

impl AdapterError {
    pub fn new(error: StableError) -> Self {
        let exit_code = exit_code_for_error(error.code);
        Self { error, exit_code }
    }

    pub fn with_exit_code(error: StableError, exit_code: AdapterExitCode) -> Self {
        Self { error, exit_code }
    }
}

impl From<StableError> for AdapterError {
    fn from(error: StableError) -> Self {
        Self::new(error)
    }
}

pub trait Adapter {
    fn adapter_id(&self) -> &str;

    fn protocol_range(&self) -> ProtocolRange {
        ProtocolRange::v0_1()
    }

    fn manifest(&self) -> Manifest;

    fn probe(&self, path: &str) -> ProbeResult;

    fn outline(
        &self,
        _request: &RequestEnvelope,
        _arguments: &OutlineArguments,
    ) -> AdapterResult<OutlineResult> {
        Err(self.unsupported(Operation::Outline))
    }

    fn read(
        &self,
        _request: &RequestEnvelope,
        _arguments: &ReadArguments,
    ) -> AdapterResult<ReadResult> {
        Err(self.unsupported(Operation::Read))
    }

    fn find(
        &self,
        _request: &RequestEnvelope,
        _arguments: &FindArguments,
    ) -> AdapterResult<FindResult> {
        Err(self.unsupported(Operation::Find))
    }

    fn info(
        &self,
        _request: &RequestEnvelope,
        _arguments: &InfoArguments,
    ) -> AdapterResult<InfoResult> {
        Err(self.unsupported(Operation::Info))
    }

    fn unsupported(&self, operation: Operation) -> AdapterError {
        StableError::capability_unsupported(operation, self.adapter_id()).into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SdkCommand {
    Manifest,
    Probe { path: String },
    Invoke,
}

pub fn run_command<A, R, W, E>(
    adapter: &A,
    command: SdkCommand,
    stdin: R,
    stdout: W,
    stderr: E,
) -> i32
where
    A: Adapter,
    R: Read,
    W: Write,
    E: Write,
{
    match command {
        SdkCommand::Manifest => write_manifest_json(adapter.manifest(), stdout, stderr),
        SdkCommand::Probe { path } => write_probe_json(adapter.probe(&path), stdout, stderr),
        SdkCommand::Invoke => invoke_once(adapter, stdin, stdout, stderr),
    }
}

pub fn invoke_once<A, R, W, E>(adapter: &A, mut stdin: R, mut stdout: W, mut stderr: E) -> i32
where
    A: Adapter,
    R: Read,
    W: Write,
    E: Write,
{
    let supported = adapter.protocol_range();
    let mut input = String::new();
    if let Err(error) = stdin.read_to_string(&mut input) {
        let response = ProtocolResponse::Failure(FailureResponse::unparsed(
            StableError::invalid_request("request", error.to_string()),
            &supported,
        ));
        let _ = emit_diagnostic(&mut stderr, &format!("failed to read request: {error}"));
        return write_protocol_response(
            &response,
            &mut stdout,
            &mut stderr,
            AdapterExitCode::IoError,
        );
    }

    let request_value: Value = match serde_json::from_str(&input) {
        Ok(value) => value,
        Err(error) => {
            let response = ProtocolResponse::failure(
                supported.max.to_string(),
                UNKNOWN_REQUEST_ID,
                None,
                StableError::invalid_request("request", error.to_string()),
            );
            let _ = emit_diagnostic(&mut stderr, &format!("invalid request JSON: {error}"));
            return write_protocol_response(
                &response,
                &mut stdout,
                &mut stderr,
                AdapterExitCode::ProtocolError,
            );
        }
    };

    let context = extract_request_context_from_value(&request_value);
    let request_id = context
        .request_id
        .clone()
        .unwrap_or_else(|| UNKNOWN_REQUEST_ID.to_owned());

    if let Some(requested_protocol) = context.protocol_version.as_deref() {
        if let Err(error) = ensure_supported_protocol(requested_protocol, &supported) {
            let response = ProtocolResponse::failure(
                supported.max.to_string(),
                request_id,
                context.operation,
                error,
            );
            return write_protocol_response(
                &response,
                &mut stdout,
                &mut stderr,
                AdapterExitCode::ProtocolError,
            );
        }
    }

    if let Err(error) = validate_protocol_request_value(&request_value) {
        let response = ProtocolResponse::failure(
            context
                .protocol_version
                .clone()
                .unwrap_or_else(|| supported.max.to_string()),
            request_id,
            context.operation,
            StableError::invalid_request("request", error.to_string()),
        );
        let _ = emit_diagnostic(
            &mut stderr,
            &format!("request schema validation failed: {error}"),
        );
        return write_protocol_response(
            &response,
            &mut stdout,
            &mut stderr,
            AdapterExitCode::ProtocolError,
        );
    }

    let request: RequestEnvelope = match serde_json::from_value(request_value) {
        Ok(request) => request,
        Err(error) => {
            let response = ProtocolResponse::failure(
                supported.max.to_string(),
                request_id,
                context.operation,
                StableError::invalid_request("request", error.to_string()),
            );
            let _ = emit_diagnostic(
                &mut stderr,
                &format!("request deserialization failed after schema validation: {error}"),
            );
            return write_protocol_response(
                &response,
                &mut stdout,
                &mut stderr,
                AdapterExitCode::ProtocolError,
            );
        }
    };

    if let Err(error) = request.operation_arguments() {
        let response = ProtocolResponse::failure_for_request(&request, error);
        return write_protocol_response(
            &response,
            &mut stdout,
            &mut stderr,
            AdapterExitCode::ProtocolError,
        );
    }

    match dispatch_operation(adapter, &request) {
        Ok(result) => {
            let response = ProtocolResponse::success(
                request.protocol_version.clone(),
                request.request_id.clone(),
                result,
            );
            write_protocol_response(
                &response,
                &mut stdout,
                &mut stderr,
                AdapterExitCode::Success,
            )
        }
        Err(error) => {
            let response = ProtocolResponse::failure_for_request(&request, error.error);
            write_protocol_response(&response, &mut stdout, &mut stderr, error.exit_code)
        }
    }
}

fn dispatch_operation<A: Adapter>(
    adapter: &A,
    request: &RequestEnvelope,
) -> AdapterResult<OperationResult> {
    match (&request.operation, &request.arguments) {
        (Operation::Outline, OperationArguments::Outline(arguments)) => adapter
            .outline(request, arguments)
            .map(OperationResult::Outline),
        (Operation::Read, OperationArguments::Read(arguments)) => {
            adapter.read(request, arguments).map(OperationResult::Read)
        }
        (Operation::Find, OperationArguments::Find(arguments)) => {
            adapter.find(request, arguments).map(OperationResult::Find)
        }
        (Operation::Info, OperationArguments::Info(arguments)) => {
            adapter.info(request, arguments).map(OperationResult::Info)
        }
        _ => Err(StableError::invalid_request(
            "arguments",
            format!("arguments do not match operation {}", request.operation),
        )
        .into()),
    }
}

fn write_manifest_json<W, E>(manifest: Manifest, stdout: W, mut stderr: E) -> i32
where
    W: Write,
    E: Write,
{
    let value = match to_json_value(&manifest, &mut stderr, "manifest") {
        Ok(value) => value,
        Err(exit_code) => return exit_code.code(),
    };
    if let Err(error) = validate_manifest_value(&value) {
        let _ = emit_diagnostic(
            &mut stderr,
            &format!("manifest schema validation failed: {error}"),
        );
        return AdapterExitCode::ProtocolError.code();
    }
    if let Err(error) = manifest.validate_semantics() {
        let _ = emit_diagnostic(
            &mut stderr,
            &format!("manifest semantic validation failed: {error}"),
        );
        return AdapterExitCode::ProtocolError.code();
    }
    write_json_value(&value, stdout, stderr)
}

fn write_probe_json<W, E>(probe: ProbeResult, stdout: W, mut stderr: E) -> i32
where
    W: Write,
    E: Write,
{
    let value = match to_json_value(&probe, &mut stderr, "probe result") {
        Ok(value) => value,
        Err(exit_code) => return exit_code.code(),
    };
    if let Err(error) = validate_probe_result_value(&value) {
        let _ = emit_diagnostic(
            &mut stderr,
            &format!("probe result schema validation failed: {error}"),
        );
        return AdapterExitCode::ProtocolError.code();
    }
    if let Err(error) = probe.validate_semantics() {
        let _ = emit_diagnostic(
            &mut stderr,
            &format!("probe result semantic validation failed: {error}"),
        );
        return AdapterExitCode::ProtocolError.code();
    }
    write_json_value(&value, stdout, stderr)
}

fn to_json_value<T, E>(value: &T, stderr: &mut E, label: &str) -> Result<Value, AdapterExitCode>
where
    T: Serialize,
    E: Write,
{
    match serde_json::to_value(value) {
        Ok(value) => Ok(value),
        Err(error) => {
            let _ = emit_diagnostic(
                &mut *stderr,
                &format!("failed to serialize {label}: {error}"),
            );
            Err(AdapterExitCode::IoError)
        }
    }
}

fn write_json_value<W, E>(value: &Value, mut stdout: W, mut stderr: E) -> i32
where
    W: Write,
    E: Write,
{
    match serde_json::to_writer(&mut stdout, value) {
        Ok(()) => AdapterExitCode::Success.code(),
        Err(error) => {
            let _ = emit_diagnostic(&mut stderr, &format!("failed to write JSON: {error}"));
            AdapterExitCode::IoError.code()
        }
    }
}

fn write_protocol_response<W, E>(
    response: &ProtocolResponse,
    stdout: &mut W,
    stderr: &mut E,
    exit_code: AdapterExitCode,
) -> i32
where
    W: Write,
    E: Write,
{
    if let Err(error) = response.validate() {
        let _ = emit_diagnostic(
            stderr,
            &format!("protocol response semantic validation failed: {error}"),
        );
        return AdapterExitCode::ProtocolError.code();
    }
    let value = match to_json_value(response, stderr, "protocol response") {
        Ok(value) => value,
        Err(exit_code) => return exit_code.code(),
    };
    if let Err(error) = validate_protocol_response_value(&value) {
        let _ = emit_diagnostic(
            stderr,
            &format!("protocol response schema validation failed: {error}"),
        );
        return AdapterExitCode::ProtocolError.code();
    }
    if let Err(error) = serde_json::to_writer(stdout, &value) {
        let _ = emit_diagnostic(
            stderr,
            &format!("failed to write protocol response: {error}"),
        );
        return AdapterExitCode::IoError.code();
    }
    exit_code.code()
}

pub fn emit_diagnostic<W: Write>(stderr: &mut W, message: &str) -> std::io::Result<()> {
    writeln!(stderr, "{message}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use docnav_protocol::{
        positive, AdapterIdentity, Entry, FormatDescriptor, InfoResult, PagedOperation,
        ProbeReason, ProbeReasonCode, RecommendedParameters, PROBE_VERSION, PROTOCOL_VERSION,
    };
    use std::collections::BTreeMap;

    struct StubAdapter;

    impl Adapter for StubAdapter {
        fn adapter_id(&self) -> &str {
            "stub"
        }

        fn manifest(&self) -> Manifest {
            let mut recommended_parameters = BTreeMap::new();
            recommended_parameters.insert(
                PagedOperation::Outline,
                RecommendedParameters {
                    limit_chars: positive(80),
                    options: None,
                },
            );

            Manifest {
                manifest_version: docnav_protocol::MANIFEST_VERSION.to_owned(),
                adapter: AdapterIdentity {
                    id: "stub".to_owned(),
                    name: "Stub Adapter".to_owned(),
                    version: "0.1.0".to_owned(),
                },
                protocol: ProtocolRange::v0_1(),
                formats: vec![FormatDescriptor {
                    id: "stub".to_owned(),
                    extensions: vec![".stub".to_owned()],
                    content_types: vec!["text/stub".to_owned()],
                }],
                capabilities: vec![Operation::Outline, Operation::Info],
                recommended_parameters,
            }
        }

        fn probe(&self, path: &str) -> ProbeResult {
            ProbeResult {
                probe_version: PROBE_VERSION.to_owned(),
                adapter_id: "stub".to_owned(),
                path: path.to_owned(),
                supported: true,
                format: Some("stub".to_owned()),
                confidence: 1.0,
                reasons: vec![ProbeReason {
                    code: ProbeReasonCode::ExtensionMatch,
                    detail: "stub extension".to_owned(),
                }],
            }
        }

        fn outline(
            &self,
            _request: &RequestEnvelope,
            _arguments: &OutlineArguments,
        ) -> AdapterResult<OutlineResult> {
            Ok(OutlineResult {
                entries: vec![Entry {
                    ref_id: "L1:Stub".to_owned(),
                    display: "1 line | 0.1 KB".to_owned(),
                }],
                page: None,
            })
        }

        fn info(
            &self,
            _request: &RequestEnvelope,
            _arguments: &InfoArguments,
        ) -> AdapterResult<InfoResult> {
            Ok(InfoResult {
                display: "Stub".to_owned(),
                capabilities: vec![Operation::Outline, Operation::Info],
            })
        }
    }

    struct InvalidManifestAdapter;

    impl Adapter for InvalidManifestAdapter {
        fn adapter_id(&self) -> &str {
            "bad-manifest"
        }

        fn manifest(&self) -> Manifest {
            let mut manifest = StubAdapter.manifest();
            manifest.adapter.id.clear();
            manifest
        }

        fn probe(&self, path: &str) -> ProbeResult {
            StubAdapter.probe(path)
        }
    }

    struct EmptyReasonsProbeAdapter;

    impl Adapter for EmptyReasonsProbeAdapter {
        fn adapter_id(&self) -> &str {
            "bad-probe"
        }

        fn manifest(&self) -> Manifest {
            StubAdapter.manifest()
        }

        fn probe(&self, path: &str) -> ProbeResult {
            let mut probe = StubAdapter.probe(path);
            probe.reasons.clear();
            probe
        }
    }

    struct BadConfidenceProbeAdapter;

    impl Adapter for BadConfidenceProbeAdapter {
        fn adapter_id(&self) -> &str {
            "bad-probe"
        }

        fn manifest(&self) -> Manifest {
            StubAdapter.manifest()
        }

        fn probe(&self, path: &str) -> ProbeResult {
            let mut probe = StubAdapter.probe(path);
            probe.confidence = 1.5;
            probe
        }
    }

    struct MissingDetailsErrorAdapter;

    impl Adapter for MissingDetailsErrorAdapter {
        fn adapter_id(&self) -> &str {
            "missing-details"
        }

        fn manifest(&self) -> Manifest {
            StubAdapter.manifest()
        }

        fn probe(&self, path: &str) -> ProbeResult {
            StubAdapter.probe(path)
        }

        fn read(
            &self,
            _request: &RequestEnvelope,
            _arguments: &ReadArguments,
        ) -> AdapterResult<ReadResult> {
            Err(AdapterError::new(StableError::new(
                StableErrorCode::RefNotFound,
                "Missing required details.",
                BTreeMap::new(),
            )))
        }
    }

    #[test]
    fn invoke_reads_one_request_and_writes_one_protocol_response() {
        let input = br#"{
          "protocol_version": "0.1",
          "request_id": "req-1",
          "operation": "outline",
          "document": { "path": "sample.stub" },
          "arguments": { "limit_chars": 80, "page": 1 }
        }"#;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let exit = invoke_once(&StubAdapter, &input[..], &mut stdout, &mut stderr);

        assert_eq!(exit, AdapterExitCode::Success.code());
        assert!(stderr.is_empty());
        let response: ProtocolResponse =
            serde_json::from_slice(&stdout).expect("stdout is one JSON response");
        response.validate().expect("response validates");
        let value: serde_json::Value = serde_json::from_slice(&stdout).expect("response JSON");
        assert_eq!(value["operation"], "outline");
        assert_eq!(value["ok"], true);
        assert_eq!(value["result"]["entries"][0]["ref"], "L1:Stub");
    }

    #[test]
    fn invalid_request_outputs_structured_failure_on_stdout() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let exit = invoke_once(
            &StubAdapter,
            b"{not-json}" as &[u8],
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(exit, AdapterExitCode::ProtocolError.code());
        assert!(!stderr.is_empty());
        let response: ProtocolResponse =
            serde_json::from_slice(&stdout).expect("stdout is one JSON response");
        match response {
            ProtocolResponse::Failure(response) => {
                assert_eq!(response.protocol_version, PROTOCOL_VERSION);
                assert_eq!(response.request_id, UNKNOWN_REQUEST_ID);
                assert_eq!(response.operation, None);
                assert_eq!(response.error.code, StableErrorCode::InvalidRequest);
            }
            ProtocolResponse::Success(_) => panic!("expected failure response"),
        }
    }

    #[test]
    fn unsupported_protocol_is_protocol_incompatible_before_schema_const() {
        let input = br#"{
          "protocol_version": "1.0",
          "request_id": "req-1",
          "operation": "outline",
          "document": { "path": "sample.stub" },
          "arguments": { "limit_chars": 80, "page": 1 }
        }"#;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let exit = invoke_once(&StubAdapter, &input[..], &mut stdout, &mut stderr);

        assert_eq!(exit, AdapterExitCode::ProtocolError.code());
        assert!(stderr.is_empty());
        let response: ProtocolResponse =
            serde_json::from_slice(&stdout).expect("stdout is one JSON response");
        match response {
            ProtocolResponse::Failure(response) => {
                assert_eq!(response.error.code, StableErrorCode::ProtocolIncompatible);
                assert_eq!(response.operation, Some(Operation::Outline));
                assert_eq!(response.request_id, "req-1");
            }
            ProtocolResponse::Success(_) => panic!("expected failure response"),
        }
    }

    #[test]
    fn request_schema_rejections_are_structured_invalid_request_failures() {
        let input = br#"{
          "protocol_version": "0.1",
          "request_id": "",
          "operation": "read",
          "document": { "path": "sample.stub" },
          "arguments": { "ref": "", "limit_chars": 80, "page": 1 }
        }"#;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let exit = invoke_once(&StubAdapter, &input[..], &mut stdout, &mut stderr);

        assert_eq!(exit, AdapterExitCode::ProtocolError.code());
        assert!(!stderr.is_empty());
        let response: ProtocolResponse =
            serde_json::from_slice(&stdout).expect("stdout is one JSON response");
        match response {
            ProtocolResponse::Failure(response) => {
                assert_eq!(response.request_id, UNKNOWN_REQUEST_ID);
                assert_eq!(response.operation, Some(Operation::Read));
                assert_eq!(response.error.code, StableErrorCode::InvalidRequest);
            }
            ProtocolResponse::Success(_) => panic!("expected failure response"),
        }
    }

    #[test]
    fn manifest_and_probe_are_not_wrapped_in_invoke_envelope() {
        let mut manifest_stdout = Vec::new();
        let exit = run_command(
            &StubAdapter,
            SdkCommand::Manifest,
            std::io::empty(),
            &mut manifest_stdout,
            std::io::sink(),
        );
        assert_eq!(exit, AdapterExitCode::Success.code());
        let manifest: serde_json::Value =
            serde_json::from_slice(&manifest_stdout).expect("manifest JSON");
        assert!(manifest.get("manifest_version").is_some());
        assert!(manifest.get("protocol_version").is_none());
        assert!(manifest.get("ok").is_none());

        let mut probe_stdout = Vec::new();
        let exit = run_command(
            &StubAdapter,
            SdkCommand::Probe {
                path: "sample.stub".to_owned(),
            },
            std::io::empty(),
            &mut probe_stdout,
            std::io::sink(),
        );
        assert_eq!(exit, AdapterExitCode::Success.code());
        let probe: serde_json::Value = serde_json::from_slice(&probe_stdout).expect("probe JSON");
        assert!(probe.get("probe_version").is_some());
        assert!(probe.get("protocol_version").is_none());
        assert!(probe.get("ok").is_none());
    }

    #[test]
    fn invalid_manifest_is_not_written_to_stdout() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let exit = run_command(
            &InvalidManifestAdapter,
            SdkCommand::Manifest,
            std::io::empty(),
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(exit, AdapterExitCode::ProtocolError.code());
        assert!(stdout.is_empty());
        assert!(!stderr.is_empty());
    }

    #[test]
    fn invalid_probe_is_not_written_to_stdout() {
        fn assert_invalid_probe_not_written(adapter: &impl Adapter) {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();

            let exit = run_command(
                adapter,
                SdkCommand::Probe {
                    path: "sample.stub".to_owned(),
                },
                std::io::empty(),
                &mut stdout,
                &mut stderr,
            );

            assert_eq!(exit, AdapterExitCode::ProtocolError.code());
            assert!(stdout.is_empty());
            assert!(!stderr.is_empty());
        }

        assert_invalid_probe_not_written(&EmptyReasonsProbeAdapter);
        assert_invalid_probe_not_written(&BadConfidenceProbeAdapter);
    }

    #[test]
    fn handler_error_missing_required_details_is_not_written_to_stdout() {
        let input = br#"{
          "protocol_version": "0.1",
          "request_id": "req-1",
          "operation": "read",
          "document": { "path": "sample.stub" },
          "arguments": { "ref": "L1:Stub", "limit_chars": 80, "page": 1 }
        }"#;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let exit = invoke_once(
            &MissingDetailsErrorAdapter,
            &input[..],
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(exit, AdapterExitCode::ProtocolError.code());
        assert!(stdout.is_empty());
        assert!(!stderr.is_empty());
    }
}
