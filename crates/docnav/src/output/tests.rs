// @case WB-CORE-OUTPUT-001
use super::*;
use docnav_diagnostics::{
    typed_codes, CliArgvDetails, DiagnosticCode, DiagnosticRecordDraft,
    ReadableWarningDiagnosticCode, RefDetails,
};
use docnav_protocol::{
    protocol_error_record_draft_with_summary, Cost, Entry, Measurement, OperationResult,
    OutlineResult, ProtocolResponse, ReadResult,
};
use serde_json::json;

fn cli_argv_warning(tokens: &[&str]) -> CliWarning {
    CliWarning::new::<typed_codes::readable_warning::CliArgvIgnored>(
        "test warning",
        CliArgvDetails::new(tokens.iter().map(|s| s.to_string()).collect::<Vec<_>>()),
    )
    .expect("test warning reason is non-empty")
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
        runtime_record.code(),
        DiagnosticCode::from(ReadableWarningDiagnosticCode::AdapterCandidateFailure)
    );

    push_warning_diagnostics(
        &mut outcome.diagnostics,
        std::slice::from_ref(&warning),
        DiagnosticSource::with_stage("docnav", "cli"),
    );
    let snapshot = outcome.diagnostics.snapshot();
    assert_eq!(
        snapshot[0].code(),
        DiagnosticCode::from(ReadableWarningDiagnosticCode::CliArgvIgnored)
    );
    assert_eq!(
        outcome.diagnostics.get(runtime_record.id()).unwrap().code(),
        runtime_record.code()
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
            cost: test_cost(),
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
fn related_warning_diagnostics_are_preserved_on_error_output() {
    let warning = CliWarning::adapter_candidate_failure(
        "markdown",
        "probe",
        "UNSUPPORTED",
        "no match",
        false,
    );
    let error = AppError::new(protocol_error_record_draft_with_summary::<
        typed_codes::protocol::RefNotFound,
    >(
        "No content found for ref `L99`",
        RefDetails::new("L99"),
        DiagnosticSource::with_stage("test", "output"),
    ))
    .with_related_diagnostics([
        warning.to_record_draft(DiagnosticSource::with_stage("docnav", "runtime"))
    ]);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = write_error(ErrorOutput {
        error: &error,
        output_mode: OutputMode::ReadableJson,
        operation: Some(Operation::Read),
        diagnostics: DiagnosticStack::new(),
        stdout: &mut stdout,
        stderr: &mut stderr,
    });

    assert_eq!(exit, DocnavExitCode::DocumentError.code());
    assert!(stderr.is_empty());
    let output: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(output["code"], "REF_NOT_FOUND");
    assert_eq!(output["warnings"][0]["id"], "adapter_candidate_failure");
    assert_eq!(output["warnings"][0]["details"]["adapter_id"], "markdown");
}

#[test]
fn app_error_normalizes_non_protocol_diagnostic_before_document_output() {
    let error = AppError::new(DiagnosticRecordDraft::new::<
        typed_codes::readable_warning::CliArgvIgnored,
    >(
        "ignored argv is not a fatal protocol error",
        CliArgvDetails::new(vec!["--unused".into()]),
        DiagnosticSource::with_stage("test", "output"),
    ));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = write_error(ErrorOutput {
        error: &error,
        output_mode: OutputMode::ProtocolJson,
        operation: None,
        diagnostics: DiagnosticStack::new(),
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
