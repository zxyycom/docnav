use docnav_protocol::ProtocolDiagnosticCode;
use serde_json::Value;

use crate::{execute_loaded_navigation_command, NavigationFailureLayer};

use super::super::support::{
    config_sources, navigation_command, StubRegistry, UnsupportedRegistry,
};

#[test]
// @case WB-NAV-ADAPTER-SOURCE-001
fn explicit_missing_adapter_reports_static_registry_guidance() {
    let mut command = navigation_command(Vec::new());
    command.adapter = Some("custom-local-adapter".to_owned());

    let error = execute_loaded_navigation_command(
        command,
        config_sources(Value::Null, Value::Null),
        &StubRegistry,
    )
    .expect_err("missing adapter");
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error.code(),
        ProtocolDiagnosticCode::AdapterUnavailable
    );
    assert_eq!(protocol_error.owner(), "docnav_navigation_routing");
    let guidance = protocol_error
        .guidance()
        .and_then(|items| items.first())
        .expect("default guidance");
    assert!(
        guidance.contains("current core release static registry"),
        "guidance should describe static registry: {guidance}"
    );
    for removed_term in ["install", "register", "executable", "artifact"] {
        assert!(
            !guidance.contains(removed_term),
            "guidance should not mention {removed_term}: {guidance}"
        );
    }
}

#[test]
fn explicit_missing_adapter_error_carries_invocation_failure_layer() {
    let mut command = navigation_command(Vec::new());
    command.adapter = Some("custom-local-adapter".to_owned());

    let error = execute_loaded_navigation_command(
        command,
        config_sources(Value::Null, Value::Null),
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

#[test]
fn automatic_discovery_all_fail_projects_candidate_failures() {
    let error = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(Value::Null, Value::Null),
        &UnsupportedRegistry,
    )
    .expect_err("all adapter candidates should fail");
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_eq!(protocol_error.code(), ProtocolDiagnosticCode::FormatUnknown);
    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some("NO_SUPPORTED_ADAPTER")
    );
    assert_eq!(
        protocol_error
            .details()
            .get("candidate_failures")
            .and_then(Value::as_array)
            .and_then(|failures| failures.first())
            .and_then(|failure| failure.get("reason"))
            .and_then(Value::as_str),
        Some("PROBE_UNSUPPORTED")
    );
}
