use std::fs;

use cli_config_resolution::{FieldIdentity, SourceCandidate, SourceLocator};
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
fn core_linked_json_supports_automatic_and_declared_selection_and_reports_selected_content_failure()
{
    let workspace = temp_workspace("linked-json-selection");
    let project_root = workspace.path().join("project");
    let docs_dir = project_root.join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(
        docs_dir.join("settings.json"),
        "{\"first\":1,\"second\":2}\n",
    )
    .unwrap();
    fs::write(docs_dir.join("fallback.md"), "# Markdown fallback\n").unwrap();
    let fallback_path = docs_dir.join("fallback.md").to_string_lossy().into_owned();

    let context = default_context(project_root);

    let automatic = AdapterRuntime
        .execute_document(DocumentRequest::from_config_context(
            json_outline_command("docs/settings.json", None, 1, 80),
            context.clone(),
        ))
        .expect("automatic discovery should select the linked JSON adapter");
    let automatic = write_protocol_json(automatic);

    assert_eq!(automatic["ok"], true);
    assert_eq!(automatic["operation"], "outline");
    assert_eq!(automatic["result"]["entries"].as_array().unwrap().len(), 2);
    assert_eq!(automatic["result"]["entries"][0]["ref"], "json:#/first");
    assert_eq!(automatic["result"]["entries"][0]["kind"], "number");
    assert_eq!(automatic["result"]["page"], Value::Null);

    let declared = AdapterRuntime
        .execute_document(DocumentRequest::from_config_context(
            json_read_command("docs/settings.json", "json:#/second", Some("docnav-json")),
            context.clone(),
        ))
        .expect("declared selection should dispatch the linked JSON read strategy");
    let declared = write_protocol_json(declared);

    assert_eq!(declared["ok"], true);
    assert_eq!(declared["operation"], "read");
    assert_eq!(declared["result"]["ref"], "json:#/second");
    assert_eq!(declared["result"]["content"], "2");
    assert_eq!(declared["result"]["content_type"], "application/json");

    let selected_failure = AdapterRuntime
        .execute_document(DocumentRequest::from_config_context(
            json_outline_command("docs/fallback.md", Some("docnav-json"), 1, 80),
            context,
        ))
        .expect("selected strategy diagnostic should reach document output");
    let (exit_code, selected_failure) = write_protocol_json_with_exit(selected_failure);

    assert_eq!(exit_code, 3);
    assert_eq!(selected_failure["ok"], false);
    assert_eq!(selected_failure["operation"], "outline");
    assert_eq!(
        selected_failure["error"]["code"],
        "DOCUMENT_CONTENT_INVALID"
    );
    assert_eq!(selected_failure["error"]["owner"], "adapter");
    assert_eq!(
        selected_failure["error"]["details"],
        serde_json::json!({
            "path": fallback_path,
            "reason": "JSON_SYNTAX_INVALID",
        })
    );
}

#[test]
fn selected_json_uses_only_common_closed_inputs_and_excludes_markdown_native_option() {
    let workspace = temp_workspace("linked-json-closed-inputs");
    let project_root = workspace.path().join("project");
    let docs_dir = project_root.join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(
        docs_dir.join("settings.json"),
        "{\"first\":1,\"second\":2}\n",
    )
    .unwrap();

    let context = default_context(project_root);
    write_config_file(
        context.project.project_config_path(),
        json!({
            "options": {
                "docnav-markdown": {
                    "max_heading_level": 1
                }
            }
        }),
    );

    let output = AdapterRuntime
        .execute_document(DocumentRequest::from_config_context(
            json_outline_command("docs/settings.json", Some("docnav-json"), 2, 1),
            context,
        ))
        .expect("selected JSON should dispatch with common inputs only");
    let output = write_protocol_json(output);

    assert_eq!(output["ok"], true);
    assert_eq!(output["operation"], "outline");
    assert_eq!(output["result"]["entries"].as_array().unwrap().len(), 1);
    assert_eq!(output["result"]["entries"][0]["ref"], "json:#/second");
    assert_eq!(output["result"]["entries"][0]["kind"], "number");
    assert_eq!(output["result"]["page"], Value::Null);
}

#[test]
fn core_linked_markdown_reports_project_and_user_native_option_sources() {
    for (workspace_name, project_option, user_option, reason, source) in [
        (
            "project-native-option-source",
            Some(json!("wide")),
            None,
            "type_mismatch",
            "project",
        ),
        (
            "user-native-option-source",
            None,
            Some(json!(9)),
            "range_invalid",
            "user",
        ),
    ] {
        assert_invalid_native_option_source(
            workspace_name,
            project_option,
            user_option,
            reason,
            source,
        );
    }
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

fn json_outline_command(
    path: &str,
    adapter: Option<&str>,
    page: u32,
    limit: u32,
) -> DocumentCommand {
    let mut candidates = vec![
        cli_value_candidate("docnav.document.page", "--page", json!(page)),
        cli_value_candidate("docnav.defaults.pagination.limit", "--limit", json!(limit)),
        cli_value_candidate(
            "docnav.defaults.auto_read",
            "--auto-read",
            json!("disabled"),
        ),
        cli_value_candidate("docnav.defaults.output", "--output", json!("protocol-json")),
    ];
    if let Some(adapter) = adapter {
        candidates.push(cli_value_candidate(
            "docnav.defaults.adapter",
            "--adapter",
            json!(adapter),
        ));
    }
    DocumentCommand {
        operation: Operation::Outline,
        path: path.to_owned(),
        ref_id: None,
        query: None,
        cli_source: cli_source(candidates),
        invocation_log: None,
        invocation_log_content_root: None,
        config_paths: Default::default(),
    }
}

fn json_read_command(path: &str, ref_id: &str, adapter: Option<&str>) -> DocumentCommand {
    let mut candidates = vec![
        cli_value_candidate("docnav.defaults.pagination.limit", "--limit", json!(80)),
        cli_value_candidate("docnav.defaults.output", "--output", json!("protocol-json")),
    ];
    if let Some(adapter) = adapter {
        candidates.push(cli_value_candidate(
            "docnav.defaults.adapter",
            "--adapter",
            json!(adapter),
        ));
    }
    DocumentCommand {
        operation: Operation::Read,
        path: path.to_owned(),
        ref_id: Some(ref_id.to_owned()),
        query: None,
        cli_source: cli_source(candidates),
        invocation_log: None,
        invocation_log_content_root: None,
        config_paths: Default::default(),
    }
}

fn cli_value_candidate(identity: &str, flag: &str, value: Value) -> SourceCandidate {
    SourceCandidate::value(
        FieldIdentity::new(identity).unwrap(),
        SourceLocator::CliFlag(flag.to_owned()),
        value,
    )
}
