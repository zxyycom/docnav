use super::super::super::{AdapterRuntime, DocnavRuntime, DocumentRequest};
use super::super::support::*;

#[test]
fn invocation_failure_logs_bounded_layer_code_and_summary() {
    let (_workspace, project_root) = markdown_project("invocation-failure", "# One\n");
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("failure.jsonl");
    let mut command = outline_command(None, Some("missing-adapter"));
    command.invocation_log = Some(".log/failure.jsonl".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    let error = match AdapterRuntime.execute_document(request) {
        Ok(_) => panic!("missing adapter should fail"),
        Err(error) => error,
    };
    let events = read_jsonl_events(&log_path);
    let failure = &events[0]["failure"];

    assert_eq!(error.exit_code().code(), 4);
    assert_eq!(events[0]["event"], "operation_failed");
    assert_eq!(failure["layer"], "adapter_selection");
    assert!(failure["code"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert!(failure["summary"]
        .as_str()
        .is_some_and(|value| !value.is_empty() && value.len() <= 512));
}

#[test]
fn invocation_linked_handler_structured_diagnostic_logs_adapter_dispatch_failure() {
    let (_workspace, project_root) = markdown_project("invocation-handler-diagnostic", "# One\n");
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("handler-failure.jsonl");
    let mut command = read_command("H:L99:H1");
    command.invocation_log = Some(".log/handler-failure.jsonl".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    let (exit_code, stdout, stderr) = write_outcome_text_with_exit(outcome);
    let output = parse_single_json_value(&stdout);
    let events = read_jsonl_events(&log_path);
    let failure = &event_named(&events, "operation_failed")["failure"];

    assert_eq!(exit_code, 3);
    assert_eq!(stderr, "");
    assert_eq!(output["ok"], false);
    assert_eq!(output["operation"], "read");
    assert_eq!(output["error"]["code"], "REF_NOT_FOUND");
    assert_eq!(output["error"]["details"]["ref"], "H:L99:H1");
    assert_no_invocation_event_text(&stdout);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["adapter_id"], "docnav-markdown");
    assert_eq!(events[0]["status"], "failure");
    assert_eq!(failure["layer"], "adapter_dispatch");
    assert!(failure["code"]
        .as_str()
        .is_some_and(|value| !value.is_empty() && value.len() <= 128));
    assert!(failure["summary"]
        .as_str()
        .is_some_and(|value| !value.is_empty() && value.len() <= 512));
}
