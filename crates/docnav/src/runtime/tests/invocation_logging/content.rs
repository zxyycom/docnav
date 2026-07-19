use std::fs;

use docnav_adapter_contracts::{
    Adapter, AdapterDefinition, AdapterError, AdapterResult, FindInput, InfoInput, OutlineInput,
    ReadInput,
};
use docnav_markdown::{markdown_adapter_definition, MarkdownAdapter};
use docnav_navigation::execute_navigation_command;
use docnav_protocol::{FindResult, InfoResult, OutlineResult, ProbeResult, ReadResult};

use super::super::super::{
    navigation_command, output_mode, AdapterRuntime, DocnavRuntime, DocumentRequest,
};
use super::super::support::*;
use crate::invocation_log::{DocumentInvocationLog, InvocationLogger};
use crate::output::{outcome_for_response, CommandOutcome};
use crate::parameter_catalog::document_parameter_catalog;
use crate::project_paths::normalize_document_path;
use crate::registry::AdapterRegistry;

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
fn invocation_auto_read_content_capture_reuses_root_event_and_hash_shape() {
    let body = "# Secret\n\nHidden secret content.\n";
    let (_workspace, project_root) = markdown_project("invocation-auto-read-capture", body);
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("outline.jsonl");
    let capture_root = project_root.join("capture");
    let mut command = outline_command(None, None);
    command.invocation_log = Some(".log/outline.jsonl".to_owned());
    command.invocation_log_content_root = Some("capture".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    let output = write_protocol_json(outcome);
    let log_text = fs::read_to_string(&log_path).unwrap();
    let events = read_jsonl_events(&log_path);
    let completed = event_named(&events, "operation_completed");
    let captured = event_named(&events, "content_captured");
    let nested_content = output["result"]["auto_read"]["read"]["content"]
        .as_str()
        .expect("unique outline ref should produce auto-read content");
    let hash = completed["result"]["content"]["content_hash"]
        .as_str()
        .unwrap();
    let relative_path = captured["relative_path"].as_str().unwrap();
    let content_file = capture_root.join(relative_path.replace('/', std::path::MAIN_SEPARATOR_STR));
    let captured_bytes = fs::read(&content_file).unwrap();

    assert_eq!(output["operation"], "outline");
    assert_eq!(
        events
            .iter()
            .filter(|event| event["event"] == "operation_completed")
            .count(),
        1
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event["event"] == "content_captured")
            .count(),
        1
    );
    assert!(events.iter().all(|event| event["operation"] == "outline"));
    assert_eq!(captured["content"]["content_hash"], hash);
    assert!(relative_path.ends_with(&format!("sha256-{hash}.content")));
    assert_eq!(test_sha256_hex(&captured_bytes), hash);
    assert_eq!(String::from_utf8(captured_bytes).unwrap(), nested_content);
    assert!(!log_text.contains("Hidden secret content."));
}

