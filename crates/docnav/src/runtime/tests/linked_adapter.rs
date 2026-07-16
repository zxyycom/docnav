use std::fs;

use docnav_protocol::Operation;
use serde_json::{json, Value};

use super::super::{AdapterRuntime, DocnavRuntime, DocumentRequest};
use super::support::*;
use crate::cli::{DocumentCommand, OutputMode};
use crate::config::{ConfigContext, CoreConfig};

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
        cli_source: cli_source(vec![cli_config_resolution::SourceCandidate::value(
            cli_config_resolution::FieldIdentity::new("docnav.defaults.output").unwrap(),
            cli_config_resolution::SourceLocator::CliFlag("--output".to_owned()),
            json!("protocol-json"),
        )]),
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
                "docnav-markdown": {
                    "max_heading_level": 2
                }
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
        output["error"]["details"]["config_issues"][0]["source_level"],
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
