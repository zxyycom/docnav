use docnav_protocol::{Operation, PositiveInteger};
use serde::Serialize;
use serde_json::{json, Value};

use crate::cli::{CliWarning, DocumentCommand, OutputMode};
use crate::config::{self, ConfigContext, ResolvedValue};
use crate::error::AppResult;
use crate::invoke::invoke_adapter;
use crate::output::{outcome_for_response, CommandOutcome};
use crate::project_context::ProjectContext;
use crate::project_paths::normalize_document_path;
use crate::registry::AdapterRegistry;
use crate::routing::{select_adapter, AdapterSelectionRequest, AdapterSelectionWarning};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentRequest {
    pub project: ProjectContext,
    pub operation: Operation,
    pub path: String,
    pub ref_id: Option<String>,
    pub query: Option<String>,
    pub page: Option<PositiveInteger>,
    pub limit_chars: Option<PositiveInteger>,
    pub output: OutputMode,
    pub adapter: Option<String>,
    pub defaults: ResolvedDocumentDefaults,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ResolvedDocumentDefaults {
    pub adapter: ResolvedValue,
    pub limit_chars: Option<ResolvedValue>,
    pub output: ResolvedValue,
    pub page: Option<ResolvedValue>,
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
        let invoke = invoke_adapter(
            &request.project.project_root,
            &selection.record,
            &document,
            &request,
        )?;

        let _ = (
            &selection.manifest,
            &selection.probe,
            &selection.evidence,
            &invoke.request,
        );
        Ok(outcome_for_response(invoke.response, request.output)?
            .with_warnings(cli_warnings(selection.warnings)))
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
        let defaults = resolve_document_defaults(&command, context)?;
        let output = defaults
            .output
            .value
            .as_str()
            .and_then(|value| value.parse::<OutputMode>().ok())
            .unwrap_or(OutputMode::ReadableView);
        let adapter = defaults.adapter.value.as_str().map(str::to_owned);

        Ok(Self {
            project: context.project.clone(),
            operation: command.operation,
            path: command.path,
            ref_id: command.ref_id,
            query: command.query,
            page: command.page.or_else(|| positive(1)),
            limit_chars: command.limit_chars.or_else(|| {
                defaults
                    .limit_chars
                    .as_ref()
                    .and_then(|value| value.value.as_u64())
                    .and_then(|value| u32::try_from(value).ok())
                    .and_then(std::num::NonZeroU32::new)
            }),
            output,
            adapter,
            defaults,
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
        limit_chars: None,
        output: None,
        adapter: None,
    };
    let defaults = resolve_document_defaults(&command, context)?;
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

fn resolve_document_defaults(
    command: &DocumentCommand,
    context: &ConfigContext,
) -> AppResult<ResolvedDocumentDefaults> {
    let adapter = config::resolve_adapter(command.adapter.as_deref(), context);
    let output = config::resolve_output(command.output, context)?;
    let limit_chars = if command.operation == Operation::Info {
        None
    } else {
        Some(config::resolve_limit_chars(command.limit_chars, context)?)
    };
    let page = if command.operation == Operation::Info {
        None
    } else {
        Some(match command.page {
            Some(page) => ResolvedValue::explicit(json!(page.get())),
            None => ResolvedValue::built_in(json!(1)),
        })
    };

    Ok(ResolvedDocumentDefaults {
        adapter,
        limit_chars,
        output,
        page,
    })
}

fn positive(value: u32) -> Option<PositiveInteger> {
    std::num::NonZeroU32::new(value)
}
