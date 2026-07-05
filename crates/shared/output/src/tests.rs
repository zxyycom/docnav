// @case WB-OUTPUT-DOCUMENT-001
use super::*;
use docnav_diagnostics::{
    typed_codes, DiagnosticRecord, DiagnosticRecordDraft, DiagnosticSource, RefDetails,
};
use docnav_protocol::{
    Cost, Entry, Measurement, Operation, OperationResult, OutlineResult, ProtocolResponse,
    ReadResult, UnstructuredOutlineReason, PROTOCOL_VERSION,
};
use serde_json::Value;

fn ref_not_found_record() -> DiagnosticRecord {
    DiagnosticRecordDraft::new::<typed_codes::protocol::RefNotFound>(
        "No content found for ref `R99`",
        RefDetails::new("R99"),
        DiagnosticSource::with_stage("test", "output"),
    )
    .with_guidance(["Run outline first."])
    .into_record()
    .unwrap()
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

fn unstructured_outline_result(cost: Cost) -> OperationResult {
    OperationResult::Outline(OutlineResult::unstructured(
        UnstructuredOutlineReason::PathRule,
        "full body\n",
        "text/markdown",
        cost,
    ))
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
            Measurement {
                unit: "tokens".to_owned(),
                value: 8,
                scope: Some("selection".to_owned()),
            },
        ],
    }
}

fn empty_cost() -> Cost {
    Cost {
        measurements: Vec::new(),
    }
}

#[test]
fn readable_json_success_omits_diagnostics_without_protocol_envelope() {
    let result = OperationResult::Outline(OutlineResult::structured(
        vec![Entry {
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
        None,
    ));
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
    assert_eq!(value["kind"], "structured");
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
fn readable_json_read_cost_is_derived_from_measurements() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    write_document_result(
        &read_result(),
        "request-1",
        DocumentOutputMode::ReadableJson,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();

    assert!(stderr.is_empty());
    let value: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(value["cost"], "1 line | 0.0 KB | 8 tokens");
}

#[test]
fn readable_json_unstructured_outline_preserves_content_cost_and_reason() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    write_document_result(
        &unstructured_outline_result(empty_cost()),
        "request-1",
        DocumentOutputMode::ReadableJson,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();

    assert!(stderr.is_empty());
    let value: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(value["kind"], "unstructured");
    assert_eq!(value["reason"], "path_rule");
    assert_eq!(value["content"], "full body\n");
    assert_eq!(value["content_type"], "text/markdown");
    assert_eq!(value["cost"]["measurements"].as_array().unwrap().len(), 0);
    assert!(value.get("entries").is_none());
    assert!(value.get("ref").is_none());
    assert!(value.get("page").is_none());
    assert!(value.get("continuation").is_none());
}

#[test]
fn readable_view_unstructured_outline_uses_content_block() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    write_document_result(
        &unstructured_outline_result(test_cost()),
        "request-1",
        DocumentOutputMode::ReadableView,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();

    assert!(stderr.is_empty());
    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("\"kind\": \"unstructured\""));
    assert!(output.contains("\"reason\": \"path_rule\""));
    assert!(output.contains("\"$block\": \"/content\""));
    assert!(output.contains("[block /content bytes=10]"));
    assert!(output.contains("full body\n"));
    assert!(output.contains("\"measurements\""));
    assert!(!output.contains("\"entries\""));
    assert!(!output.contains("\"page\""));
}

#[test]
fn protocol_json_unstructured_outline_uses_union_branch_without_navigation_fields() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    write_document_result(
        &unstructured_outline_result(empty_cost()),
        "request-1",
        DocumentOutputMode::ProtocolJson,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();

    assert!(stderr.is_empty());
    let value: Value = serde_json::from_slice(&stdout).unwrap();
    let result = &value["result"];
    assert_eq!(value["operation"], "outline");
    assert_eq!(result["kind"], "unstructured");
    assert_eq!(result["reason"], "path_rule");
    assert_eq!(result["content"], "full body\n");
    assert_eq!(result["cost"]["measurements"].as_array().unwrap().len(), 0);
    assert!(result.get("entries").is_none());
    assert!(result.get("ref").is_none());
    assert!(result.get("page").is_none());
    assert!(result.get("continuation").is_none());
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

#[test]
fn readable_view_render_error_uses_stable_primary_error_id() {
    let error = DocumentOutputError::ReadableViewRender(docnav_readable::RenderError::new("boom"));

    assert!(error.can_project_as_primary_diagnostic());
    assert_eq!(error.primary_error_id(), "readable_view_render_failed");
}
