// @case WB-CORE-OUTPUT-001
use super::*;
use crate::cli::warning::{CliWarningDetails, CliWarningEffect, CLI_ARGV_IGNORED};
use docnav_diagnostics::{DiagnosticCode, ReadableWarningDiagnosticCode};
use docnav_protocol::{
    Entry, OperationResult, OutlineResult, ProtocolResponse, ReadResult, StableError,
    StableErrorCode,
};
use serde_json::json;
use std::collections::BTreeMap;

fn cli_argv_warning(tokens: &[&str]) -> CliWarning {
    CliWarning {
        id: CLI_ARGV_IGNORED,
        reason: "test warning".into(),
        effect: CliWarningEffect::OperationContinued,
        details: CliWarningDetails::CliArgv {
            tokens: tokens.iter().map(|s| s.to_string()).collect(),
        },
    }
}

fn error_details(map: &[(&str, &str)]) -> BTreeMap<String, Value> {
    map.iter().map(|(k, v)| (k.to_string(), json!(v))).collect()
}

fn write_success(outcome: CommandOutcome, warnings: &[CliWarning]) -> (Vec<u8>, Vec<u8>) {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = write_outcome(
        outcome,
        diagnostics_for_warnings(warnings),
        &mut stdout,
        &mut stderr,
    );
    assert_eq!(exit, 0);
    (stdout, stderr)
}

fn diagnostics_for_warnings(warnings: &[CliWarning]) -> DiagnosticStack {
    let mut diagnostics = DiagnosticStack::new();
    push_warning_diagnostics(
        &mut diagnostics,
        warnings,
        DiagnosticSource::with_stage("docnav", "cli"),
    );
    diagnostics
}

#[test]
fn plain_text_outcome_writes_text_directly() {
    let outcome = CommandOutcome::plain_text("hello world");
    let (stdout, _) = write_success(outcome, &[]);
    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("hello world"));
    assert!(!output.trim().starts_with('{'));
}

#[test]
fn non_document_json_warnings_stay_in_json_payload() {
    let outcome = CommandOutcome::json(json!({"config": "ok"}));
    let warning = cli_argv_warning(&["--extra"]);
    let (stdout, stderr) = write_success(outcome, &[warning]);
    assert!(stderr.is_empty());
    let value: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(value["config"], "ok");
    assert_eq!(value["warnings"][0]["id"], "cli_argv_ignored");
}

#[test]
fn runtime_and_cli_warnings_share_diagnostic_stack_projection() {
    let mut outcome = CommandOutcome::json(json!({"config": "ok"})).with_warnings(vec![
        CliWarning::adapter_candidate_failure(
            "markdown",
            "probe",
            "UNSUPPORTED",
            "no match",
            false,
        ),
    ]);
    let warning = cli_argv_warning(&["--extra"]);
    let runtime_record = outcome.diagnostics.snapshot()[0].clone();
    assert_eq!(
        runtime_record.code,
        DiagnosticCode::from(ReadableWarningDiagnosticCode::AdapterCandidateFailure)
    );

    push_warning_diagnostics(
        &mut outcome.diagnostics,
        std::slice::from_ref(&warning),
        DiagnosticSource::with_stage("docnav", "cli"),
    );
    let snapshot = outcome.diagnostics.snapshot();
    assert_eq!(
        snapshot[0].code,
        DiagnosticCode::from(ReadableWarningDiagnosticCode::CliArgvIgnored)
    );
    assert_eq!(
        outcome.diagnostics.get(runtime_record.id).unwrap().code,
        runtime_record.code
    );

    let (stdout, stderr) = write_success(outcome, &[]);

    assert!(stderr.is_empty());
    let value: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(value["warnings"][0]["id"], "adapter_candidate_failure");
    assert_eq!(value["warnings"][0]["details"]["adapter_id"], "markdown");
    assert_eq!(value["warnings"][1]["id"], "cli_argv_ignored");
    assert_eq!(value["warnings"][1]["details"]["tokens"][0], "--extra");
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
            cost: "1 lines | 4 bytes".into(),
            page: None,
        }),
    );
    let outcome = outcome_for_response(response, OutputMode::ReadableView).unwrap();
    let (stdout, _) = write_success(outcome, &[]);
    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("\"$block\": \"/content\""));
    assert!(output.contains("[block /content bytes=4]"));
}

#[test]
fn document_readable_json_embeds_warnings() {
    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "request-1",
        OperationResult::Outline(OutlineResult {
            entries: vec![Entry {
                ref_id: "R1".into(),
                display: "Test".into(),
            }],
            page: None,
        }),
    );
    let outcome = outcome_for_response(response, OutputMode::ReadableJson).unwrap();
    let warning = cli_argv_warning(&["--extra"]);
    let (stdout, stderr) = write_success(outcome, &[warning]);
    assert!(stderr.is_empty());
    let value: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(value["entries"][0]["ref"], "R1");
    assert_eq!(value["warnings"][0]["id"], "cli_argv_ignored");
    assert!(value.get("protocol_version").is_none());
}

#[test]
fn document_protocol_json_warnings_go_to_stderr() {
    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "request-1",
        OperationResult::Outline(OutlineResult {
            entries: vec![],
            page: None,
        }),
    );
    let outcome = outcome_for_response(response, OutputMode::ProtocolJson).unwrap();
    let warning = cli_argv_warning(&["--extra"]);
    let (stdout, stderr) = write_success(outcome, &[warning]);
    let stdout: Value = serde_json::from_slice(&stdout).unwrap();
    assert!(stdout.get("warnings").is_none());
    assert_eq!(stdout["protocol_version"], PROTOCOL_VERSION);
    let stderr = String::from_utf8(stderr).unwrap();
    assert!(stderr.contains("cli_argv_ignored"));
}

#[test]
fn readable_error_uses_document_facade_and_exit_policy_stays_local() {
    let error = AppError::new(StableError {
        code: StableErrorCode::RefNotFound,
        message: "No content found for ref `L99`".into(),
        details: error_details(&[("ref", "L99")]),
        guidance: Some(vec!["Check available entries.".into()]),
    });
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = write_error(ErrorOutput {
        error: &error,
        output_mode: OutputMode::ReadableView,
        operation: Some(Operation::Read),
        diagnostics: DiagnosticStack::new(),
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
fn output_mode_values_remain_unchanged() {
    assert_eq!(OutputMode::ReadableView.as_str(), "readable-view");
    assert_eq!(OutputMode::ReadableJson.as_str(), "readable-json");
    assert_eq!(OutputMode::ProtocolJson.as_str(), "protocol-json");
}
