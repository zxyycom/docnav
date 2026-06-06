use super::*;
use docnav_protocol::{
    try_positive, AdapterIdentity, Document, Entry, FormatDescriptor, InfoArguments, InfoResult,
    Manifest, Operation, OperationArguments, OperationResult, OutlineArguments, OutlineResult,
    PagedOperation, ProbeReason, ProbeReasonCode, ProbeResult, ProtocolRange, ProtocolResponse,
    ProtocolVersion, ReadArguments, ReadResult, RecommendedParameters, RequestEnvelope,
    StableError, StableErrorCode, PROBE_VERSION, PROTOCOL_VERSION, UNKNOWN_REQUEST_ID,
};
use std::collections::BTreeMap;

fn positive(value: u32) -> docnav_protocol::PositiveInteger {
    try_positive(value).expect("test positive integer")
}

#[test]
fn internal_error_maps_to_internal_exit_code() {
    assert_eq!(AdapterExitCode::InternalError.code(), 1);
    assert_eq!(
        exit_code_for_error(StableErrorCode::InternalError),
        AdapterExitCode::InternalError
    );
    assert_eq!(
        AdapterError::new(StableError::internal_error("test")).exit_code(),
        AdapterExitCode::InternalError
    );
}

#[test]
fn stable_error_codes_map_to_adapter_exit_codes() {
    let cases = [
        (
            StableErrorCode::InvalidRequest,
            AdapterExitCode::ProtocolError,
        ),
        (
            StableErrorCode::ProtocolIncompatible,
            AdapterExitCode::ProtocolError,
        ),
        (
            StableErrorCode::DocumentNotFound,
            AdapterExitCode::HandlerError,
        ),
        (
            StableErrorCode::DocumentPathInvalid,
            AdapterExitCode::HandlerError,
        ),
        (
            StableErrorCode::DocumentEncodingUnsupported,
            AdapterExitCode::HandlerError,
        ),
        (
            StableErrorCode::FormatUnknown,
            AdapterExitCode::HandlerError,
        ),
        (
            StableErrorCode::FormatAmbiguous,
            AdapterExitCode::HandlerError,
        ),
        (
            StableErrorCode::CapabilityUnsupported,
            AdapterExitCode::ProtocolError,
        ),
        (StableErrorCode::RefNotFound, AdapterExitCode::HandlerError),
        (StableErrorCode::RefAmbiguous, AdapterExitCode::HandlerError),
        (
            StableErrorCode::AdapterUnavailable,
            AdapterExitCode::IoError,
        ),
        (
            StableErrorCode::AdapterInvokeFailed,
            AdapterExitCode::IoError,
        ),
        (
            StableErrorCode::InternalError,
            AdapterExitCode::InternalError,
        ),
    ];

    for (code, expected) in cases {
        assert_eq!(exit_code_for_error(code), expected, "{code:?}");
    }
}

#[test]
fn adapter_error_rejects_success_exit_code() {
    let error = AdapterError::with_exit_code(
        StableError::ref_not_found("missing"),
        AdapterExitCode::Success,
    )
    .expect_err("failure cannot use success exit code");

    assert_eq!(error.exit_code(), AdapterExitCode::Success);
}

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

struct ManifestAdapterIdDriftAdapter;

impl Adapter for ManifestAdapterIdDriftAdapter {
    fn adapter_id(&self) -> &str {
        "stub"
    }

    fn manifest(&self) -> Manifest {
        let mut manifest = StubAdapter.manifest();
        manifest.adapter.id = "drift".to_owned();
        manifest
    }

    fn probe(&self, path: &str) -> ProbeResult {
        StubAdapter.probe(path)
    }
}

struct ManifestSemanticErrorAdapter;

impl Adapter for ManifestSemanticErrorAdapter {
    fn adapter_id(&self) -> &str {
        "stub"
    }

    fn manifest(&self) -> Manifest {
        let mut manifest = StubAdapter.manifest();
        manifest.capabilities = vec![Operation::Info];
        manifest
    }

    fn probe(&self, path: &str) -> ProbeResult {
        StubAdapter.probe(path)
    }
}

struct ManifestProtocolRangeAdapter;

impl Adapter for ManifestProtocolRangeAdapter {
    fn adapter_id(&self) -> &str {
        "stub"
    }

    fn manifest(&self) -> Manifest {
        let mut manifest = StubAdapter.manifest();
        manifest.protocol =
            ProtocolRange::new(ProtocolVersion::new(0, 0), ProtocolVersion::new(0, 1))
                .expect("test protocol range");
        manifest
    }

    fn probe(&self, path: &str) -> ProbeResult {
        StubAdapter.probe(path)
    }
}

struct EmptyReasonsProbeAdapter;

