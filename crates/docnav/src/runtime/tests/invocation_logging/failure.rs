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
    assert_eq!(failure["code"], "ADAPTER_UNAVAILABLE");
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
    assert_eq!(failure["code"], "REF_NOT_FOUND");
    assert!(failure["summary"]
        .as_str()
        .is_some_and(|value| !value.is_empty() && value.len() <= 512));
}

#[test]
fn invocation_preselection_failure_uses_cwd_resolved_document_summary() {
    let (_workspace, project_root) = markdown_project("invocation-cwd-summary", "# One\n");
    let nested = project_root.join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    let mut context = default_context(project_root.clone());
    context.project.cwd = nested.clone();
    let log_path = project_root.join(".log").join("cwd-failure.jsonl");
    let mut command = outline_command(None, Some("missing-adapter"));
    command.invocation_log = Some(".log/cwd-failure.jsonl".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    assert!(
        AdapterRuntime.execute_document(request).is_err(),
        "missing adapter should fail before document selection"
    );
    let events = read_jsonl_events(&log_path);
    let document = &events[0]["document"];
    let expected_path = nested.join("docs/guide.md");

    assert_eq!(document["path_display"], "nested/docs/guide.md");
    assert_eq!(document["path_kind"], "project_relative");
    assert_eq!(
        document["path_hash"],
        test_sha256_hex(crate::project_paths::path_to_slash(&expected_path).as_bytes())
    );

    let outside_log_path = project_root.join(".log").join("cwd-outside-failure.jsonl");
    let mut outside_context = default_context(project_root.clone());
    outside_context.project.cwd = nested;
    let mut outside_command = outline_command(None, Some("missing-adapter"));
    outside_command.path = "../../outside.md".to_owned();
    outside_command.invocation_log = Some(".log/cwd-outside-failure.jsonl".to_owned());

    assert!(
        AdapterRuntime
            .execute_document(DocumentRequest::from_config_context(
                outside_command,
                outside_context,
            ))
            .is_err(),
        "missing adapter should fail before accessing the outside document"
    );
    let outside_events = read_jsonl_events(&outside_log_path);
    let outside_document = &outside_events[0]["document"];
    let outside_path = project_root.parent().unwrap().join("outside.md");
    let outside_display = crate::project_paths::path_to_slash(&outside_path);

    assert_eq!(outside_document["path_display"], outside_display);
    assert_eq!(outside_document["path_kind"], "absolute");
    assert_eq!(
        outside_document["path_hash"],
        test_sha256_hex(outside_display.as_bytes())
    );
}
