// @case WB-OUTPUT-DOCUMENT-001
use super::*;
use docnav_diagnostics::{
    typed_codes, DiagnosticRecord, DiagnosticRecordDraft, DiagnosticSource, DiagnosticStack,
    RefDetails, WarningProjection, CLI_ARGV_IGNORED,
};
use docnav_protocol::{
    Cost, Entry, Measurement, Operation, OperationResult, OutlineResult, ProtocolResponse,
    ReadResult, PROTOCOL_VERSION,
};
use serde_json::Value;

fn warning() -> WarningProjection {
    WarningProjection::cli_argv_ignored(vec!["--future".to_owned()], "unknown CLI flag ignored")
}

fn warning_records() -> Vec<DiagnosticRecord> {
    let mut diagnostics = DiagnosticStack::new();
    diagnostics
        .push(warning().to_record_draft(DiagnosticSource::with_stage("test", "output")))
        .unwrap();
    let mut records = diagnostics.snapshot();
    records.reverse();
    records
}

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
fn readable_json_success_embeds_warnings_without_protocol_envelope() {
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
    let diagnostics = warning_records();

    write_document_result(
        &result,
        "request-1",
        DocumentOutputOptions::new(DocumentOutputMode::ReadableJson, &diagnostics),
        &mut stdout,
        &mut stderr,
    )
    .unwrap();

    assert!(stderr.is_empty());
    let value: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(value["entries"][0]["ref"], "R1");
    assert_eq!(value["warnings"][0]["id"], CLI_ARGV_IGNORED.warning_id());
    assert_ne!(value["warnings"][0]["effect"], "diagnostic_only");
    assert!(value.get("protocol_version").is_none());
}

#[test]
fn protocol_json_success_writes_warning_to_stderr_only() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let diagnostics = warning_records();

    write_document_result(
        &read_result(),
        "request-1",
        DocumentOutputOptions::new(DocumentOutputMode::ProtocolJson, &diagnostics),
        &mut stdout,
        &mut stderr,
    )
    .unwrap();

    let stdout: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(stdout["request_id"], "request-1");
    assert!(stdout.get("warnings").is_none());
    let stderr = String::from_utf8(stderr).unwrap();
    assert!(stderr.contains("cli_argv_ignored"));
}

#[test]
fn readable_view_read_uses_block_renderer() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    write_document_result(
        &read_result(),
        "request-1",
        DocumentOutputOptions::new(DocumentOutputMode::ReadableView, &[]),
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
fn readable_view_projects_record_warnings_without_diagnostic_only_effect() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let diagnostics = warning_records();

    write_document_result(
        &read_result(),
        "request-1",
        DocumentOutputOptions::new(DocumentOutputMode::ReadableView, &diagnostics),
        &mut stdout,
        &mut stderr,
    )
    .unwrap();

    assert!(stderr.is_empty());
    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("cli_argv_ignored"));
    assert!(!output.contains("diagnostic_only"));
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
    assert_eq!(value["details"]["ref"], "R99");
    assert_eq!(value["guidance"][0], "Run outline first.");
}

#[test]
fn diagnostic_record_error_projects_readable_and_protocol_surfaces() {
    let error_record = ref_not_found_record();
    let diagnostics = vec![error_record.clone()];
    let mut readable_stdout = Vec::new();
    let mut readable_stderr = Vec::new();

    write_document_diagnostic_error(
        &error_record,
        ProtocolOutputContext::new(PROTOCOL_VERSION, "request-1", Some(Operation::Read)),
        DocumentOutputOptions::new(DocumentOutputMode::ReadableJson, &diagnostics),
        &mut readable_stdout,
        &mut readable_stderr,
    )
    .unwrap();

    assert!(readable_stderr.is_empty());
    let readable: Value = serde_json::from_slice(&readable_stdout).unwrap();
    assert_eq!(readable["code"], "REF_NOT_FOUND");
    assert_eq!(readable["details"]["ref"], "R99");
    assert_eq!(readable["guidance"][0], "Run outline first.");

    let mut protocol_stdout = Vec::new();
    let mut protocol_stderr = Vec::new();
    write_document_diagnostic_error(
        &error_record,
        ProtocolOutputContext::new(PROTOCOL_VERSION, "request-1", Some(Operation::Read)),
        DocumentOutputOptions::new(DocumentOutputMode::ProtocolJson, &diagnostics),
        &mut protocol_stdout,
        &mut protocol_stderr,
    )
    .unwrap();

    assert!(protocol_stderr.is_empty());
    let protocol: Value = serde_json::from_slice(&protocol_stdout).unwrap();
    assert_eq!(protocol["error"]["code"], "REF_NOT_FOUND");
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
        &[],
        &mut stdout,
        &mut stderr,
    )
    .unwrap();

    assert!(matches!(status, DocumentOutputStatus::Failure(_)));
    let value: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(value["code"], "REF_NOT_FOUND");
}
