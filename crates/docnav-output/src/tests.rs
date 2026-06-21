// @case WB-OUTPUT-DOCUMENT-001
use super::*;
use docnav_diagnostics::{Warning, CLI_ARGV_IGNORED};
use docnav_protocol::{
    Entry, Operation, OperationResult, OutlineResult, ProtocolResponse, ReadResult,
    StableErrorCode, PROTOCOL_VERSION,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn warning() -> Warning {
    Warning::cli_argv_ignored(vec!["--future".to_owned()], "unknown CLI flag ignored")
}

fn read_result() -> OperationResult {
    OperationResult::Read(ReadResult {
        ref_id: "R1".into(),
        content: "body".into(),
        content_type: "text/plain".into(),
        cost: "1 lines | 4 bytes".into(),
        page: None,
    })
}

#[test]
fn readable_json_success_embeds_warnings_without_protocol_envelope() {
    let result = OperationResult::Outline(OutlineResult {
        entries: vec![Entry {
            ref_id: "R1".into(),
            display: "Intro".into(),
        }],
        page: None,
    });
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    write_document_result(
        &result,
        "request-1",
        DocumentOutputOptions::new(DocumentOutputMode::ReadableJson, &[warning()]),
        &mut stdout,
        &mut stderr,
    )
    .unwrap();

    assert!(stderr.is_empty());
    let value: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(value["entries"][0]["ref"], "R1");
    assert_eq!(value["warnings"][0]["id"], CLI_ARGV_IGNORED.as_str());
    assert!(value.get("protocol_version").is_none());
}

#[test]
fn protocol_json_success_writes_warning_to_stderr_only() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    write_document_result(
        &read_result(),
        "request-1",
        DocumentOutputOptions::new(DocumentOutputMode::ProtocolJson, &[warning()]),
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
fn readable_error_keeps_code_details_and_guidance() {
    let error = StableError {
        code: StableErrorCode::RefNotFound,
        message: "not found".into(),
        details: BTreeMap::from([("ref".to_owned(), json!("R99"))]),
        guidance: Some(vec!["Run outline first.".into()]),
    };
    let value = stable_error_readable(&error);
    assert_eq!(value["code"], "REF_NOT_FOUND");
    assert_eq!(value["details"]["ref"], "R99");
    assert_eq!(value["guidance"][0], "Run outline first.");
}

#[test]
fn response_failure_returns_failure_status() {
    let error = StableError::ref_not_found("R99");
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
