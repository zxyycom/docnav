use docnav_protocol::{OperationResult, ProtocolDiagnosticCode, ProtocolResponse};
use serde_json::{json, Value};

use crate::{execute_loaded_navigation_command, NavigationOutputMode};

use super::super::support::{
    cli_value_candidate, config_sources, navigation_command, StubRegistry,
};

// @case WB-NAVIGATION-HARD-CUTOVER-001
#[test]
fn hard_cutover_preserves_common_and_native_option_source_priority() {
    let command = navigation_command(vec![
        cli_value_candidate(
            "docnav.adapters.docnav-markdown.options.max_heading_level",
            "--max-heading-level",
            json!(4),
        ),
        cli_value_candidate("docnav.defaults.pagination.limit", "--limit", json!(42)),
        cli_value_candidate("docnav.defaults.output", "--output", json!("protocol-json")),
    ]);

    let outcome = execute_loaded_navigation_command(
        command,
        config_sources(
            json!({
                "defaults": {
                    "pagination": { "limit": 120 },
                    "output": "readable-view"
                },
                "options": {
                    "docnav-markdown": { "max_heading_level": 2 }
                }
            }),
            json!({
                "defaults": {
                    "pagination": { "limit": 240 },
                    "output": "readable-view"
                },
                "options": {
                    "docnav-markdown": { "max_heading_level": 1 }
                }
            }),
        ),
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect("hard cutover priority outcome");

    assert_eq!(outcome.output, NavigationOutputMode::ProtocolJson);
    let ProtocolResponse::Success(success) = outcome.response else {
        panic!("expected success");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };
    let result = result.as_structured().expect("structured outline result");
    assert_eq!(result.entries[0].label, "Max 4");
}

#[test]
fn valid_explicit_common_value_does_not_hide_invalid_project_config() {
    let command = navigation_command(vec![cli_value_candidate(
        "docnav.defaults.output",
        "--output",
        json!("protocol-json"),
    )]);

    let error = execute_loaded_navigation_command(
        command,
        config_sources(
            json!({"defaults": {"output": "readable-json"}}),
            Value::Null,
        ),
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("invalid project config remains blocking");
    assert_source_diagnostic(
        error.diagnostic(),
        "defaults.output",
        "enum_invalid",
        "project",
        "project/.docnav/docnav.json",
    );
}

#[test]
fn removed_readable_json_cli_value_is_rejected_by_canonical_resolution() {
    let error = execute_loaded_navigation_command(
        navigation_command(vec![cli_value_candidate(
            "docnav.defaults.output",
            "--output",
            json!("readable-json"),
        )]),
        config_sources(Value::Null, Value::Null),
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("removed readable-json output must be rejected");
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error.code(),
        ProtocolDiagnosticCode::InvalidRequest
    );
    assert!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str)
            .is_some_and(|reason| {
                reason.contains("accepted values: readable-view, protocol-json")
            }),
        "{protocol_error:?}"
    );
}

#[test]
fn valid_explicit_native_value_does_not_hide_invalid_user_config() {
    let error = execute_loaded_navigation_command(
        navigation_command(vec![cli_value_candidate(
            "docnav.adapters.docnav-markdown.options.max_heading_level",
            "--max-heading-level",
            json!(4),
        )]),
        config_sources(
            Value::Null,
            json!({
                "options": {
                    "docnav-markdown": {"max_heading_level": 9}
                }
            }),
        ),
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("invalid user config remains blocking");
    assert_source_diagnostic(
        error.diagnostic(),
        "options.docnav-markdown.max_heading_level",
        "range_invalid",
        "user",
        "user/docnav.json",
    );
}

#[test]
fn hard_cutover_preserves_field_declaration_order_for_primary_diagnostic() {
    let error = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "defaults": {
                    "output": "invalid-output"
                },
                "options": {
                    "docnav-markdown": {
                        "max_heading_level": 9
                    }
                }
            }),
            Value::Null,
        ),
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("mixed invalid common and native option fields");
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error
            .details()
            .get("field")
            .and_then(Value::as_str),
        Some("defaults.output")
    );
}

fn assert_source_diagnostic(
    diagnostic: &docnav_diagnostics::DiagnosticRecordDraft,
    field: &str,
    reason: &str,
    source_level: &str,
    path: &str,
) {
    let protocol_error = super::protocol_error(diagnostic);
    assert_eq!(
        protocol_error
            .details()
            .get("field")
            .and_then(Value::as_str),
        Some(field)
    );
    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some(reason)
    );
    let issue = protocol_error
        .details()
        .get("config_issues")
        .and_then(Value::as_array)
        .and_then(|issues| issues.first())
        .expect("source-scoped config issue");
    assert_eq!(
        issue.get("source_level").and_then(Value::as_str),
        Some(source_level)
    );
    assert_eq!(issue.get("path").and_then(Value::as_str), Some(path));
    assert_eq!(
        issue.get("reason_code").and_then(Value::as_str),
        Some(reason)
    );
}
