use super::common::{positive, HandlerErrorAdapter, ManifestShapeErrorAdapter, StubAdapter};
use crate::invoke::invoke_once_with_default_limit;
use crate::{invoke_once, Adapter, AdapterExitCode, AdapterResult};
use docnav_protocol::{
    AdapterIdentity, Entry, FailureResponse, FormatDescriptor, Manifest, Operation,
    OutlineArguments, OutlineResult, ProbeResult, ProtocolDiagnosticCode, ProtocolResponse,
    RequestEnvelope, MANIFEST_VERSION, PROBE_VERSION, PROTOCOL_VERSION, UNKNOWN_REQUEST_ID,
};

// @case WB-SDK-INVOKE-001
#[test]
fn invoke_reads_one_request_and_writes_one_protocol_response() {
    let input = br#"{
          "protocol_version": "0.1",
          "request_id": "req-1",
          "operation": "outline",
          "document": { "path": "sample.stub" },
          "arguments": { "limit": 80, "page": 1 }
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
fn invoke_standard_parameter_normalization_preserves_options_passthrough() {
    let input = br#"{
          "protocol_version": "0.1",
          "request_id": "req-opts",
          "operation": "outline",
          "document": { "path": "sample.stub" },
          "arguments": {
            "limit": 80,
            "page": 1,
            "options": { "required_by_test": true }
          }
        }"#;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = invoke_once(
        &OptionsRequiredAdapter,
        &input[..],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, AdapterExitCode::Success.code());
    assert!(stderr.is_empty());
    let response: ProtocolResponse =
        serde_json::from_slice(&stdout).expect("stdout is one JSON response");
    response.validate().expect("response validates");
}

#[test]
fn invoke_standard_parameter_normalization_supplies_default_window() {
    let input = br#"{
          "protocol_version": "0.1",
          "request_id": "req-defaults",
          "operation": "outline",
          "document": { "path": "sample.stub" },
          "arguments": {}
        }"#;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = invoke_once_with_default_limit(
        &DefaultWindowRequiredAdapter {
            expected_limit: 123,
        },
        123,
        &input[..],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, AdapterExitCode::Success.code());
    assert!(stderr.is_empty());
    let response: ProtocolResponse =
        serde_json::from_slice(&stdout).expect("stdout is one JSON response");
    response.validate().expect("response validates");
}

#[test]
fn invoke_protocol_decode_accepts_unmapped_arguments_before_standardization() {
    let input = br#"{
          "protocol_version": "0.1",
          "request_id": "req-unmapped",
          "operation": "outline",
          "document": { "path": "sample.stub" },
          "arguments": {
            "limit": 80,
            "page": 1,
            "future_argument": true
          }
        }"#;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = invoke_once(&StubAdapter, &input[..], &mut stdout, &mut stderr);

    assert_eq!(exit, AdapterExitCode::Success.code());
    assert!(stderr.is_empty());
    let response: ProtocolResponse =
        serde_json::from_slice(&stdout).expect("stdout is one JSON response");
    response.validate().expect("response validates");
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
    let response = failure_response_from_stdout(&stdout);
    assert_eq!(response.protocol_version, PROTOCOL_VERSION);
    assert_eq!(response.request_id, UNKNOWN_REQUEST_ID);
    assert_eq!(response.operation, None);
    assert_eq!(
        response.error.code(),
        ProtocolDiagnosticCode::InvalidRequest
    );
}

#[test]
fn unsupported_protocol_version_is_invalid_request_schema_failure() {
    let input = br#"{
          "protocol_version": "1.0",
          "request_id": "req-1",
          "operation": "outline",
          "document": { "path": "sample.stub" },
          "arguments": { "limit": 80, "page": 1 }
        }"#;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = invoke_once(&StubAdapter, &input[..], &mut stdout, &mut stderr);

    assert_eq!(exit, AdapterExitCode::ProtocolError.code());
    assert!(!stderr.is_empty());
    let response = failure_response_from_stdout(&stdout);
    assert_eq!(response.protocol_version, PROTOCOL_VERSION);
    assert_eq!(
        response.error.code(),
        ProtocolDiagnosticCode::InvalidRequest
    );
    assert_eq!(response.operation, Some(Operation::Outline));
    assert_eq!(response.request_id, "req-1");
}

#[test]
fn request_schema_failure_without_version_uses_current_protocol_version() {
    let input = br#"{
          "request_id": "req-1",
          "operation": "outline",
          "document": { "path": "sample.stub" },
          "arguments": { "limit": 80, "page": 1 }
        }"#;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = invoke_once(&StubAdapter, &input[..], &mut stdout, &mut stderr);

    assert_eq!(exit, AdapterExitCode::ProtocolError.code());
    assert!(!stderr.is_empty());
    let response = failure_response_from_stdout(&stdout);
    assert_eq!(response.protocol_version, PROTOCOL_VERSION);
    assert_eq!(response.operation, Some(Operation::Outline));
    assert_eq!(response.request_id, "req-1");
    assert_eq!(
        response.error.code(),
        ProtocolDiagnosticCode::InvalidRequest
    );
}

#[test]
fn invoke_rejects_invalid_manifest_shape_without_protocol_envelope() {
    let input = br#"{
          "protocol_version": "0.1",
          "request_id": "req-1",
          "operation": "outline",
          "document": { "path": "sample.stub" },
          "arguments": { "limit": 80, "page": 1 }
        }"#;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = invoke_once(
        &ManifestShapeErrorAdapter,
        &input[..],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, AdapterExitCode::ProtocolError.code());
    assert!(stdout.is_empty());
    let stderr = String::from_utf8(stderr).expect("stderr is UTF-8");
    assert!(stderr.contains("manifest schema validation failed"));
}

