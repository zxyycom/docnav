use docnav_diagnostics::{DiagnosticRecordDraft, DiagnosticSource};
use docnav_protocol::{Operation, PositiveInteger};
use serde::Serialize;
use serde_json::Value;

use crate::cli::{CliWarning, DocumentCommand, OutputMode};
use crate::config::{ConfigContext, ResolvedValue};
use crate::error::AppResult;
use crate::invoke::invoke_adapter;
use crate::output::{outcome_for_response, CommandOutcome};
use crate::project_context::ProjectContext;
use crate::project_paths::normalize_document_path;
use crate::registry::AdapterRegistry;
use crate::routing::{select_adapter, AdapterSelectionRequest, AdapterSelectionWarning};
use crate::standard_parameters::resolve_core_document_parameters;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentRequest {
    pub project: ProjectContext,
    pub operation: Operation,
    pub path: String,
    pub ref_id: Option<String>,
    pub query: Option<String>,
    pub page: Option<PositiveInteger>,
    pub limit: Option<PositiveInteger>,
    pub output: OutputMode,
    pub adapter: Option<String>,
    pub defaults: ResolvedDocumentDefaults,
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
        let selection = select_adapter(AdapterSelectionRequest {
            project: &request.project,
            registry: &registry,
            document: &document,
            operation: request.operation,
            preselected_adapter_id: request.adapter.as_deref(),
            preselected_source: &request.defaults.adapter.source,
        })?;
        let warnings = cli_warnings(selection.warnings.clone());
        let invoke = match invoke_adapter(
            &request.project.project_root,
            &selection.record,
            &document,
            &request,
        ) {
            Ok(invoke) => invoke,
            Err(error) => return Err(error.with_related_diagnostics(cli_warning_drafts(&warnings))),
        };

        let _ = (
            &selection.manifest,
            &selection.probe,
            &selection.evidence,
            &invoke.request,
        );
        Ok(outcome_for_response(invoke.response, request.output)?.with_warnings(warnings))
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
        let selection = select_adapter(AdapterSelectionRequest {
            project: &context.project,
            registry: &registry,
            document: &document,
            operation: effective_operation,
            preselected_adapter_id: defaults.adapter.value.as_str(),
            preselected_source: &defaults.adapter.source,
        })?;

        Ok(DocumentContextOutput {
            path: document.adapter_path,
            operation: Some(effective_operation),
            adapter: AdapterContextOutput {
                selected: Some(selection.record.id),
                source: adapter_source(&defaults.adapter, &selection.evidence),
                note: "selected adapter passed manifest and probe checks".to_owned(),
            },
            defaults,
            runtime_status: "adapter_runtime_ready".to_owned(),
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
            output: resolved.output,
            adapter: resolved.adapter,
            defaults: resolved.defaults,
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

fn cli_warnings(warnings: Vec<AdapterSelectionWarning>) -> Vec<CliWarning> {
    warnings
        .into_iter()
        .map(|warning| {
            CliWarning::adapter_candidate_failure(
                &warning.adapter_id,
                warning.stage.as_str(),
                &warning.code,
                &warning.reason,
                warning.preselected,
            )
        })
        .collect()
}

fn cli_warning_drafts(warnings: &[CliWarning]) -> Vec<DiagnosticRecordDraft> {
    warnings
        .iter()
        .map(|warning| warning.to_record_draft(DiagnosticSource::with_stage("docnav", "runtime")))
        .collect()
}
