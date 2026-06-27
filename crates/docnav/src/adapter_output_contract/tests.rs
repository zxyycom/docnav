// @case WB-CORE-ADAPTER-001
use super::*;
use crate::adapter_process::AdapterProcessOutput;
use crate::error::AppError;
use docnav_diagnostics::{DiagnosticCode, DiagnosticDetails, ProtocolDiagnosticCode};
use docnav_protocol::Operation;

fn adapter_output(stdout: impl Into<String>) -> AdapterProcessOutput {
    AdapterProcessOutput {
        stdout: stdout.into(),
        stderr: "adapter stderr".to_owned(),
        exit_code: Some(0),
    }
}

fn error_reason(error: &AppError) -> &str {
    let DiagnosticDetails::AdapterReason { reason, .. } = error.diagnostic().details() else {
        panic!("expected adapter invoke failure details");
    };
    reason
}

#[test]
fn protocol_response_schema_invalid_maps_to_adapter_invoke_failed() {
    let output = adapter_output(
        r#"{
              "protocol_version": "0.1",
              "request_id": "req-1",
              "operation": "outline",
              "ok": true,
              "result": { "page": null }
            }"#,
    );

    let error = protocol_response_from_output("stub", "req-1", Operation::Outline, output)
        .expect_err("schema-invalid response should fail");

    assert_eq!(
        error.diagnostic().code(),
        DiagnosticCode::from(ProtocolDiagnosticCode::AdapterInvokeFailed)
    );
    assert!(error_reason(&error).contains("protocol response schema validation failed"));
}

#[test]
fn protocol_response_semantic_invalid_maps_to_adapter_invoke_failed() {
    let output = adapter_output(
        r#"{
              "protocol_version": "0.1",
              "request_id": "other-req",
              "operation": "outline",
              "ok": true,
              "result": { "entries": [], "page": null }
            }"#,
    );

    let error = protocol_response_from_output("stub", "req-1", Operation::Outline, output)
        .expect_err("request id drift should fail after schema and typed decode");

    assert_eq!(
        error.diagnostic().code(),
        DiagnosticCode::from(ProtocolDiagnosticCode::AdapterInvokeFailed)
    );
    assert!(error_reason(&error).contains("response request_id does not match invoke request"));
}

#[test]
fn manifest_parse_schema_and_semantic_invalid_paths_are_distinct() {
    let malformed = manifest_from_output("stub", adapter_output("{not-json}"))
        .expect_err("malformed manifest stdout should fail");
    assert!(malformed.contains("stdout is not JSON"));

    let schema_invalid = manifest_from_output(
        "stub",
        adapter_output(r#"{ "manifest_version": "0.1", "adapter": {} }"#),
    )
    .expect_err("schema-invalid manifest should fail");
    assert!(schema_invalid.contains("manifest.schema.json"));

    let semantic_invalid = manifest_from_output(
        "stub",
        adapter_output(
            r#"{
                  "manifest_version": "0.1",
                  "adapter": {
                    "id": "other",
                    "name": "Other",
                    "version": "0.1.0"
                  },
                  "formats": [
                    {
                      "id": "stub",
                      "extensions": [".stub"],
                      "content_types": ["text/stub"]
                    }
                  ],
                  "capabilities": ["outline"]
                }"#,
        ),
    )
    .expect_err("manifest adapter id drift should fail");
    assert!(semantic_invalid.contains("does not match registry id"));
}

#[test]
fn probe_parse_schema_and_semantic_invalid_paths_are_distinct() {
    let malformed = probe_from_output("stub", "doc.stub", adapter_output("{not-json}"))
        .expect_err("malformed probe stdout should fail");
    assert!(malformed.contains("stdout is not JSON"));

    let schema_invalid = probe_from_output(
        "stub",
        "doc.stub",
        adapter_output(
            r#"{
                  "probe_version": "0.1",
                  "adapter_id": "stub",
                  "path": "doc.stub",
                  "supported": true,
                  "format": "stub",
                  "confidence": 2.0,
                  "reasons": []
                }"#,
        ),
    )
    .expect_err("schema-invalid probe should fail");
    assert!(schema_invalid.contains("probe-result.schema.json"));

    let semantic_invalid = probe_from_output(
        "stub",
        "doc.stub",
        adapter_output(
            r#"{
                  "probe_version": "0.1",
                  "adapter_id": "stub",
                  "path": "other.stub",
                  "supported": true,
                  "format": "stub",
                  "confidence": 1.0,
                  "reasons": [
                    { "code": "EXTENSION_MATCH", "detail": "extension matched" }
                  ]
                }"#,
        ),
    )
    .expect_err("probe path drift should fail after schema and typed decode");
    assert!(semantic_invalid.contains("does not match requested path"));
}
