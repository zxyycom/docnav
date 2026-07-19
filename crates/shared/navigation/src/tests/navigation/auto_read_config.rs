use cli_config_resolution::SourceCandidate;
use docnav_protocol::Operation;
use serde_json::json;

use crate::{parameters::resolve_operation_input, AutoReadMode, NavigationCommand};

use super::super::support::{
    cli_value_candidate, config_sources, document_parameter_catalog, navigation_command,
};

const AUTO_READ_IDENTITY: &str = "docnav.defaults.auto_read";
const SELECTED_ADAPTER_ID: &str = "docnav-markdown";

#[test]
fn auto_read_mode_resolves_with_cli_project_user_and_builtin_precedence() {
    let cases = [
        (
            command(Operation::Outline, Vec::new()),
            json!({}),
            json!({}),
            AutoReadMode::UniqueRef,
        ),
        (
            command(Operation::Find, Vec::new()),
            json!({"defaults": {"auto_read": "unique-ref"}}),
            json!({"defaults": {"auto_read": "disabled"}}),
            AutoReadMode::UniqueRef,
        ),
        (
            command(Operation::Outline, Vec::new()),
            json!({}),
            json!({"defaults": {"auto_read": "disabled"}}),
            AutoReadMode::Disabled,
        ),
        (
            command(
                Operation::Outline,
                vec![cli_value_candidate(
                    AUTO_READ_IDENTITY,
                    "--auto-read",
                    json!("disabled"),
                )],
            ),
            json!({"defaults": {"auto_read": "unique-ref"}}),
            json!({"defaults": {"auto_read": "disabled"}}),
            AutoReadMode::Disabled,
        ),
    ];

    for (command, project, user, expected) in cases {
        let resolved = resolve_operation_input(
            &command,
            &config_sources(project, user),
            SELECTED_ADAPTER_ID,
            &document_parameter_catalog(),
        )
        .expect("auto-read resolution");
        assert_eq!(resolved.auto_read, Some(expected));
    }
}

#[test]
fn read_and_info_recognize_valid_config_without_projecting_auto_read() {
    for operation in [Operation::Read, Operation::Info] {
        let resolved = resolve_operation_input(
            &command(operation, Vec::new()),
            &config_sources(json!({"defaults": {"auto_read": "disabled"}}), json!({})),
            SELECTED_ADAPTER_ID,
            &document_parameter_catalog(),
        )
        .expect("valid global config shape remains known");

        assert_eq!(resolved.auto_read, None);
        assert_eq!(resolved.standard_input.operation(), operation);
    }
}

#[test]
fn invalid_auto_read_config_is_attributed_to_its_source() {
    let error = resolve_operation_input(
        &command(Operation::Outline, Vec::new()),
        &config_sources(json!({"defaults": {"auto_read": "sometimes"}}), json!({})),
        SELECTED_ADAPTER_ID,
        &document_parameter_catalog(),
    )
    .expect_err("invalid project auto-read value must fail");
    let details = error.diagnostic().details().to_value();
    let issue = details["config_issues"][0]
        .as_object()
        .expect("config issue");

    assert_eq!(details["field"], "defaults.auto_read");
    assert_eq!(details["reason"], "enum_invalid");
    assert_eq!(issue["source_level"], "project");
    assert_eq!(issue["field"], "defaults.auto_read");
}

#[test]
fn invalid_auto_read_cli_value_reports_the_canonical_flag_and_tokens() {
    let error = resolve_operation_input(
        &command(
            Operation::Outline,
            vec![cli_value_candidate(
                AUTO_READ_IDENTITY,
                "--auto-read",
                json!("sometimes"),
            )],
        ),
        &config_sources(json!({}), json!({})),
        SELECTED_ADAPTER_ID,
        &document_parameter_catalog(),
    )
    .expect_err("invalid explicit auto-read value must fail");
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["field"], "--auto-read");
    assert_eq!(
        details["reason"],
        "invalid --auto-read: accepted values: disabled, unique-ref"
    );
}

fn command(operation: Operation, candidates: Vec<SourceCandidate>) -> NavigationCommand {
    let mut command = navigation_command(candidates);
    command.operation = operation;
    match operation {
        Operation::Read => command.ref_id = Some("stub:1".to_owned()),
        Operation::Find => command.query = Some("needle".to_owned()),
        Operation::Outline | Operation::Info => {}
    }
    command
}
