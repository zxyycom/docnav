// @case WB-OUTPUT-DOCUMENT-001
use super::*;
use docnav_diagnostics::{
    typed_codes, DiagnosticRecord, DiagnosticRecordDraft, DiagnosticSource, DiagnosticStack,
    RefDetails,
};
use docnav_protocol::{
    Cost, Entry, Measurement, Operation, OperationResult, OutlineResult, ProtocolResponse,
    ReadResult, PROTOCOL_VERSION,
};
use serde_json::Value;

fn ref_not_found_record() -> DiagnosticRecord {
    let mut diagnostics = DiagnosticStack::new();
    let id = diagnostics
        .push(
            DiagnosticRecordDraft::new::<typed_codes::protocol::RefNotFound>(
                "No content found for ref `R99`",
                RefDetails::new("R99"),
                DiagnosticSource::with_stage("test", "output"),
            )
            .with_guidance(["Run outline first."]),
        )
        .unwrap();
    diagnostics.get(id).unwrap().clone()
}

fn read_result() -> OperationResult {
    OperationResult::Read(ReadResult {
        ref_id: "R1".into(),
        content: "body".into(),
        content_type: "text/plain".into(),
        cost: test_cost(),
        page: None,
    })
}

fn test_cost() -> Cost {
    Cost {
        measurements: vec![
            Measurement {
                unit: "lines".to_owned(),
                value: 1,
                scope: Some("selection".to_owned()),
            },
            Measurement {
                unit: "bytes".to_owned(),
                value: 4,
                scope: Some("selection".to_owned()),
            },
        ],
    }
}

#[test]
fn readable_json_success_omits_diagnostics_without_protocol_envelope() {
    let result = OperationResult::Outline(OutlineResult {
        entries: vec![Entry {
            ref_id: "R1".into(),
            label: "Intro".into(),
            kind: None,
            location: None,
            summary: None,
            excerpt: None,
            rank: None,
            cost: None,
            metadata: None,
        }],
        page: None,
    });
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    write_document_result(
        &result,
        "request-1",
        DocumentOutputMode::ReadableJson,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();

    assert!(stderr.is_empty());
    let value: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(value["entries"][0]["ref"], "R1");
    assert!(value.get("protocol_version").is_none());
}

#[test]
fn protocol_json_success_writes_protocol_envelope_with_empty_stderr() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    write_document_result(
        &read_result(),
        "request-1",
        DocumentOutputMode::ProtocolJson,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();

    let stdout: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(stdout["request_id"], "request-1");
    let stderr = String::from_utf8(stderr).unwrap();
    assert!(stderr.is_empty());
}

#[test]
fn readable_view_read_uses_block_renderer() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    write_document_result(
        &read_result(),
        "request-1",
        DocumentOutputMode::ReadableView,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();

    assert!(stderr.is_empty());
    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("\"$block\": \"/content\""));
    assert!(output.contains("[block /content bytes=4]"));
    assert!(output.contains("body"));
}

#[test]
fn readable_error_keeps_code_details_and_guidance() {
    let error = ProtocolError::new::<typed_codes::protocol::RefNotFound>(
        "not found",
        RefDetails::new("R99"),
    )
    .with_guidance(["Run outline first."]);
    let value = protocol_error_readable(&error);
    assert_eq!(value["code"], "REF_NOT_FOUND");
    assert_eq!(value["owner"], "adapter");
    assert_eq!(value["location"]["ref"], "R99");
    assert_eq!(value["details"]["ref"], "R99");
    assert_eq!(value["guidance"][0], "Run outline first.");
}

#[test]
fn diagnostic_record_error_projects_readable_and_protocol_surfaces() {
    let error_record = ref_not_found_record();
    let mut readable_stdout = Vec::new();
    let mut readable_stderr = Vec::new();

    write_document_diagnostic_error(
        &error_record,
        ProtocolOutputContext::new(PROTOCOL_VERSION, "request-1", Some(Operation::Read)),
        DocumentOutputMode::ReadableJson,
        &mut readable_stdout,
        &mut readable_stderr,
    )
    .unwrap();

    assert!(readable_stderr.is_empty());
    let readable: Value = serde_json::from_slice(&readable_stdout).unwrap();
    assert_eq!(readable["code"], "REF_NOT_FOUND");
    assert_eq!(readable["owner"], "test_output");
    assert_eq!(readable["location"]["ref"], "R99");
    assert_eq!(readable["details"]["ref"], "R99");
    assert_eq!(readable["guidance"][0], "Run outline first.");

    let mut protocol_stdout = Vec::new();
    let mut protocol_stderr = Vec::new();
    write_document_diagnostic_error(
        &error_record,
        ProtocolOutputContext::new(PROTOCOL_VERSION, "request-1", Some(Operation::Read)),
        DocumentOutputMode::ProtocolJson,
        &mut protocol_stdout,
        &mut protocol_stderr,
    )
    .unwrap();

    assert!(protocol_stderr.is_empty());
    let protocol: Value = serde_json::from_slice(&protocol_stdout).unwrap();
    assert_eq!(protocol["error"]["code"], "REF_NOT_FOUND");
    assert_eq!(protocol["error"]["owner"], "test_output");
    assert_eq!(protocol["error"]["location"]["ref"], "R99");
    assert_eq!(protocol["error"]["details"]["ref"], "R99");
    assert_eq!(protocol["error"]["guidance"][0], "Run outline first.");
}

#[test]
fn response_failure_returns_failure_status() {
    let error = ProtocolError::ref_not_found("R99");
    let response =
        ProtocolResponse::failure(PROTOCOL_VERSION, "request-1", Some(Operation::Read), error);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let status = write_document_response(
        &response,
        DocumentOutputMode::ReadableJson,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();

    assert!(matches!(status, DocumentOutputStatus::Failure(_)));
    let value: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(value["code"], "REF_NOT_FOUND");
    assert_eq!(value["owner"], "adapter");
    assert_eq!(
        value["guidance"][0],
        "Run outline again and use a returned ref."
    );
}
