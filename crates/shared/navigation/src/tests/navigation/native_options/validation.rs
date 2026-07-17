use docnav_protocol::ProtocolDiagnosticCode;
use serde_json::{json, Value};

use crate::execute_loaded_navigation_command;

use super::super::super::support::{
    cli_invalid_candidate, cli_value_candidate, config_sources, navigation_command,
    MultiAdapterRegistry, StubRegistry,
};
use super::super::{first_option_issue_source, protocol_error};

#[test]
fn navigation_reports_unknown_adapter_id_under_options_as_config_diagnostic() {
    let error = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "options": {
                    "docnav-unknown": {
                        "max_heading_level": 2
                    }
                }
            }),
            Value::Null,
        ),
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("unknown adapter id should fail config validation");
    let protocol_error = protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error.code(),
        ProtocolDiagnosticCode::InvalidRequest
    );
    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some("unknown_adapter_id")
    );
    assert_eq!(
        protocol_error
            .details()
            .get("field")
            .and_then(Value::as_str),
        Some("options.docnav-unknown")
    );
    assert_eq!(
        protocol_error
            .details()
            .get("config_issues")
            .and_then(Value::as_array)
            .and_then(|issues| issues.first())
            .and_then(|issue| issue.get("field"))
            .and_then(Value::as_str),
        Some("options.docnav-unknown")
    );
}

#[test]
fn navigation_reports_explicit_native_option_type_failure() {
    let error = execute_loaded_navigation_command(
        navigation_command(vec![cli_invalid_candidate(
            "docnav.adapters.docnav-markdown.options.max_heading_level",
            "--max-heading-level",
            json!("wide"),
            "expected an integer",
        )]),
        config_sources(Value::Null, Value::Null),
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("type invalid");
    let protocol_error = protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some("type_mismatch")
    );
    assert_eq!(first_option_issue_source(&protocol_error), Some("explicit"));
}

#[test]
fn navigation_blocks_dispatch_when_native_option_type_cannot_materialize() {
    // StubAdapter returns a successful outline if invoked, so this full-command error proves
    // materialization stops before dispatch.
    let error = execute_loaded_navigation_command(
        navigation_command(vec![cli_value_candidate(
            "docnav.adapters.docnav-markdown.options.max_heading_level",
            "--max-heading-level",
            json!("wide"),
        )]),
        config_sources(Value::Null, Value::Null),
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("type invalid");
    let protocol_error = protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error.code(),
        ProtocolDiagnosticCode::InvalidRequest
    );
    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some("type_mismatch")
    );
}

#[test]
fn navigation_reports_typed_native_option_failure_with_source() {
    let error = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
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
    .expect_err("range invalid");
    let protocol_error = protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error.code(),
        ProtocolDiagnosticCode::InvalidRequest
    );
    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some("range_invalid")
    );
    assert_eq!(
        protocol_error
            .details()
            .get("field")
            .and_then(Value::as_str),
        Some("options.docnav-markdown.max_heading_level")
    );
    assert_eq!(
        protocol_error
            .details()
            .get("config_issues")
            .and_then(Value::as_array)
            .and_then(|issues| issues.first())
            .and_then(|issue| issue.get("source_level"))
            .and_then(Value::as_str),
        Some("project")
    );
}

#[test]
fn navigation_blocks_invalid_catalog_value_for_other_known_adapter() {
    let error = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "options": {
                    "docnav-other": {
                        "max_heading_level": 9
                    }
                }
            }),
            Value::Null,
        ),
        &crate::tests::support::document_parameter_catalog(),
        &MultiAdapterRegistry,
    )
    .expect_err("known other-adapter catalog values still require full validation");
    let protocol_error = protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error.code(),
        ProtocolDiagnosticCode::InvalidRequest
    );
    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some("range_invalid")
    );
    assert_eq!(
        protocol_error
            .details()
            .get("field")
            .and_then(Value::as_str),
        Some("options.docnav-other.max_heading_level")
    );
    assert_eq!(
        protocol_error
            .details()
            .get("config_issues")
            .and_then(Value::as_array)
            .and_then(|issues| issues.first())
            .and_then(|issue| issue.get("source_level"))
            .and_then(Value::as_str),
        Some("project")
    );
}

#[test]
fn navigation_reports_explicit_range_failure_with_adapter_compatible_diagnostic() {
    let error = execute_loaded_navigation_command(
        navigation_command(vec![cli_value_candidate(
            "docnav.adapters.docnav-markdown.options.max_heading_level",
            "--max-heading-level",
            json!(7),
        )]),
        config_sources(Value::Null, Value::Null),
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("range invalid");
    let protocol_error = protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error.code(),
        ProtocolDiagnosticCode::InvalidRequest
    );
    assert_eq!(protocol_error.owner(), "adapter_options");
    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some("range_invalid")
    );
    assert_eq!(
        protocol_error
            .details()
            .get("field")
            .and_then(Value::as_str),
        Some("arguments.options.max_heading_level")
    );
    assert_eq!(
        protocol_error.expected(),
        Some(&json!("integer in range 1..6"))
    );
    assert_eq!(protocol_error.received(), Some(&json!("7")));
}

#[test]
fn navigation_rejects_unselected_explicit_candidate_after_adapter_routing() {
    let error = execute_loaded_navigation_command(
        navigation_command(vec![cli_value_candidate(
            "docnav.adapters.docnav-other.options.payload",
            "--other-payload",
            json!("known-other-adapter-value"),
        )]),
        config_sources(Value::Null, Value::Null),
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("unsupported native option");
    let protocol_error = protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some("unsupported")
    );
    assert_eq!(first_option_issue_source(&protocol_error), Some("explicit"));
}

#[test]
fn navigation_rejects_unknown_config_option_after_adapter_routing() {
    let error = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "options": {
                    "docnav-markdown": {
                        "missing_option": true
                    }
                }
            }),
            Value::Null,
        ),
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("unsupported native option");
    let protocol_error = protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error
            .details()
            .get("field")
            .and_then(Value::as_str),
        Some("arguments.options.missing_option")
    );
    assert_eq!(first_option_issue_source(&protocol_error), Some("project"));
    assert_eq!(
        protocol_error
            .details()
            .get("config_issues")
            .and_then(Value::as_array)
            .and_then(|issues| issues.first())
            .and_then(|issue| issue.get("field"))
            .and_then(Value::as_str),
        Some("options.docnav-markdown.missing_option")
    );
}

#[test]
fn navigation_rejects_config_option_not_applicable_to_operation() {
    let mut command = navigation_command(Vec::new());
    command.operation = docnav_protocol::Operation::Read;
    command.ref_id = Some("stub:1".to_owned());

    let error = execute_loaded_navigation_command(
        command,
        config_sources(
            json!({
                "options": {
                    "docnav-markdown": {
                        "max_heading_level": 2
                    }
                }
            }),
            Value::Null,
        ),
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("operation-inapplicable native option");
    let protocol_error = protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error.code(),
        ProtocolDiagnosticCode::InvalidRequest
    );
    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some("unsupported")
    );
    assert_eq!(first_option_issue_source(&protocol_error), Some("project"));
}
