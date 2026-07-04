use docnav_protocol::{OperationResult, ProtocolDiagnosticCode, ProtocolResponse};
use serde_json::{json, Value};

use crate::{execute_loaded_navigation_command, NavigationNativeOptionInput, NavigationOutputMode};

use super::super::support::{
    config_sources, navigation_command, InvalidOptionRegistry, StubRegistry,
};

#[test]
// @case WB-NAV-INPUT-RESOLUTION-001
fn navigation_resolves_selected_adapter_options_and_dispatches() {
    let outcome = execute_loaded_navigation_command(
        navigation_command(vec![NavigationNativeOptionInput {
            flag: "--max-heading-level".to_owned(),
            value: "4".to_owned(),
        }]),
        config_sources(
            json!({
                "defaults": {
                    "pagination": { "limit": 120 },
                    "output": "protocol-json"
                },
                "options": {
                    "max_heading_level": 2
                }
            }),
            json!({
                "options": {
                    "max_heading_level": 1
                }
            }),
        ),
        &StubRegistry,
    )
    .expect("navigation outcome");

    assert_eq!(outcome.output, NavigationOutputMode::ProtocolJson);
    match outcome.response {
        ProtocolResponse::Success(success) => {
            assert_eq!(success.operation, docnav_protocol::Operation::Outline);
            assert!(success.ok);
        }
        ProtocolResponse::Failure(failure) => panic!("expected success, got {failure:?}"),
    }
}

#[test]
fn navigation_includes_adapter_native_option_default() {
    let outcome = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(Value::Null, Value::Null),
        &StubRegistry,
    )
    .expect("navigation outcome");

    let ProtocolResponse::Success(success) = outcome.response else {
        panic!("expected success");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };
    assert_eq!(result.entries[0].label, "Max 3");
}

#[test]
fn navigation_resolves_json_native_option_through_typed_fields() {
    let outcome = execute_loaded_navigation_command(
        navigation_command(vec![NavigationNativeOptionInput {
            flag: "--payload".to_owned(),
            value: "null".to_owned(),
        }]),
        config_sources(
            json!({
                "options": {
                    "payload": {"source": "project"}
                }
            }),
            Value::Null,
        ),
        &StubRegistry,
    )
    .expect("navigation outcome");

    let ProtocolResponse::Success(success) = outcome.response else {
        panic!("expected success");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };

    assert_eq!(result.entries[0].label, "Payload null");
}

#[test]
fn navigation_accepts_config_option_applicable_to_operation() {
    let outcome = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "options": {
                    "max_heading_level": 2
                }
            }),
            Value::Null,
        ),
        &StubRegistry,
    )
    .expect("applicable native option");

    let ProtocolResponse::Success(success) = outcome.response else {
        panic!("expected success");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };
    assert_eq!(result.entries[0].label, "Max 2");
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
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some("type_mismatch")
    );
    assert_eq!(
        super::first_option_issue_source(&protocol_error),
        Some("explicit")
    );
}

#[test]
fn navigation_reports_typed_native_option_failure_with_source() {
    let error = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "options": {
                    "max_heading_level": 9
                }
            }),
            Value::Null,
        ),
        &StubRegistry,
    )
    .expect_err("range invalid");
    let protocol_error = super::protocol_error(error.diagnostic());

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
        super::first_option_issue_source(&protocol_error),
        Some("project")
    );
    assert_eq!(
        protocol_error
            .details()
            .get("option_issues")
            .and_then(Value::as_array)
            .and_then(|issues| issues.first())
            .and_then(|issue| issue.get("expected"))
            .and_then(Value::as_str),
        Some("integer in range 1..6")
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
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some("unsupported")
    );
    assert_eq!(
        super::first_option_issue_source(&protocol_error),
        Some("explicit")
    );
}

#[test]
fn navigation_rejects_unknown_config_option_after_adapter_routing() {
    let error = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "options": {
                    "missing_option": true
                }
            }),
            Value::Null,
        ),
        &StubRegistry,
    )
    .expect_err("unsupported native option");
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_eq!(
        super::first_option_issue_source(&protocol_error),
        Some("project")
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
                    "max_heading_level": 2
                }
            }),
            Value::Null,
        ),
        &StubRegistry,
    )
    .expect_err("operation-inapplicable native option");
    let protocol_error = super::protocol_error(error.diagnostic());

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
    assert_eq!(
        super::first_option_issue_source(&protocol_error),
        Some("project")
    );
}

#[test]
fn navigation_maps_invalid_adapter_option_declaration_to_internal_error() {
    let error = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(Value::Null, Value::Null),
        &InvalidOptionRegistry,
    )
    .expect_err("invalid adapter option declaration");
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_eq!(protocol_error.code(), ProtocolDiagnosticCode::InternalError);
    assert_eq!(
        protocol_error
            .details()
            .get("error_id")
            .and_then(Value::as_str),
        Some(
            "adapter-option:adapter option docnav.adapters.invalid.options.bad_path declaration path must be options.<key>, got invalid.bad_path"
        )
    );
}
