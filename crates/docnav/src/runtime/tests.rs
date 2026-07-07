use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use docnav_protocol::{try_positive, Operation};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::*;
use crate::cli::{DocumentCommand, NativeOptionCliInput};
use crate::config::{load_context_for_project, ConfigContext, CoreConfig};
use crate::output::{write_error, write_outcome, ErrorOutput};
use crate::project_context::{SelectedConfigPath, SelectedConfigPaths};

#[test]
fn linked_adapter_uses_absolute_document_path_from_project_subdir() {
    let workspace = temp_workspace("absolute-linked-path");
    let project_root = workspace.path().join("project");
    let docnav_dir = project_root.join(".docnav");
    let docs_dir = project_root.join("docs");
    fs::create_dir_all(&docnav_dir).unwrap();
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(docs_dir.join("expected.md"), "# Expected\n").unwrap();

    let context = ConfigContext {
        project: project_context(project_root.clone(), docnav_dir.clone()),
        project_config: CoreConfig::default(),
        user_config: CoreConfig::default(),
    };
    let command = DocumentCommand {
        operation: Operation::Outline,
        path: "../docs/expected.md".to_owned(),
        ref_id: None,
        query: None,
        page: None,
        pagination_enabled: None,
        limit: None,
        native_options: Vec::new(),
        output: Some(OutputMode::ProtocolJson),
        adapter: None,
        invocation_log: None,
        invocation_log_content_root: None,
        config_paths: Default::default(),
    };
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    let output = write_protocol_json(outcome);

    assert_eq!(output["ok"], true);
    assert_eq!(first_entry_label(&output), Some("Expected"));
    let document_path = output["result"]["document_path"].as_str();
    assert!(
        document_path.is_none(),
        "protocol output should not leak internal path shape: {output}"
    );
}

#[test]
fn core_linked_markdown_consumes_project_native_max_heading_level() {
    let (_workspace, project_root) = markdown_project(
        "linked-native-options",
        "# One\n\n## Two\n\n### Three\n\n#### Four\n",
    );

    let context = default_context(project_root);
    write_config_file(
        context.project.project_config_path(),
        json!({
            "options": {
                "max_heading_level": 2
            }
        }),
    );
    let command = outline_command(None, None);
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    let output = write_protocol_json(outcome);
    let labels = entry_labels(&output);

    assert_eq!(output["ok"], true);
    assert_eq!(labels, vec!["One", "Two"]);
}

#[test]
fn core_linked_markdown_delegates_native_option_range_to_adapter() {
    let (_workspace, project_root) =
        markdown_project("linked-native-options-invalid", "# One\n\n## Two\n");
    let context = default_context(project_root);
    let command = outline_command(Some(7), None);
    let request = DocumentRequest::from_config_context(command, context);

    let (exit_code, output) = write_document_result(
        AdapterRuntime.execute_document(request),
        Operation::Outline,
        OutputMode::ProtocolJson,
    );

    assert_eq!(exit_code, 2);
    assert_eq!(output["ok"], false);
    assert_eq!(output["error"]["code"], "INVALID_REQUEST");
    assert_eq!(
        output["error"]["details"]["field"],
        "arguments.options.max_heading_level"
    );
    assert_eq!(output["error"]["details"]["reason"], "range_invalid");
    assert_eq!(
        output["error"]["details"]["option_issues"][0]["reason_code"],
        "range_invalid"
    );
    assert_eq!(
        output["error"]["details"]["option_issues"][0]["source"],
        "explicit"
    );
}

#[test]
fn core_linked_markdown_reports_project_native_option_source() {
    assert_invalid_native_option_source(
        "project-native-option-source",
        Some(json!("wide")),
        None,
        "type_mismatch",
        "project",
    );
}

#[test]
fn core_linked_markdown_reports_user_native_option_source() {
    assert_invalid_native_option_source(
        "user-native-option-source",
        None,
        Some(json!(9)),
        "range_invalid",
        "user",
    );
}

#[test]
fn config_path_context_reports_automatic_discovery_adapter_source() {
    let (_workspace, project_root) =
        markdown_project("config-path-context-automatic-discovery-source", "# One\n");
    let context = default_context(project_root);
    let output = AdapterRuntime
        .describe_document_context("docs/guide.md".to_owned(), None, &context.project)
        .unwrap();

    assert_eq!(output.adapter.selected.as_deref(), Some("docnav-markdown"));
    assert_eq!(output.adapter.source, "automatic_discovery");
    assert_ne!(output.adapter.source, "inferred");
    assert_eq!(output.defaults.output.value, json!("readable-view"));
    assert_eq!(output.defaults.output.source, "built_in");
}

