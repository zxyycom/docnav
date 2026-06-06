use super::common::{
    ManifestProtocolRangeAdapter, ManifestSemanticErrorAdapter, MissingDetailsErrorAdapter,
    StubAdapter,
};
use crate::{invoke_once, AdapterExitCode};
use docnav_protocol::{
    Operation, ProtocolResponse, StableErrorCode, PROTOCOL_VERSION, UNKNOWN_REQUEST_ID,
};

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
