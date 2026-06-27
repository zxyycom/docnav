// @case WB-DIAG-WARN-001
use serde_json::json;

use crate::{
    attach_warnings_to_value, warning_text_line, DiagnosticSource, DiagnosticStack,
    ReadableWarningDiagnosticCode, Warning, WarningEffect, WarningId, ADAPTER_CANDIDATE_FAILURE,
    ADAPTER_CONFIG_SOURCE_SKIPPED, CLI_ARGV_IGNORED,
};

#[test]
fn warning_ids_serialize_as_stable_strings() {
    assert_eq!(CLI_ARGV_IGNORED.as_str(), "cli_argv_ignored");
    assert_eq!(
        ADAPTER_CANDIDATE_FAILURE.as_str(),
        "adapter_candidate_failure"
    );
    assert_eq!(
        ADAPTER_CONFIG_SOURCE_SKIPPED.as_str(),
        "adapter_config_source_skipped"
    );
    assert_eq!(
        serde_json::to_value(WarningId::new("adapter_owned").unwrap()).unwrap(),
        json!("adapter_owned")
    );
    assert!(WarningId::new("AdapterOwned").is_err());
}

#[test]
fn argv_warning_constructors_keep_existing_shape() {
    let warning = Warning::unused_operation_flag("--page", Some("nope"), "info");
    assert_eq!(warning.id, CLI_ARGV_IGNORED);
    assert_eq!(warning.effect, WarningEffect::OperationContinued);
    assert_eq!(warning.reason, "flag is not used by info command");
    assert_eq!(
        serde_json::to_value(warning.details).unwrap(),
        json!({"tokens": ["--page", "nope"]})
    );
}

#[test]
fn adapter_candidate_warning_keeps_existing_shape() {
    let warning =
        Warning::adapter_candidate_failure("markdown", "probe", "UNSUPPORTED", "no match", true);
    assert_eq!(warning.id, ADAPTER_CANDIDATE_FAILURE);
    assert_eq!(warning.effect, WarningEffect::CandidateSkipped);
    assert_eq!(
        serde_json::to_value(warning.details).unwrap(),
        json!({
            "adapter_id": "markdown",
            "stage": "probe",
            "code": "UNSUPPORTED",
            "preselected": true
        })
    );
}

#[test]
fn adapter_config_source_warning_keeps_stable_shape() {
    let warning = Warning::adapter_config_source_skipped(
        "project",
        "override",
        "D:\\project\\.docnav\\docnav-markdown.json",
        "invalid_json",
    );

    assert_eq!(warning.id, ADAPTER_CONFIG_SOURCE_SKIPPED);
    assert_eq!(warning.effect, WarningEffect::OperationContinued);
    assert_eq!(
        serde_json::to_value(warning.details).unwrap(),
        json!({
            "source_level": "project",
            "path_origin": "override",
            "path": "D:\\project\\.docnav\\docnav-markdown.json",
            "reason_code": "invalid_json"
        })
    );
}

#[test]
fn known_warnings_roundtrip_through_diagnostic_records() {
    let warnings = vec![
        Warning::cli_argv_ignored(vec!["--future".to_owned()], "unknown CLI flag ignored"),
        Warning::adapter_candidate_failure("markdown", "probe", "UNSUPPORTED", "no match", true),
        Warning::adapter_config_source_skipped(
            "project",
            "override",
            "missing.json",
            "missing_override",
        ),
    ];

    for warning in warnings {
        let draft = warning
            .to_record_draft(DiagnosticSource::with_stage("docnav", "test"))
            .unwrap();
        let mut stack = DiagnosticStack::new();
        let id = stack.push(draft).unwrap();
        let record = stack.get(id).unwrap();

        assert_eq!(Warning::from_record(record), Some(warning));
    }
}

#[test]
fn warning_text_line_matches_stderr_contract() {
    let cases = [
        (
            Warning::cli_argv_ignored(vec!["--future".to_owned()], "unknown\nCLI flag ignored"),
            "warning: id=cli_argv_ignored, effect=operation_continued, reason=unknown CLI flag ignored, details={\"tokens\":[\"--future\"]}",
        ),
        (
            Warning::adapter_config_source_skipped(
                "project",
                "override",
                "missing.json",
                "missing_override",
            ),
            "warning: id=adapter_config_source_skipped, effect=operation_continued, reason=adapter config source skipped, details={\"source_level\":\"project\",\"path_origin\":\"override\",\"path\":\"missing.json\",\"reason_code\":\"missing_override\"}",
        ),
    ];

    for (warning, expected) in cases {
        assert_eq!(warning_text_line(&warning).unwrap(), expected);
    }
}

#[test]
fn attach_warnings_keeps_json_payload_shape() {
    let warning = Warning::cli_argv_ignored(vec!["--future".to_owned()], "test warning");

    let object = attach_warnings_to_value(json!({"ok": true}), std::slice::from_ref(&warning));
    assert_eq!(object["ok"], json!(true));
    assert_eq!(object["warnings"][0]["id"], "cli_argv_ignored");

    let scalar = attach_warnings_to_value(json!("ok"), &[warning]);
    assert_eq!(scalar["value"], json!("ok"));
    assert_eq!(scalar["warnings"][0]["id"], "cli_argv_ignored");
}

#[test]
fn readable_warning_codes_parse_known_warning_ids() {
    assert_eq!(
        ReadableWarningDiagnosticCode::from_warning_id("cli_argv_ignored"),
        Some(ReadableWarningDiagnosticCode::CliArgvIgnored)
    );
    assert_eq!(
        ReadableWarningDiagnosticCode::from_warning_id("adapter_candidate_failure"),
        Some(ReadableWarningDiagnosticCode::AdapterCandidateFailure)
    );
    assert_eq!(
        ReadableWarningDiagnosticCode::from_warning_id("adapter_owned"),
        None
    );
}