fn assert_invalid_native_option_source(
    workspace_name: &str,
    project_option: Option<Value>,
    user_option: Option<Value>,
    reason: &str,
    source: &str,
) {
    let (_workspace, project_root) = markdown_project(workspace_name, "# One\n\n## Two\n");
    let context = default_context(project_root);
    if let Some(value) = project_option {
        write_native_option_config(context.project.project_config_path(), value);
    }
    if let Some(value) = user_option {
        write_native_option_config(context.project.user_config_path(), value);
    }
    let command = outline_command(None, None);
    let request = DocumentRequest::from_config_context(command, context);

    let (exit_code, output) = write_document_result(
        AdapterRuntime.execute_document(request),
        Operation::Outline,
        OutputMode::ProtocolJson,
    );

    assert_eq!(exit_code, 2);
    assert_eq!(output["error"]["details"]["reason"], reason);
    assert_eq!(
        output["error"]["details"]["option_issues"][0]["source"],
        source
    );
}

#[test]
fn missing_adapter_routing_precedes_invalid_native_option() {
    let (_workspace, project_root) = markdown_project("missing-adapter-before-options", "# One\n");
    let context = default_context(project_root);
    let command = outline_command(Some(9), Some("missing-adapter"));
    let request = DocumentRequest::from_config_context(command, context);

    let error = match AdapterRuntime.execute_document(request) {
        Ok(_) => panic!("missing adapter should fail before options"),
        Err(error) => error,
    };
    let record = error
        .diagnostic()
        .clone()
        .into_record()
        .expect("diagnostic should be valid");
    let protocol_error = docnav_protocol::ProtocolError::from_diagnostic_record(&record).unwrap();

    assert_eq!(
        protocol_error.code(),
        docnav_protocol::ProtocolDiagnosticCode::AdapterUnavailable
    );
    assert_eq!(protocol_error.owner(), "docnav_navigation_routing");
    assert_eq!(
        protocol_error
            .details()
            .get("adapter_id")
            .and_then(Value::as_str),
        Some("missing-adapter")
    );
}

// @case WB-CORE-INVOCATION-LOG-001
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

    assert_eq!(details["field"], "config");
    assert_eq!(details["reason"], "invalid_config_value");
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
    command.output = Some(OutputMode::ReadableJson);
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
    command.output = Some(OutputMode::ReadableView);
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

fn write_protocol_json(outcome: CommandOutcome) -> Value {
    let (exit_code, output) = write_protocol_json_with_exit(outcome);
    assert_eq!(exit_code, 0);
    output
}

fn markdown_project(name: &str, content: &str) -> (TempWorkspace, PathBuf) {
    let workspace = temp_workspace(name);
    let project_root = workspace.path().join("project");
    let docs_dir = project_root.join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(docs_dir.join("guide.md"), content).unwrap();
    (workspace, project_root)
}

fn write_native_option_config(path: &Path, value: Value) {
    write_config_file(
        path,
        json!({
            "options": {
                "max_heading_level": value
            }
        }),
    );
}

fn write_config_file(path: &Path, value: Value) {
    fs::create_dir_all(path.parent().expect("config parent")).unwrap();
    fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

fn default_context(project_root: PathBuf) -> ConfigContext {
    ConfigContext {
        project: project_context(project_root.clone(), project_root),
        project_config: CoreConfig::default(),
        user_config: CoreConfig::default(),
    }
}

fn outline_command(max_heading_level: Option<u32>, adapter: Option<&str>) -> DocumentCommand {
    DocumentCommand {
        operation: Operation::Outline,
        path: "docs/guide.md".to_owned(),
        ref_id: None,
        query: None,
        page: None,
        pagination_enabled: None,
        limit: Some(try_positive(80).unwrap()),
        native_options: max_heading_level
            .map(|value| {
                vec![NativeOptionCliInput {
                    flag: "--max-heading-level".to_owned(),
                    value: value.to_string(),
                }]
            })
            .unwrap_or_default(),
        output: Some(OutputMode::ProtocolJson),
        adapter: adapter.map(str::to_owned),
        invocation_log: None,
        invocation_log_content_root: None,
        config_paths: Default::default(),
    }
}

fn read_command(ref_id: &str) -> DocumentCommand {
    DocumentCommand {
        operation: Operation::Read,
        path: "docs/guide.md".to_owned(),
        ref_id: Some(ref_id.to_owned()),
        query: None,
        page: None,
        pagination_enabled: None,
        limit: Some(try_positive(80).unwrap()),
        native_options: Vec::new(),
        output: Some(OutputMode::ProtocolJson),
        adapter: None,
        invocation_log: None,
        invocation_log_content_root: None,
        config_paths: Default::default(),
    }
}

fn read_jsonl_events(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn event_named<'a>(events: &'a [Value], event: &str) -> &'a Value {
    events
        .iter()
        .find(|value| value["event"] == event)
        .unwrap_or_else(|| panic!("missing event {event}: {events:#?}"))
}

fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn test_sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut text = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(text, "{byte:02x}");
    }
    text
}

fn write_protocol_json_with_exit(outcome: CommandOutcome) -> (i32, Value) {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit_code = write_outcome(outcome, &mut stdout, &mut stderr);
    assert!(
        stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&stderr)
    );
    (exit_code, serde_json::from_slice(&stdout).unwrap())
}

fn write_outcome_text_with_exit(outcome: CommandOutcome) -> (i32, String, String) {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit_code = write_outcome(outcome, &mut stdout, &mut stderr);
    (
        exit_code,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

fn parse_single_json_value(stdout: &str) -> Value {
    let mut values = serde_json::Deserializer::from_str(stdout).into_iter::<Value>();
    let value = values
        .next()
        .expect("stdout should contain one JSON value")
        .expect("stdout JSON should parse");
    assert!(
        values.next().is_none(),
        "stdout should contain a single JSON value: {stdout}"
    );
    value
}

fn assert_no_invocation_event_text(stdout: &str) {
    for forbidden in [
        "operation_completed",
        "operation_failed",
        "content_captured",
        "content_capture_failed",
        "correlation_id",
        "\"event\"",
    ] {
        assert!(
            !stdout.contains(forbidden),
            "stdout should not contain invocation log text {forbidden:?}: {stdout}"
        );
    }
}

fn write_document_result(
    result: AppResult<CommandOutcome>,
    operation: Operation,
    output_mode: OutputMode,
) -> (i32, Value) {
    match result {
        Ok(outcome) => write_protocol_json_with_exit(outcome),
        Err(error) => {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let exit_code = write_error(ErrorOutput {
                error: &error,
                output_mode,
                operation: Some(operation),
                stdout: &mut stdout,
                stderr: &mut stderr,
            });
            assert!(
                stderr.is_empty(),
                "stderr: {}",
                String::from_utf8_lossy(&stderr)
            );
            (exit_code, serde_json::from_slice(&stdout).unwrap())
        }
    }
}

fn first_entry_label(output: &Value) -> Option<&str> {
    output["result"]["entries"]
        .as_array()
        .and_then(|entries| entries.first())
        .and_then(|entry| entry["label"].as_str())
}

fn entry_labels(output: &Value) -> Vec<&str> {
    output["result"]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["label"].as_str().unwrap())
        .collect()
}

fn project_context(project_root: PathBuf, cwd: PathBuf) -> ProjectContext {
    ProjectContext {
        cwd,
        config_paths: SelectedConfigPaths {
            project: SelectedConfigPath::default(project_root.join(".docnav").join("docnav.json")),
            user: SelectedConfigPath::default(
                project_root.join(".docnav-user").join("docnav.json"),
            ),
        },
        project_root,
    }
}

struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn temp_workspace(name: &str) -> TempWorkspace {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir()
        .join("docnav-runtime-tests")
        .join(format!("{name}-{suffix}"));
    fs::create_dir_all(&path).unwrap();
    TempWorkspace { path }
}

struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("stdout closed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct LogAbsentWriter<'a> {
    log_path: &'a Path,
    bytes: Vec<u8>,
}

impl<'a> LogAbsentWriter<'a> {
    fn new(log_path: &'a Path) -> Self {
        Self {
            log_path,
            bytes: Vec::new(),
        }
    }

    fn into_string(self) -> String {
        String::from_utf8(self.bytes).unwrap()
    }
}

impl Write for LogAbsentWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        assert!(
            !self.log_path.exists(),
            "output failure log must be written after fallback output error projection"
        );
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