#[test]
fn request_schema_rejections_are_structured_invalid_request_failures() {
    let input = br#"{
          "protocol_version": "0.1",
          "request_id": "",
          "operation": "read",
          "document": { "path": "sample.stub" },
          "arguments": { "ref": "", "limit": 80, "page": 1 }
        }"#;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = invoke_once(&StubAdapter, &input[..], &mut stdout, &mut stderr);

    assert_eq!(exit, AdapterExitCode::ProtocolError.code());
    assert!(!stderr.is_empty());
    let response = failure_response_from_stdout(&stdout);
    assert_eq!(response.request_id, UNKNOWN_REQUEST_ID);
    assert_eq!(response.operation, Some(Operation::Read));
    assert_eq!(
        response.error.code(),
        ProtocolDiagnosticCode::InvalidRequest
    );
}

#[test]
fn handler_error_projects_diagnostic_failure_to_protocol_stdout() {
    let input = br#"{
          "protocol_version": "0.1",
          "request_id": "req-1",
          "operation": "read",
          "document": { "path": "sample.stub" },
          "arguments": { "ref": "L1:Stub", "limit": 80, "page": 1 }
        }"#;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = invoke_once(&HandlerErrorAdapter, &input[..], &mut stdout, &mut stderr);

    assert_eq!(exit, AdapterExitCode::HandlerError.code());
    assert!(stderr.is_empty());
    let response = failure_response_from_stdout(&stdout);
    assert_eq!(response.error.code(), ProtocolDiagnosticCode::RefNotFound);
    assert_eq!(response.error.details()["ref"], "L1:Stub");
}

fn failure_response_from_stdout(stdout: &[u8]) -> FailureResponse {
    let response: ProtocolResponse =
        serde_json::from_slice(stdout).expect("stdout is one JSON response");
    match response {
        ProtocolResponse::Failure(response) => response,
        ProtocolResponse::Success(_) => panic!("expected failure response"),
    }
}

struct OptionsRequiredAdapter;

impl Adapter for OptionsRequiredAdapter {
    fn adapter_id(&self) -> &str {
        "options-required"
    }

    fn manifest(&self) -> Manifest {
        Manifest {
            manifest_version: MANIFEST_VERSION.to_owned(),
            adapter: AdapterIdentity {
                id: "options-required".to_owned(),
                name: "Options Required".to_owned(),
                version: "0.1.0".to_owned(),
            },
            formats: vec![FormatDescriptor {
                id: "stub".to_owned(),
                extensions: vec![".stub".to_owned()],
                content_types: vec!["text/stub".to_owned()],
            }],
            capabilities: vec![Operation::Outline],
        }
    }

    fn probe(&self, path: &str) -> ProbeResult {
        ProbeResult {
            probe_version: PROBE_VERSION.to_owned(),
            adapter_id: "options-required".to_owned(),
            path: path.to_owned(),
            supported: true,
            format: Some("stub".to_owned()),
            confidence: 1.0,
            reasons: Vec::new(),
        }
    }

    fn outline(
        &self,
        _request: &RequestEnvelope,
        arguments: &OutlineArguments,
    ) -> AdapterResult<OutlineResult> {
        let Some(options) = &arguments.options else {
            return Err(crate::AdapterError::invalid_request(
                "arguments.options",
                "missing options",
            ));
        };
        if options
            .get("required_by_test")
            .and_then(serde_json::Value::as_bool)
            != Some(true)
        {
            return Err(crate::AdapterError::invalid_request(
                "arguments.options.required_by_test",
                "missing required_by_test option",
            ));
        }
        Ok(OutlineResult {
            entries: vec![Entry {
                ref_id: "L1:Options".to_owned(),
                label: "options preserved".to_owned(),
                kind: None,
                location: None,
                summary: None,
                excerpt: None,
                rank: None,
                cost: None,
                metadata: None,
            }],
            page: None,
        })
    }
}

struct DefaultWindowRequiredAdapter {
    expected_limit: u32,
}

impl Adapter for DefaultWindowRequiredAdapter {
    fn adapter_id(&self) -> &str {
        "default-window-required"
    }

    fn manifest(&self) -> Manifest {
        Manifest {
            manifest_version: MANIFEST_VERSION.to_owned(),
            adapter: AdapterIdentity {
                id: "default-window-required".to_owned(),
                name: "Default Window Required".to_owned(),
                version: "0.1.0".to_owned(),
            },
            formats: vec![FormatDescriptor {
                id: "stub".to_owned(),
                extensions: vec![".stub".to_owned()],
                content_types: vec!["text/stub".to_owned()],
            }],
            capabilities: vec![Operation::Outline],
        }
    }

    fn probe(&self, path: &str) -> ProbeResult {
        ProbeResult {
            probe_version: PROBE_VERSION.to_owned(),
            adapter_id: "default-window-required".to_owned(),
            path: path.to_owned(),
            supported: true,
            format: Some("stub".to_owned()),
            confidence: 1.0,
            reasons: Vec::new(),
        }
    }

    fn outline(
        &self,
        _request: &RequestEnvelope,
        arguments: &OutlineArguments,
    ) -> AdapterResult<OutlineResult> {
        assert_eq!(arguments.page, positive(1));
        assert_eq!(arguments.limit, positive(self.expected_limit));
        Ok(OutlineResult {
            entries: vec![Entry {
                ref_id: "L1:Defaults".to_owned(),
                label: "defaults supplied".to_owned(),
                kind: None,
                location: None,
                summary: None,
                excerpt: None,
                rank: None,
                cost: None,
                metadata: None,
            }],
            page: None,
        })
    }
}
