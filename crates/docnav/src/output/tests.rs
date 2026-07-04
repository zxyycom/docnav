// @case WB-CORE-OUTPUT-001
use super::*;
use docnav_diagnostics::{
    typed_codes, BoundaryDetails, DiagnosticRecordDraft, DiagnosticSource, RefDetails,
};
use docnav_protocol::{
    protocol_error_record_draft_with_summary, Cost, Entry, Measurement, OperationResult,
    OutlineResult, ProtocolResponse, ReadResult,
};
use serde_json::{json, Value};

fn write_success(outcome: CommandOutcome) -> (Vec<u8>, Vec<u8>) {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = write_outcome(outcome, &mut stdout, &mut stderr);
    assert_eq!(exit, 0);
    (stdout, stderr)
}

#[test]
fn plain_text_outcome_writes_text_directly() {
    let outcome = CommandOutcome::plain_text("hello world");
    let (stdout, _) = write_success(outcome);
    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("hello world"));
    assert!(!output.trim().starts_with('{'));
}

#[test]
fn non_document_json_writes_value_directly() {
    let outcome = CommandOutcome::json(json!({"config": "ok"}));
    let (stdout, stderr) = write_success(outcome);
    assert!(stderr.is_empty());
    let value: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(value["config"], "ok");
}

#[test]
fn document_readable_view_uses_shared_output_facade() {
    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "request-1",
        OperationResult::Read(ReadResult {
            ref_id: "R1".into(),
            content: "body".into(),
            content_type: "text/plain".into(),
            cost: test_cost(),
            page: None,
        }),
    );
    let outcome = outcome_for_response(response, OutputMode::ReadableView).unwrap();
    let (stdout, _) = write_success(outcome);
    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("\"$block\": \"/content\""));
    assert!(output.contains("[block /content bytes=4]"));
}

#[test]
fn document_readable_json_uses_success_payload_without_protocol_envelope() {
    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "request-1",
        OperationResult::Outline(OutlineResult {
            entries: vec![Entry {
                ref_id: "R1".into(),
                label: "Test".into(),
                kind: None,
                location: None,
                summary: None,
                excerpt: None,
                rank: None,
                cost: None,
                metadata: None,
            }],
            page: None,
        }),
    );
    let outcome = outcome_for_response(response, OutputMode::ReadableJson).unwrap();
    let (stdout, stderr) = write_success(outcome);
    assert!(stderr.is_empty());
    let value: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(value["entries"][0]["ref"], "R1");
    assert!(value.get("protocol_version").is_none());
}

#[test]
fn document_protocol_json_writes_protocol_envelope_with_empty_stderr() {
    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "request-1",
        OperationResult::Outline(OutlineResult {
            entries: vec![],
            page: None,
        }),
    );
    let outcome = outcome_for_response(response, OutputMode::ProtocolJson).unwrap();
    let (stdout, stderr) = write_success(outcome);
    let stdout: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(stdout["protocol_version"], PROTOCOL_VERSION);
    let stderr = String::from_utf8(stderr).unwrap();
    assert!(stderr.is_empty());
}

#[test]
fn readable_error_uses_document_facade_and_exit_policy_stays_local() {
    let error = AppError::new(
        protocol_error_record_draft_with_summary::<typed_codes::protocol::RefNotFound>(
            "No content found for ref `L99`",
            RefDetails::new("L99"),
            DiagnosticSource::with_stage("test", "output"),
        )
        .with_guidance(["Check available entries."]),
    );
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = write_error(ErrorOutput {
        error: &error,
        output_mode: OutputMode::ReadableView,
        operation: Some(Operation::Read),
        stdout: &mut stdout,
        stderr: &mut stderr,
    });
    assert_eq!(exit, DocnavExitCode::DocumentError.code());
    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("[block /error bytes="));
    assert!(output.contains("\"code\": \"REF_NOT_FOUND\""));
    assert!(output.contains("Check available entries."));
}

#[test]
fn app_error_normalizes_non_protocol_diagnostic_before_document_output() {
    let error = AppError::new(DiagnosticRecordDraft::new::<
        typed_codes::boundary::FailedToWriteJson,
    >(
        "failed to write JSON output",
        BoundaryDetails::new("stdout closed"),
        DiagnosticSource::with_stage("test", "output"),
    ));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = write_error(ErrorOutput {
        error: &error,
        output_mode: OutputMode::ProtocolJson,
        operation: None,
        stdout: &mut stdout,
        stderr: &mut stderr,
    });

    assert_eq!(exit, DocnavExitCode::InternalError.code());
    assert!(stderr.is_empty());
    let output: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(output["error"]["code"], "INTERNAL_ERROR");
    assert_eq!(
        output["error"]["details"]["error_id"],
        "app-error-diagnostic-not-protocol"
    );
}

#[test]
fn document_output_error_projects_primary_internal_diagnostic_when_possible() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = write_document_output_error(
        DocumentOutputError::DiagnosticProjection,
        DocumentOutputMode::ProtocolJson,
        Some(Operation::Read),
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, DocnavExitCode::InternalError.code());
    assert!(stderr.is_empty());
    let output: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(output["error"]["code"], "INTERNAL_ERROR");
    assert_eq!(
        output["error"]["details"]["error_id"],
        "diagnostic-projection-failed"
    );
    assert_eq!(output["operation"], "read");
}

#[test]
fn output_mode_values_remain_unchanged() {
    assert_eq!(OutputMode::ReadableView.as_str(), "readable-view");
    assert_eq!(OutputMode::ReadableJson.as_str(), "readable-json");
    assert_eq!(OutputMode::ProtocolJson.as_str(), "protocol-json");
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
