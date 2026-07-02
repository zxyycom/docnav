use docnav_adapter_contracts::{AdapterError, NativeOptionIssue, NativeOptionSpec};
use docnav_navigation::{execute_protocol_request, protocol_request, OperationInput};
use docnav_protocol::{Operation, OptionEntry, Options, PositiveInteger};
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
        let options = project_native_options_for_selected_adapter(
            options,
            selection.record.id(),
            &selection.record.native_options_for(request.operation),
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

fn project_native_options_for_selected_adapter(
    options: Option<Options>,
    adapter_id: &str,
    selected_specs: &[NativeOptionSpec],
) -> AppResult<Option<Options>> {
    let Some(options) = options else {
        return Ok(None);
    };
    let mut projected = Options::new();
    for (key, value) in options.iter() {
        let selected_entries = options
            .entries()
            .iter()
            .filter(|entry| selected_option_entry(entry, selected_specs))
            .filter(|entry| entry.key == *key)
            .cloned()
            .collect::<Vec<_>>();
        if selected_entries.is_empty() {
            return Err(unsupported_native_option(adapter_id, key, value, &options));
        }
        for entry in selected_entries {
            projected.insert_entry(entry);
        }
    }
    Ok((!projected.is_empty()).then_some(projected))
}

fn selected_option_entry(entry: &OptionEntry, selected_specs: &[NativeOptionSpec]) -> bool {
    selected_specs.iter().any(|spec| {
        spec.identity == entry.identity
            && spec.owner == entry.owner
            && spec.namespace == entry.namespace
            && spec.key == entry.key
            && spec.value_kind().as_str() == entry.type_variant
    })
}

fn unsupported_native_option(
    adapter_id: &str,
    key: &str,
    value: &Value,
    options: &Options,
) -> AppError {
    let entry = options.entry_for_key(key);
    let issue = NativeOptionIssue {
        owner: adapter_id.to_owned(),
        namespace: entry
            .map(|entry| entry.namespace.clone())
            .unwrap_or_else(|| "options".to_owned()),
        key: key.to_owned(),
        source: entry
            .map(|entry| entry.source.clone())
            .unwrap_or_else(|| "direct".to_owned()),
        reason_code: "unsupported".to_owned(),
        field: format!("arguments.options.{key}"),
        received: Some(received_value(value)),
        expected: None,
        type_variant: entry.map(|entry| entry.type_variant.clone()),
    };
    AppError::new(
        AdapterError::native_option_invalid(
            "Native option is not supported by the selected adapter.",
            issue,
            [format!(
                "Remove option {key} or select an adapter that supports it."
            )],
        )
        .into_diagnostic(),
    )
}

fn received_value(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| value.to_string())
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
        native_options: Vec::new(),
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
mod tests;
