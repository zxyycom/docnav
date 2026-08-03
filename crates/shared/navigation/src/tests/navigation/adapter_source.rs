use docnav_protocol::ProtocolDiagnosticCode;
use serde_json::Value;

use crate::{execute_loaded_navigation_command, NavigationFailureLayer};

use super::super::support::{
    cli_value_candidate, config_sources, navigation_command, StubRegistry,
};

#[test]
fn explicit_missing_adapter_reports_exact_lookup_diagnostic() {
    let command = navigation_command(vec![cli_value_candidate(
        "docnav.defaults.adapter",
        "--adapter",
        Value::String("custom-local-adapter".to_owned()),
    )]);

    let error = execute_loaded_navigation_command(
        command,
        config_sources(Value::Null, Value::Null),
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("missing adapter");
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error.code(),
        ProtocolDiagnosticCode::AdapterUnavailable
    );
    assert_eq!(protocol_error.owner(), "docnav_navigation_routing");
    let expected_details = serde_json::from_value(serde_json::json!({
        "adapter_id": "custom-local-adapter",
        "reason": "ADAPTER_NOT_FOUND",
        "selection_source": "explicit",
        "stage": "resolve"
    }))
    .unwrap();
    assert_eq!(protocol_error.details(), &expected_details);
}

#[test]
fn explicit_missing_adapter_error_carries_invocation_failure_layer() {
    let command = navigation_command(vec![cli_value_candidate(
        "docnav.defaults.adapter",
        "--adapter",
        Value::String("custom-local-adapter".to_owned()),
    )]);

    let error = execute_loaded_navigation_command(
        command,
        config_sources(Value::Null, Value::Null),
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("missing adapter");

    assert_eq!(
        error.failure_layer(),
        Some(NavigationFailureLayer::AdapterSelection)
    );
    assert_eq!(error.selected_adapter_id(), None);
    assert_eq!(error.request_id(), None);
}
