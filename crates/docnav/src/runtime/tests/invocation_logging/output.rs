use super::super::super::{AdapterRuntime, DocnavRuntime, DocumentRequest};
use super::super::support::*;
use crate::output::write_outcome;

#[test]
fn invocation_logging_enabled_success_writes_jsonl_with_request_id() {
    let (_workspace, project_root) = markdown_project("invocation-success", "# One\n");
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("invocation.jsonl");
    let mut command = outline_command(None, None);
    command.invocation_log = Some(".log/invocation.jsonl".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    assert!(
        !log_path.exists(),
        "success must not be logged before output is written"
    );
    let (exit_code, stdout) = write_protocol_json_with_exit(outcome);
    let events = read_jsonl_events(&log_path);

    assert_eq!(exit_code, 0);
    assert_eq!(stdout["ok"], true);
    assert!(
        !String::from_utf8(serde_json::to_vec(&stdout).unwrap())
            .unwrap()
            .contains("operation_completed"),
        "stdout should not contain invocation log events"
    );
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["schema_version"], "0.1");
    assert_eq!(events[0]["event"], "operation_completed");
    assert_eq!(events[0]["status"], "success");
    assert_eq!(events[0]["operation"], "outline");
    assert_eq!(events[0]["adapter_id"], "docnav-markdown");
    assert!(events[0]["request_id"].as_str().is_some());
}

#[test]
fn invocation_output_write_failure_logs_output_projection_without_completion() {
    let (_workspace, project_root) = markdown_project("invocation-output-write-failure", "# One\n");
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("output-failure.jsonl");
    let mut command = outline_command(None, None);
    command.invocation_log = Some(".log/output-failure.jsonl".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    assert!(
        !log_path.exists(),
        "completion must not be logged before output write"
    );
    let mut stdout = FailingWriter;
    let mut stderr = LogAbsentWriter::new(&log_path);
    let exit_code = write_outcome(outcome, &mut stdout, &mut stderr);
    let stderr = stderr.into_string();
    let events = read_jsonl_events(&log_path);

    assert_ne!(exit_code, 0);
    assert!(
        !stderr.is_empty(),
        "writer failure should still report output failure"
    );
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event"], "operation_failed");
    assert_eq!(events[0]["failure"]["layer"], "output_projection");
    assert!(
        events
            .iter()
            .all(|event| event["event"] != "operation_completed"),
        "output failure must not log completion: {events:#?}"
    );
}

#[test]
fn invocation_readable_json_stdout_stays_single_readable_value() {
    let (_workspace, project_root) = markdown_project("invocation-readable-json", "# One\n");
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("readable-json.jsonl");
    let mut command = outline_command(None, None);
    set_cli_value(
        &mut command,
        "docnav.defaults.output",
        "--output",
        serde_json::json!("readable-json"),
    );
    command.invocation_log = Some(".log/readable-json.jsonl".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    let (exit_code, stdout, stderr) = write_outcome_text_with_exit(outcome);
    let output = parse_single_json_value(&stdout);
    let events = read_jsonl_events(&log_path);

    assert_eq!(exit_code, 0);
    assert_eq!(stderr, "");
    assert_eq!(output["kind"], "structured");
    assert!(
        output.get("ok").is_none(),
        "readable-json leaked protocol envelope"
    );
    assert_no_invocation_event_text(&stdout);
    assert_eq!(
        event_named(&events, "operation_completed")["event"],
        "operation_completed"
    );
}

#[test]
fn invocation_readable_view_stdout_stays_free_of_log_events() {
    let (_workspace, project_root) = markdown_project("invocation-readable-view", "# One\n");
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("readable-view.jsonl");
    let mut command = outline_command(None, None);
    set_cli_value(
        &mut command,
        "docnav.defaults.output",
        "--output",
        serde_json::json!("readable-view"),
    );
    command.invocation_log = Some(".log/readable-view.jsonl".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    let (exit_code, stdout, stderr) = write_outcome_text_with_exit(outcome);
    let events = read_jsonl_events(&log_path);

    assert_eq!(exit_code, 0);
    assert_eq!(stderr, "");
    assert!(
        stdout.contains("One"),
        "readable-view did not render document content: {stdout}"
    );
    assert_no_invocation_event_text(&stdout);
    assert_eq!(
        event_named(&events, "operation_completed")["event"],
        "operation_completed"
    );
}
