use std::fs;

use super::super::super::{AdapterRuntime, DocnavRuntime, DocumentRequest};
use super::super::support::*;

#[test]
fn invocation_read_metadata_only_hashes_content_without_capture_file() {
    let body = "# Secret\n\nHidden secret content.\n";
    let (_workspace, project_root) = markdown_project("invocation-read-metadata", body);
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("read.jsonl");
    let capture_root = project_root.join("capture");
    let mut command = read_command("H:L1:H1");
    command.invocation_log = Some(".log/read.jsonl".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    let output = write_protocol_json(outcome);
    let log_text = fs::read_to_string(&log_path).unwrap();
    let events = read_jsonl_events(&log_path);
    let content = &events[0]["result"]["content"];

    assert_eq!(output["ok"], true);
    assert_eq!(content["hash_algorithm"], "sha256");
    assert!(is_lower_sha256(content["content_hash"].as_str().unwrap()));
    assert_eq!(content["content_type"], "text/markdown");
    assert!(content["size_bytes"].as_u64().unwrap() > 0);
    assert!(!log_text.contains("Hidden secret content."));
    assert!(
        !capture_root.exists(),
        "capture disabled should not create content files"
    );
}

#[test]
fn invocation_content_capture_writes_hash_named_file_and_event() {
    let body = "# Secret\n\nHidden secret content.\n";
    let (_workspace, project_root) = markdown_project("invocation-content-capture", body);
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("read.jsonl");
    let capture_root = project_root.join("capture");
    let mut command = read_command("H:L1:H1");
    command.invocation_log = Some(".log/read.jsonl".to_owned());
    command.invocation_log_content_root = Some("capture".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    let output = write_protocol_json(outcome);
    let events = read_jsonl_events(&log_path);
    let completed = event_named(&events, "operation_completed");
    let captured = event_named(&events, "content_captured");
    let hash = completed["result"]["content"]["content_hash"]
        .as_str()
        .unwrap();
    let relative_path = captured["relative_path"].as_str().unwrap();
    let content_file = capture_root.join(relative_path.replace('/', std::path::MAIN_SEPARATOR_STR));
    let captured_bytes = fs::read(&content_file).unwrap();

    assert_eq!(output["ok"], true);
    assert_eq!(captured["content"]["content_hash"], hash);
    assert!(relative_path.ends_with(&format!("sha256-{hash}.content")));
    assert_eq!(test_sha256_hex(&captured_bytes), hash);
    assert_eq!(String::from_utf8(captured_bytes).unwrap(), body);
}

#[test]
fn invocation_unwritable_log_path_does_not_change_operation_result() {
    let (_workspace, project_root) = markdown_project("invocation-unwritable-log", "# One\n");
    let context = default_context(project_root.clone());
    let log_dir = project_root.join("existing-dir");
    fs::create_dir_all(&log_dir).unwrap();
    let mut command = outline_command(None, None);
    command.invocation_log = Some("existing-dir".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    let output = write_protocol_json(outcome);

    assert_eq!(output["ok"], true);
}

#[test]
fn invocation_capture_failure_does_not_change_operation_result() {
    let body = "# Secret\n\nHidden secret content.\n";
    let (_workspace, project_root) = markdown_project("invocation-capture-failure", body);
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("read.jsonl");
    fs::write(project_root.join("capture-file"), "not a directory").unwrap();
    let mut command = read_command("H:L1:H1");
    command.invocation_log = Some(".log/read.jsonl".to_owned());
    command.invocation_log_content_root = Some("capture-file".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    let output = write_protocol_json(outcome);
    let events = read_jsonl_events(&log_path);

    assert_eq!(output["ok"], true);
    assert_eq!(
        event_named(&events, "content_capture_failed")["failure"]["layer"],
        "operation"
    );
}
