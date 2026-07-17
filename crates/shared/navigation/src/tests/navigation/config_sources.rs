use std::fs;
use std::path::Path;

use docnav_protocol::{OperationResult, ProtocolDiagnosticCode, ProtocolError, ProtocolResponse};
use serde_json::{json, Value};

use crate::{
    execute_loaded_navigation_command, execute_navigation_command,
    NavigationConfigSourceDescriptor, NavigationConfigSourceDescriptors,
    NavigationConfigSourceOrigin,
};

use super::super::support::{
    cli_value_candidate, config_sources, navigation_command, temp_workspace_path,
    write_config_file, StubRegistry,
};

// @case WB-NAVIGATION-DISPATCH-001
#[test]
fn navigation_loads_project_and_user_config_sources_from_descriptors() {
    let workspace = temp_workspace_path("navigation-owned-config-loading");
    let project_config_path = workspace.join("project").join("docnav.json");
    let user_config_path = workspace.join("user").join("docnav.json");
    write_config_file(
        &project_config_path,
        json!({
            "options": {
                "docnav-markdown": {
                    "max_heading_level": 2
                }
            }
        }),
    );
    write_config_file(
        &user_config_path,
        json!({
            "options": {
                "docnav-markdown": {
                    "max_heading_level": 1
                }
            }
        }),
    );

    let outcome = execute_navigation_command(
        navigation_command(Vec::new()),
        NavigationConfigSourceDescriptors {
            project: NavigationConfigSourceDescriptor::default(project_config_path),
            user: NavigationConfigSourceDescriptor::default(user_config_path),
        },
        &crate::tests::support::document_parameter_catalog(),
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

// @case WB-NAVIGATION-CONFIG-SOURCES-002
#[test]
fn default_missing_config_sources_are_absent_without_diagnostics() {
    let workspace = temp_workspace_path("navigation-default-missing-config-loading");
    let project_config_path = workspace.join("project").join("missing-docnav.json");
    let user_config_path = workspace.join("user").join("missing-docnav.json");

    let outcome = execute_navigation_command(
        navigation_command(Vec::new()),
        NavigationConfigSourceDescriptors {
            project: NavigationConfigSourceDescriptor::default(project_config_path),
            user: NavigationConfigSourceDescriptor::default(user_config_path),
        },
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect("missing default config sources should be absent");

    assert_outline_label(outcome.response, "Max 3");
    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn explicit_missing_config_source_is_blocking_diagnostic() {
    let workspace = temp_workspace_path("navigation-explicit-missing-config-loading");
    let project_config_path = workspace.join("project").join("missing-docnav.json");
    let user_config_path = workspace.join("user").join("missing-docnav.json");

    let error = execute_navigation_command(
        navigation_command(Vec::new()),
        NavigationConfigSourceDescriptors {
            project: NavigationConfigSourceDescriptor::explicit_cli(project_config_path.clone()),
            user: NavigationConfigSourceDescriptor::default(user_config_path),
        },
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("missing explicit config source should fail");
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_config_issue(
        &protocol_error,
        ExpectedConfigIssue::new(
            "project",
            "explicit_cli",
            &project_config_path,
            "missing_explicit_cli",
            None,
        ),
    );
    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn override_missing_config_source_preserves_override_diagnostic() {
    let workspace = temp_workspace_path("navigation-override-missing-config-loading");
    let project_config_path = workspace.join("project").join("missing-docnav.json");
    let user_config_path = workspace.join("user").join("missing-docnav.json");

    let error = execute_navigation_command(
        navigation_command(Vec::new()),
        NavigationConfigSourceDescriptors {
            project: NavigationConfigSourceDescriptor::new(
                NavigationConfigSourceOrigin::Override,
                project_config_path.clone(),
            ),
            user: NavigationConfigSourceDescriptor::default(user_config_path),
        },
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("missing override config source should fail");
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_config_issue(
        &protocol_error,
        ExpectedConfigIssue::new(
            "project",
            "override",
            &project_config_path,
            "missing_override",
            None,
        ),
    );
    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn explicit_invalid_json_config_source_is_blocking_diagnostic() {
    let workspace = temp_workspace_path("navigation-explicit-invalid-config-loading");
    let invalid_json_path = workspace.join("project").join("invalid-docnav.json");
    let default_missing_path = workspace.join("default").join("missing-docnav.json");
    write_raw_config_file(&invalid_json_path, "{invalid");

    let error = execute_navigation_command(
        navigation_command(Vec::new()),
        NavigationConfigSourceDescriptors {
            project: NavigationConfigSourceDescriptor::explicit_cli(invalid_json_path.clone()),
            user: NavigationConfigSourceDescriptor::default(default_missing_path),
        },
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("invalid JSON explicit config source should fail");
    let protocol_error = super::protocol_error(error.diagnostic());
    assert_config_issue(
        &protocol_error,
        ExpectedConfigIssue::new(
            "project",
            "explicit_cli",
            &invalid_json_path,
            "invalid_json",
            None,
        ),
    );
    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn explicit_config_field_diagnostics_preserve_selected_source_path() {
    let workspace = temp_workspace_path("navigation-explicit-field-diagnostic");
    let project_config_path = workspace.join("project").join("docnav.json");
    let user_config_path = workspace.join("user").join("missing-docnav.json");
    write_config_file(&project_config_path, json!({"defaults": {"limit": 20}}));

    let error = execute_navigation_command(
        navigation_command(Vec::new()),
        NavigationConfigSourceDescriptors {
            project: NavigationConfigSourceDescriptor::explicit_cli(project_config_path.clone()),
            user: NavigationConfigSourceDescriptor::default(user_config_path),
        },
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("invalid config field should fail");
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_config_issue(
        &protocol_error,
        ExpectedConfigIssue::new(
            "project",
            "explicit_cli",
            &project_config_path,
            "unknown_config_field",
            Some("defaults.limit"),
        ),
    );
    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn explicit_config_value_diagnostics_preserve_selected_source_path() {
    let workspace = temp_workspace_path("navigation-explicit-value-diagnostic");
    let project_config_path = workspace.join("project").join("docnav.json");
    let user_config_path = workspace.join("user").join("missing-docnav.json");
    write_config_file(
        &project_config_path,
        json!({
            "options": {
                "docnav-markdown": {
                    "max_heading_level": 9
                }
            }
        }),
    );

    let error = execute_navigation_command(
        navigation_command(Vec::new()),
        NavigationConfigSourceDescriptors {
            project: NavigationConfigSourceDescriptor::explicit_cli(project_config_path.clone()),
            user: NavigationConfigSourceDescriptor::default(user_config_path),
        },
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("invalid config value should fail");
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some("range_invalid")
    );
    assert_config_issue_payload(
        &protocol_error,
        ExpectedConfigIssue::new(
            "project",
            "explicit_cli",
            &project_config_path,
            "range_invalid",
            Some("options.docnav-markdown.max_heading_level"),
        ),
    );
    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn explicit_default_config_value_diagnostics_preserve_selected_source_path() {
    let workspace = temp_workspace_path("navigation-explicit-ordinary-value-diagnostic");
    let project_config_path = workspace.join("project").join("docnav.json");
    let default_missing_path = workspace.join("default").join("missing-docnav.json");
    write_config_file(
        &project_config_path,
        json!({
            "defaults": {
                "pagination": {
                    "limit": 0
                }
            }
        }),
    );

    let error = execute_navigation_command(
        navigation_command(Vec::new()),
        NavigationConfigSourceDescriptors {
            project: NavigationConfigSourceDescriptor::explicit_cli(project_config_path.clone()),
            user: NavigationConfigSourceDescriptor::default(default_missing_path),
        },
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("invalid project config limit should fail");
    let protocol_error = super::protocol_error(error.diagnostic());
    assert_config_issue(
        &protocol_error,
        ExpectedConfigIssue::new(
            "project",
            "explicit_cli",
            &project_config_path,
            "range_invalid",
            Some("defaults.pagination.limit"),
        ),
    );
    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn explicit_config_path_selection_preserves_parameter_priority() {
    let workspace = temp_workspace_path("navigation-explicit-config-priority");
    let project_config_path = workspace.join("project").join("docnav.json");
    let user_config_path = workspace.join("user").join("docnav.json");
    write_config_file(
        &project_config_path,
        json!({
            "options": {
                "docnav-markdown": {
                    "max_heading_level": 2
                }
            }
        }),
    );
    write_config_file(
        &user_config_path,
        json!({
            "options": {
                "docnav-markdown": {
                    "max_heading_level": 1
                }
            }
        }),
    );

    let explicit_outcome = execute_navigation_command(
        navigation_command(vec![cli_value_candidate(
            "docnav.adapters.docnav-markdown.options.max_heading_level",
            "--max-heading-level",
            json!(4),
        )]),
        NavigationConfigSourceDescriptors {
            project: NavigationConfigSourceDescriptor::explicit_cli(project_config_path.clone()),
            user: NavigationConfigSourceDescriptor::explicit_cli(user_config_path.clone()),
        },
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect("direct argv should preserve highest priority");
    assert_outline_label(explicit_outcome.response, "Max 4");

    let project_outcome = execute_navigation_command(
        navigation_command(Vec::new()),
        NavigationConfigSourceDescriptors {
            project: NavigationConfigSourceDescriptor::new(
                NavigationConfigSourceOrigin::ExplicitCli,
                project_config_path,
            ),
            user: NavigationConfigSourceDescriptor::explicit_cli(user_config_path),
        },
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect("project config should outrank user config");
    assert_outline_label(project_outcome.response, "Max 2");

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
            &crate::tests::support::document_parameter_catalog(),
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

fn write_raw_config_file(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().expect("config parent")).unwrap();
    fs::write(path, content).unwrap();
}

fn assert_outline_label(response: ProtocolResponse, expected: &str) {
    let ProtocolResponse::Success(success) = response else {
        panic!("expected success");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };
    let result = result.as_structured().expect("structured outline result");
    assert_eq!(result.entries[0].label, expected);
}

#[derive(Clone, Copy)]
struct ExpectedConfigIssue<'a> {
    source_level: &'a str,
    path_origin: &'a str,
    path: &'a Path,
    reason_code: &'a str,
    field: Option<&'a str>,
}

impl<'a> ExpectedConfigIssue<'a> {
    fn new(
        source_level: &'a str,
        path_origin: &'a str,
        path: &'a Path,
        reason_code: &'a str,
        field: Option<&'a str>,
    ) -> Self {
        Self {
            source_level,
            path_origin,
            path,
            reason_code,
            field,
        }
    }
}

fn assert_config_issue(error: &ProtocolError, expected: ExpectedConfigIssue<'_>) {
    let expected_path = expected.path.display().to_string();
    assert_eq!(error.code(), ProtocolDiagnosticCode::InvalidRequest);
    assert_eq!(
        error.details().get("path").and_then(Value::as_str),
        Some(expected_path.as_str())
    );
    assert_eq!(
        error.details().get("reason").and_then(Value::as_str),
        Some(expected.reason_code)
    );
    assert_config_issue_payload(error, expected);
}

fn assert_config_issue_payload(error: &ProtocolError, expected: ExpectedConfigIssue<'_>) {
    let expected_path = expected.path.display().to_string();
    let issue = error
        .details()
        .get("config_issues")
        .and_then(Value::as_array)
        .and_then(|issues| issues.first())
        .expect("config issue");
    assert_eq!(
        issue.get("source_level").and_then(Value::as_str),
        Some(expected.source_level)
    );
    assert_eq!(
        issue.get("path_origin").and_then(Value::as_str),
        Some(expected.path_origin)
    );
    assert_eq!(
        issue.get("path").and_then(Value::as_str),
        Some(expected_path.as_str())
    );
    assert_eq!(
        issue.get("reason_code").and_then(Value::as_str),
        Some(expected.reason_code)
    );
    assert_eq!(issue.get("field").and_then(Value::as_str), expected.field);
}
