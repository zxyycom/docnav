use std::fs;

use docnav_protocol::{OperationResult, ProtocolDiagnosticCode, ProtocolResponse};
use serde_json::{json, Value};

use crate::{
    execute_loaded_navigation_command, execute_navigation_command,
    NavigationConfigSourceDescriptor, NavigationConfigSourceDescriptors,
};

use super::super::support::{
    config_sources, navigation_command, temp_workspace_path, write_config_file, StubRegistry,
};

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
    let result = result.as_structured().expect("structured outline result");
    assert_eq!(result.entries[0].label, "Max 2");
    let _ = fs::remove_dir_all(workspace);
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
        let protocol_error = super::protocol_error(error.diagnostic());

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
        assert_eq!(
            protocol_error
                .details()
                .get("reason")
                .and_then(Value::as_str),
            Some("invalid_config_object")
        );
        assert_eq!(
            protocol_error
                .details()
                .get("config_issues")
                .and_then(Value::as_array)
                .and_then(|issues| issues.first())
                .and_then(|issue| issue.get("field"))
                .and_then(Value::as_str),
            Some(field)
        );
    }
}