impl Adapter for EmptyReasonsProbeAdapter {
    fn adapter_id(&self) -> &str {
        "stub"
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
        "stub"
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

struct ProbeAdapterIdDriftAdapter;

impl Adapter for ProbeAdapterIdDriftAdapter {
    fn adapter_id(&self) -> &str {
        "stub"
    }

    fn manifest(&self) -> Manifest {
        StubAdapter.manifest()
    }

    fn probe(&self, path: &str) -> ProbeResult {
        let mut probe = StubAdapter.probe(path);
        probe.adapter_id = "drift".to_owned();
        probe
    }
}

struct MissingDetailsErrorAdapter;

impl Adapter for MissingDetailsErrorAdapter {
    fn adapter_id(&self) -> &str {
        "stub"
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
fn execute_operation_dispatches_typed_request() {
    let request = RequestEnvelope {
        protocol_version: PROTOCOL_VERSION.to_owned(),
        request_id: "req-1".to_owned(),
        operation: Operation::Outline,
        document: Document {
            path: "sample.stub".to_owned(),
        },
        arguments: OperationArguments::Outline(OutlineArguments {
            limit_chars: positive(80),
            page: positive(1),
            options: None,
        }),
    };

    let result = execute_operation(&StubAdapter, &request).expect("execute outline");

    match result {
        OperationResult::Outline(result) => {
            assert_eq!(result.entries[0].ref_id, "L1:Stub");
            assert_eq!(result.page, None);
        }
        other => panic!("expected outline result, got {other:?}"),
    }
}

#[test]
fn execute_operation_rejects_mismatched_operation_arguments() {
    let request = RequestEnvelope {
        protocol_version: PROTOCOL_VERSION.to_owned(),
        request_id: "req-1".to_owned(),
        operation: Operation::Read,
        document: Document {
            path: "sample.stub".to_owned(),
        },
        arguments: OperationArguments::Outline(OutlineArguments {
            limit_chars: positive(80),
            page: positive(1),
            options: None,
        }),
    };

    let error = execute_operation(&StubAdapter, &request).expect_err("mismatch fails");

    assert_eq!(error.error().code, StableErrorCode::InvalidRequest);
    assert_eq!(error.error().details["field"], "arguments");
    assert_eq!(
        error.error().details["reason"],
        "arguments do not match operation read"
    );
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
fn invoke_uses_manifest_protocol_range_for_request_version_check() {
    let input = br#"{
          "protocol_version": "1.0",
          "request_id": "req-1",
          "operation": "outline",
          "document": { "path": "sample.stub" },
          "arguments": { "limit_chars": 80, "page": 1 }
        }"#;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = invoke_once(
        &ManifestProtocolRangeAdapter,
        &input[..],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, AdapterExitCode::ProtocolError.code());
    assert!(stderr.is_empty());
    let response: ProtocolResponse =
        serde_json::from_slice(&stdout).expect("stdout is one JSON response");
    match response {
        ProtocolResponse::Failure(response) => {
            assert_eq!(response.error.code, StableErrorCode::ProtocolIncompatible);
            assert_eq!(response.error.details["supported_min"], "0.0");
            assert_eq!(response.error.details["supported_max"], "0.1");
        }
        ProtocolResponse::Success(_) => panic!("expected failure response"),
    }
}

#[test]
fn invoke_rejects_invalid_manifest_without_protocol_envelope() {
    let input = br#"{
          "protocol_version": "0.1",
          "request_id": "req-1",
          "operation": "outline",
          "document": { "path": "sample.stub" },
          "arguments": { "limit_chars": 80, "page": 1 }
        }"#;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = invoke_once(
        &ManifestSemanticErrorAdapter,
        &input[..],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, AdapterExitCode::ProtocolError.code());
    assert!(stdout.is_empty());
    let stderr = String::from_utf8(stderr).expect("stderr is UTF-8");
    assert!(stderr.contains("manifest semantic validation failed"));
    assert!(stderr.contains("recommended_parameters.outline"));
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
fn manifest_adapter_id_drift_is_not_written_to_stdout() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_command(
        &ManifestAdapterIdDriftAdapter,
        SdkCommand::Manifest,
        std::io::empty(),
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, AdapterExitCode::ProtocolError.code());
    assert!(stdout.is_empty());
    let stderr = String::from_utf8(stderr).expect("stderr is UTF-8");
    assert!(stderr.contains("manifest adapter id mismatch"));
    assert!(stderr.contains("\"stub\""));
    assert!(stderr.contains("\"drift\""));
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
fn probe_adapter_id_drift_is_not_written_to_stdout() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_command(
        &ProbeAdapterIdDriftAdapter,
        SdkCommand::Probe {
            path: "sample.stub".to_owned(),
        },
        std::io::empty(),
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, AdapterExitCode::ProtocolError.code());
    assert!(stdout.is_empty());
    let stderr = String::from_utf8(stderr).expect("stderr is UTF-8");
    assert!(stderr.contains("probe result adapter id mismatch"));
    assert!(stderr.contains("probe.adapter_id"));
    assert!(stderr.contains("\"stub\""));
    assert!(stderr.contains("\"drift\""));
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
