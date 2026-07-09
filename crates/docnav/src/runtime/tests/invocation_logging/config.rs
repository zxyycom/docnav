use std::{fs, io};

use serde_json::{json, Value};

use super::super::super::{AdapterRuntime, DocnavRuntime, DocumentRequest};
use super::super::support::*;
use crate::config::load_context_for_project;

#[test]
fn invocation_logging_disabled_creates_no_log_side_effect() {
    let (_workspace, project_root) = markdown_project("invocation-disabled", "# One\n");
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("invocation.jsonl");
    write_config_file(
        context.project.project_config_path(),
        json!({
            "invocation_log": {
                "enabled": false,
                "path": ".log/invocation.jsonl"
            }
        }),
    );
    let request = DocumentRequest::from_config_context(outline_command(None, None), context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    let output = write_protocol_json(outcome);

    assert_eq!(output["ok"], true);
    assert!(!log_path.exists(), "disabled logging created {log_path:?}");
}

#[test]
fn invocation_logging_config_enabled_uses_validated_core_config() {
    let (_workspace, project_root) = markdown_project("invocation-config-enabled", "# One\n");
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("config.jsonl");
    write_config_file(
        context.project.project_config_path(),
        json!({
            "invocation_log": {
                "enabled": true,
                "path": ".log/config.jsonl"
            }
        }),
    );
    let context = load_context_for_project(context.project).unwrap();
    let request = DocumentRequest::from_config_context(outline_command(None, None), context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    let output = write_protocol_json(outcome);
    let events = read_jsonl_events(&log_path);

    assert_eq!(output["ok"], true);
    assert_eq!(
        event_named(&events, "operation_completed")["event"],
        "operation_completed"
    );
}

#[test]
fn invocation_cli_content_root_without_cli_log_does_not_override_config_log() {
    let body = "# Secret\n\nHidden secret content.\n";
    let (_workspace, project_root) = markdown_project("invocation-cli-root-with-config-log", body);
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("config.jsonl");
    let capture_root = project_root.join("capture");
    write_config_file(
        context.project.project_config_path(),
        json!({
            "invocation_log": {
                "enabled": true,
                "path": ".log/config.jsonl"
            }
        }),
    );
    let context = load_context_for_project(context.project).unwrap();
    let mut command = read_command("H:L1:H1");
    command.invocation_log_content_root = Some("capture".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    let output = write_protocol_json(outcome);
    let events = read_jsonl_events(&log_path);

    assert_eq!(output["ok"], true);
    assert_eq!(
        event_named(&events, "operation_completed")["event"],
        "operation_completed"
    );
    assert!(
        events
            .iter()
            .all(|event| event["event"] != "content_captured"),
        "CLI content root without CLI log should not capture content: {events:#?}"
    );
    assert!(
        !capture_root.exists(),
        "CLI content root without CLI log created {capture_root:?}"
    );
}

#[test]
fn invocation_log_config_type_error_is_blocking_core_config_error() {
    let (_workspace, project_root) = markdown_project("invocation-config-invalid", "# One\n");
    let context = default_context(project_root.clone());
    write_config_file(
        context.project.project_config_path(),
        json!({
            "invocation_log": {
                "enabled": "true",
                "path": ".log/config.jsonl"
            }
        }),
    );

    let error = load_context_for_project(context.project).unwrap_err();
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["field"], "invocation_log.enabled");
    assert_eq!(details["reason"], "type_mismatch");
    assert_eq!(
        details["config_issues"][0]["source_level"],
        Value::String("project".to_owned())
    );
}

#[test]
fn invocation_cli_log_records_config_load_failure_before_runtime_config() {
    let workspace = temp_workspace("invocation-config-load-failure");
    let project_config = workspace.path().join("broken-project.json");
    let user_config = workspace.path().join("user.json");
    let doc_path = workspace.path().join("guide.md");
    let log_path = workspace.path().join("invocation.jsonl");
    fs::write(&project_config, "{not-json").unwrap();
    fs::write(&user_config, "{}").unwrap();
    fs::write(&doc_path, "# One\n").unwrap();
    let args = vec![
        "outline".to_owned(),
        "--project-config".to_owned(),
        project_config.display().to_string(),
        "--user-config".to_owned(),
        user_config.display().to_string(),
        "--output".to_owned(),
        "protocol-json".to_owned(),
        "--invocation-log".to_owned(),
        log_path.display().to_string(),
        doc_path.display().to_string(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit_code = crate::run(args, io::empty(), &mut stdout, &mut stderr);
    assert!(
        log_path.exists(),
        "expected pre-start log at {log_path:?}; exit={exit_code}; stdout={}; stderr={}",
        String::from_utf8_lossy(&stdout),
        String::from_utf8_lossy(&stderr)
    );
    let events = read_jsonl_events(&log_path);

    assert_ne!(exit_code, 0);
    assert!(
        stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&stderr)
    );
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event"], "operation_failed");
    assert_eq!(events[0]["failure"]["layer"], "config");
    assert!(
        events
            .iter()
            .all(|event| event["event"] != "operation_completed"),
        "config failure must not log completion: {events:#?}"
    );
}
