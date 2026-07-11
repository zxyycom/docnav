use docnav_protocol::ProtocolDiagnosticCode;
use serde_json::{json, Value};

use crate::{execute_loaded_navigation_command, NavigationNativeOptionInput};

use super::super::super::support::{config_sources, navigation_command, StubRegistry};
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
        navigation_command(vec![NavigationNativeOptionInput {
            flag: "--max-heading-level".to_owned(),
            value: "wide".to_owned(),
        }]),
        config_sources(Value::Null, Value::Null),
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
fn navigation_rejects_unknown_explicit_native_option_after_adapter_routing() {
    let error = execute_loaded_navigation_command(
        navigation_command(vec![NavigationNativeOptionInput {
            flag: "--missing-option".to_owned(),
            value: "true".to_owned(),
        }]),
        config_sources(Value::Null, Value::Null),
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
