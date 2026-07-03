use std::fs;

use docnav_protocol::{OperationResult, ProtocolDiagnosticCode, ProtocolResponse};
use serde_json::{json, Value};

use crate::{
    execute_loaded_navigation_command, execute_navigation_command,
    NavigationConfigSourceDescriptor, NavigationConfigSourceDescriptors,
    NavigationNativeOptionInput, NavigationOutputMode,
};

use super::support::{
    config_sources, diagnostic_record, navigation_command, temp_workspace_path, write_config_file,
    StubRegistry, UnsupportedRegistry,
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
    let record = diagnostic_record(error.diagnostic());
    let protocol_error = docnav_protocol::ProtocolError::from_diagnostic_record(&record)
        .expect("protocol projection");

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
fn automatic_discovery_all_fail_projects_candidate_failures() {
    let error = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(Value::Null, Value::Null),
        &UnsupportedRegistry,
    )
    .expect_err("all adapter candidates should fail");
    let protocol_error = protocol_error(error.diagnostic());

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

#[test]
fn navigation_loads_project_and_user_config_sources_from_descriptors() {
    let workspace = temp_workspace_path("navigation-owned-config-loading");
    let project_config_path = workspace.join("project").join("docnav.json");
    let user_config_path = workspace.join("user").join("docnav.json");
    write_config_file(
        &project_config_path,
        json!({
            "options": {
                "max_heading_level": 2
            }
        }),
    );
    write_config_file(
        &user_config_path,
        json!({
            "options": {
                "max_heading_level": 1
            }
        }),
    );

    let outcome = execute_navigation_command(
        navigation_command(Vec::new()),
        NavigationConfigSourceDescriptors {
            project: NavigationConfigSourceDescriptor::default(project_config_path),
            user: NavigationConfigSourceDescriptor::default(user_config_path),
        },
        &StubRegistry,
    )
    .expect("navigation outcome");

    let ProtocolResponse::Success(success) = outcome.response else {
        panic!("expected success");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };
    assert_eq!(result.entries[0].label, "Max 2");
    let _ = fs::remove_dir_all(workspace);
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
                    "max_heading_level": 9
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
    assert_eq!(first_option_issue_source(&protocol_error), Some("project"));
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
                    "missing_option": true
                }
            }),
            Value::Null,
        ),
        &StubRegistry,
    )
    .expect_err("unsupported native option");
    let protocol_error = protocol_error(error.diagnostic());

    assert_eq!(first_option_issue_source(&protocol_error), Some("project"));
}

#[test]
fn navigation_rejects_nested_non_object_config_shapes() {
    for (field, project_config) in [
        ("defaults", json!({"defaults": false})),
        (
            "defaults.pagination",
            json!({"defaults": {"pagination": false}}),
        ),
        ("options", json!({"options": false})),
    ] {
        let error = execute_loaded_navigation_command(
            navigation_command(Vec::new()),
            config_sources(project_config, Value::Null),
            &StubRegistry,
        )
        .expect_err("nested non-object should fail");
        let protocol_error = protocol_error(error.diagnostic());

        assert_eq!(
            protocol_error.code(),
            ProtocolDiagnosticCode::InvalidRequest
        );
        assert_eq!(
            protocol_error
                .details()
                .get("field")
                .and_then(Value::as_str),
            Some(field)
        );
    }
}

fn protocol_error(
    diagnostic: &docnav_diagnostics::DiagnosticRecordDraft,
) -> docnav_protocol::ProtocolError {
    let record = diagnostic_record(diagnostic);
    docnav_protocol::ProtocolError::from_diagnostic_record(&record).expect("protocol projection")
}

fn first_option_issue_source(error: &docnav_protocol::ProtocolError) -> Option<&str> {
    error
        .details()
        .get("option_issues")
        .and_then(Value::as_array)
        .and_then(|issues| issues.first())
        .and_then(|issue| issue.get("source"))
        .and_then(Value::as_str)
}
