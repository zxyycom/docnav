use docnav_navigation::{execute_protocol_request, protocol_request, OperationInput};
use docnav_protocol::{Operation, PositiveInteger};
use serde::Serialize;
use serde_json::Value;

use crate::cli::{DocumentCommand, OutputMode};
use crate::config::{ConfigContext, ResolvedValue};
use crate::error::{AppError, AppResult};
use crate::output::{outcome_for_response, CommandOutcome};
use crate::project_context::ProjectContext;
use crate::project_paths::normalize_document_path;
use crate::registry::AdapterRegistry;
use crate::routing::{select_adapter, AdapterSelectionRequest};
use crate::standard_parameters::{
    resolve_core_document_parameters, resolve_registered_native_options,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentRequest {
    pub project: ProjectContext,
    pub operation: Operation,
    pub path: String,
    pub ref_id: Option<String>,
    pub query: Option<String>,
    pub page: Option<PositiveInteger>,
    pub limit: Option<PositiveInteger>,
    pub options: Option<docnav_protocol::Options>,
    pub output: OutputMode,
    pub adapter: Option<String>,
    pub defaults: ResolvedDocumentDefaults,
    source_command: DocumentCommand,
    source_context: ConfigContext,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ResolvedDocumentDefaults {
    pub adapter: ResolvedValue,
    pub pagination: Option<ResolvedPaginationDefaults>,
    pub output: ResolvedValue,
    pub page: Option<ResolvedValue>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ResolvedPaginationDefaults {
    pub enabled: ResolvedValue,
    pub limit: ResolvedValue,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DocumentContextOutput {
    pub path: String,
    pub operation: Option<Operation>,
    pub adapter: AdapterContextOutput,
    pub defaults: ResolvedDocumentDefaults,
    pub runtime_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AdapterContextOutput {
    pub selected: Option<String>,
    pub source: String,
    pub note: String,
}

pub trait DocnavRuntime {
    fn execute_document(&self, request: DocumentRequest) -> AppResult<CommandOutcome>;

    fn describe_document_context(
        &self,
        path: String,
        operation: Option<Operation>,
        defaults: ResolvedDocumentDefaults,
        context: &ConfigContext,
    ) -> AppResult<DocumentContextOutput>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AdapterRuntime;

impl DocnavRuntime for AdapterRuntime {
    fn execute_document(&self, request: DocumentRequest) -> AppResult<CommandOutcome> {
        let document = normalize_document_path(&request.project, &request.path)?;
        let registry = AdapterRegistry::load(&request.project)?;
        let preselected_adapter_id = request.adapter.as_deref();
        let selection = select_adapter(AdapterSelectionRequest {
            registry: &registry,
            document: &document,
            operation: request.operation,
            preselected_adapter_id,
            preselected_adapter_source: request.defaults.adapter.source.as_str(),
        })?;
        let native_options = registry.native_options_for(request.operation);
        let options = resolve_registered_native_options(
            &request.source_command,
            &request.source_context,
            &native_options,
        )?;
        let protocol_request = protocol_request(OperationInput {
            operation: request.operation,
            document_path: document.adapter_path.clone(),
            ref_id: request.ref_id.clone(),
            query: request.query.clone(),
            page: request.page,
            limit: request.limit,
            options,
        })
        .map_err(|error| AppError::invalid_request(error.field(), error.reason()))?;
        let response = execute_protocol_request(selection.record.adapter(), &protocol_request);

        outcome_for_response(response, request.output)
    }

    fn describe_document_context(
        &self,
        path: String,
        operation: Option<Operation>,
        defaults: ResolvedDocumentDefaults,
        context: &ConfigContext,
    ) -> AppResult<DocumentContextOutput> {
        let effective_operation = operation.unwrap_or(Operation::Outline);
        let document = normalize_document_path(&context.project, &path)?;
        let registry = AdapterRegistry::load(&context.project)?;
        let preselected_adapter_id = defaults.adapter.value.as_str();
        let selection = select_adapter(AdapterSelectionRequest {
            registry: &registry,
            document: &document,
            operation: effective_operation,
            preselected_adapter_id,
            preselected_adapter_source: defaults.adapter.source.as_str(),
        })?;

        Ok(DocumentContextOutput {
            path: document.adapter_path,
            operation: Some(effective_operation),
            adapter: AdapterContextOutput {
                selected: Some(selection.record.id().to_owned()),
                source: adapter_source(&defaults.adapter, &selection.evidence),
                note: "selected built-in adapter passed static metadata and support checks"
                    .to_owned(),
            },
            defaults,
            runtime_status: "static_adapter_registry_ready".to_owned(),
        })
    }
}

impl DocumentRequest {
    pub fn from_command(command: DocumentCommand, context: &ConfigContext) -> AppResult<Self> {
        let resolved = resolve_core_document_parameters(&command, context)?;

        Ok(Self {
            project: context.project.clone(),
            operation: command.operation,
            path: resolved.path,
            ref_id: resolved.ref_id,
            query: resolved.query,
            page: resolved.page,
            limit: resolved.limit,
            options: resolved.options,
            output: resolved.output,
            adapter: resolved.adapter,
            defaults: resolved.defaults,
            source_command: command,
            source_context: context.clone(),
        })
    }
}

pub fn resolve_context_defaults(
    path: String,
    operation: Option<Operation>,
    context: &ConfigContext,
) -> AppResult<(String, Option<Operation>, ResolvedDocumentDefaults)> {
    let command = DocumentCommand {
        operation: operation.unwrap_or(Operation::Outline),
        path: path.clone(),
        ref_id: None,
        query: None,
        page: None,
        pagination_enabled: None,
        limit: None,
        max_heading_level: None,
        output: None,
        adapter: None,
    };
    let defaults = resolve_core_document_parameters(&command, context)?.defaults;
    Ok((path, operation, defaults))
}

fn adapter_source(
    preselected: &ResolvedValue,
    evidence: &[crate::routing::CandidateEvidence],
) -> String {
    if preselected.value != Value::Null {
        preselected.source.clone()
    } else if evidence.is_empty() {
        "inferred".to_owned()
    } else {
        "registry".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use docnav_protocol::{try_positive, Operation};
    use serde_json::{json, Value};

    use super::*;
    use crate::cli::DocumentCommand;
    use crate::config::{ConfigContext, CoreConfig};
    use crate::output::write_outcome;

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
            max_heading_level: None,
            output: Some(OutputMode::ProtocolJson),
            adapter: None,
        };
        let request = DocumentRequest::from_command(command, &context).unwrap();

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

        let context = ConfigContext {
            project: project_context(project_root.clone(), project_root.clone()),
            project_config: serde_json::from_value(json!({
                "options": {
                    "max_heading_level": 2
                }
            }))
            .unwrap(),
            user_config: CoreConfig::default(),
        };
        let command = outline_command(None, None);
        let request = DocumentRequest::from_command(command, &context).unwrap();

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
        let request = DocumentRequest::from_command(command, &context).unwrap();

        let outcome = AdapterRuntime.execute_document(request).unwrap();
        let (exit_code, output) = write_protocol_json_with_exit(outcome);

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
            "direct"
        );
    }

    #[test]
    fn core_linked_markdown_reports_project_native_option_source() {
        assert_invalid_native_option_source(
            "project-native-option-source",
            Some(json!("wide")),
            None,
            "type_mismatch",
            "project_config",
        );
    }

    #[test]
    fn core_linked_markdown_reports_user_native_option_source() {
        assert_invalid_native_option_source(
            "user-native-option-source",
            None,
            Some(json!(9)),
            "range_invalid",
            "user_config",
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
        let context = ConfigContext {
            project: project_context(project_root.clone(), project_root),
            project_config: core_config_with_native_option(project_option),
            user_config: core_config_with_native_option(user_option),
        };
        let command = outline_command(None, None);
        let request = DocumentRequest::from_command(command, &context).unwrap();

        let outcome = AdapterRuntime.execute_document(request).unwrap();
        let (exit_code, output) = write_protocol_json_with_exit(outcome);

        assert_eq!(exit_code, 2);
        assert_eq!(output["error"]["details"]["reason"], reason);
        assert_eq!(
            output["error"]["details"]["option_issues"][0]["source"],
            source
        );
    }

    fn core_config_with_native_option(value: Option<Value>) -> CoreConfig {
        match value {
            Some(value) => serde_json::from_value(json!({
                "options": {
                    "max_heading_level": value
                }
            }))
            .unwrap(),
            None => CoreConfig::default(),
        }
    }

    #[test]
    fn missing_adapter_selection_precedes_invalid_native_option() {
        let (_workspace, project_root) =
            markdown_project("missing-adapter-before-options", "# One\n");
        let context = default_context(project_root);
        let command = outline_command(Some(9), Some("missing-adapter"));
        let request = DocumentRequest::from_command(command, &context).unwrap();

        let error = match AdapterRuntime.execute_document(request) {
            Ok(_) => panic!("missing adapter should fail before options"),
            Err(error) => error,
        };
        let mut diagnostics = docnav_diagnostics::DiagnosticStack::new();
        let id = diagnostics
            .push(error.diagnostic().clone())
            .expect("diagnostic should be valid");
        let record = diagnostics.get(id).expect("diagnostic record");
        let protocol_error =
            docnav_protocol::ProtocolError::from_diagnostic_record(record).unwrap();

        assert_eq!(
            protocol_error.code(),
            docnav_protocol::ProtocolDiagnosticCode::AdapterUnavailable
        );
        assert_eq!(protocol_error.owner(), "adapter_selection");
        assert_eq!(
            protocol_error
                .details()
                .get("adapter_id")
                .and_then(Value::as_str),
            Some("missing-adapter")
        );
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
            max_heading_level: max_heading_level.map(|value| try_positive(value).unwrap()),
            output: Some(OutputMode::ProtocolJson),
            adapter: adapter.map(str::to_owned),
        }
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
            project_config_path: project_root.join(".docnav").join("docnav.json"),
            user_config_path: project_root.join(".docnav-user").join("docnav.json"),
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
}