#[test]
fn invocation_find_auto_read_logs_root_metadata_without_capture_file() {
    let body = "# Secret\n\nHidden secret content.\n";
    let (_workspace, project_root) = markdown_project("invocation-find-auto-read-log", body);
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("find.jsonl");
    let capture_root = project_root.join("capture");
    let mut command = outline_command(None, None);
    command.operation = docnav_protocol::Operation::Find;
    command.query = Some("Hidden".to_owned());
    command.invocation_log = Some(".log/find.jsonl".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    let output = write_protocol_json(outcome);
    let log_text = fs::read_to_string(&log_path).unwrap();
    let events = read_jsonl_events(&log_path);
    let completed = event_named(&events, "operation_completed");
    let nested_content = output["result"]["auto_read"]["read"]["content"]
        .as_str()
        .expect("unique find ref should produce auto-read content");
    let content = &completed["result"]["content"];

    assert_eq!(output["operation"], "find");
    assert_eq!(events.len(), 1);
    assert_eq!(completed["operation"], "find");
    assert_eq!(
        content["content_hash"],
        test_sha256_hex(nested_content.as_bytes())
    );
    assert!(!log_text.contains("Hidden secret content."));
    assert!(
        !capture_root.exists(),
        "capture disabled should not create content files"
    );
}

#[test]
fn invocation_failed_auto_read_keeps_only_the_successful_root_event() {
    let body = "# Secret\n\nNested body must stay private.\n";
    let (_workspace, project_root) = markdown_project("invocation-auto-read-failure", body);
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("outline.jsonl");
    let capture_root = project_root.join("capture");
    let mut command = outline_command(None, None);
    command.invocation_log = Some(".log/outline.jsonl".to_owned());
    command.invocation_log_content_root = Some("capture".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = execute_document_with_nested_read_failure(request);
    let output = write_protocol_json(outcome);
    let log_text = fs::read_to_string(&log_path).unwrap();
    let events = read_jsonl_events(&log_path);
    let completed = event_named(&events, "operation_completed");

    assert_eq!(output["ok"], true);
    assert_eq!(output["operation"], "outline");
    assert_eq!(output["result"]["entries"].as_array().unwrap().len(), 1);
    assert!(output["result"].get("auto_read").is_none());
    assert_eq!(events.len(), 1);
    assert_eq!(completed["operation"], "outline");
    assert!(!log_text.contains("nested-read-private-diagnostic"));
    assert!(!log_text.contains("Nested body must stay private."));
    assert!(
        !capture_root.exists(),
        "failed nested read should not create a content capture"
    );
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

fn execute_document_with_nested_read_failure(request: DocumentRequest) -> CommandOutcome {
    let logger = InvocationLogger::from_command(
        &request.command,
        &request.project,
        &request.project_config,
        &request.user_config,
    );
    let document = normalize_document_path(&request.project, &request.command.path).unwrap();
    let log_context = logger.document_context(
        &request.command,
        &request.project,
        Some(&document.absolute_path),
    );
    let catalog = document_parameter_catalog().unwrap();
    let registry = AdapterRegistry {
        adapters: NESTED_READ_FAILURE_ADAPTERS,
    };
    let outcome = execute_navigation_command(
        navigation_command(&request.command, document.adapter_path),
        request.config_source_descriptors,
        &catalog,
        &registry,
    )
    .unwrap();
    let output = output_mode(outcome.output);
    let invocation_log = DocumentInvocationLog::new(logger, log_context, request.started);

    outcome_for_response(outcome, output, Some(invocation_log)).unwrap()
}

static NESTED_READ_FAILURE_ADAPTER: NestedReadFailureAdapter = NestedReadFailureAdapter;
static NESTED_READ_FAILURE_ADAPTERS: &[fn() -> AdapterDefinition<'static>] =
    &[nested_read_failure_adapter_definition];

struct NestedReadFailureAdapter;

impl Adapter for NestedReadFailureAdapter {
    fn probe(&self, path: &str) -> ProbeResult {
        MarkdownAdapter.probe(path)
    }

    fn outline(&self, input: &OutlineInput) -> AdapterResult<OutlineResult> {
        MarkdownAdapter.outline(input)
    }

    fn read(&self, _input: &ReadInput) -> AdapterResult<ReadResult> {
        Err(AdapterError::internal("nested-read-private-diagnostic"))
    }

    fn find(&self, input: &FindInput) -> AdapterResult<FindResult> {
        MarkdownAdapter.find(input)
    }

    fn info(&self, input: &InfoInput) -> AdapterResult<InfoResult> {
        MarkdownAdapter.info(input)
    }
}

fn nested_read_failure_adapter_definition() -> AdapterDefinition<'static> {
    AdapterDefinition::new(
        markdown_adapter_definition().manifest().clone(),
        &NESTED_READ_FAILURE_ADAPTER,
        None,
    )
    .unwrap()
}
