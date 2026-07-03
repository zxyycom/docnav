use docnav_navigation::{
    execute_navigation_command, resolve_navigation_context, NavigationCommand,
    NavigationConfigSourceDescriptor, NavigationConfigSourceDescriptors, NavigationContextDefaults,
    NavigationNativeOptionInput, NavigationOutputMode, NavigationPaginationDefaults,
    NavigationResolvedValue,
};
use docnav_protocol::Operation;
use serde::Serialize;

use crate::cli::{DocumentCommand, OutputMode};
use crate::config::{ConfigContext, ResolvedValue};
use crate::error::{AppError, AppResult};
use crate::output::{outcome_for_response, CommandOutcome};
use crate::project_context::ProjectContext;
use crate::project_paths::normalize_document_path;
use crate::registry::AdapterRegistry;

#[derive(Clone, Debug, PartialEq)]
pub struct DocumentRequest {
    pub project: ProjectContext,
    pub command: DocumentCommand,
    pub config_source_descriptors: NavigationConfigSourceDescriptors,
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
        context: &ConfigContext,
    ) -> AppResult<DocumentContextOutput>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AdapterRuntime;

impl DocnavRuntime for AdapterRuntime {
    fn execute_document(&self, request: DocumentRequest) -> AppResult<CommandOutcome> {
        let document = normalize_document_path(&request.project, &request.command.path)?;
        let registry = AdapterRegistry::load(&request.project)?;
        let outcome = execute_navigation_command(
            navigation_command(&request.command, document.adapter_path),
            request.config_source_descriptors,
            &registry,
        )
        .map_err(|error| AppError::new(error.into_diagnostic()))?;
        outcome_for_response(outcome.response, output_mode(outcome.output)?)
    }

    fn describe_document_context(
        &self,
        path: String,
        operation: Option<Operation>,
        context: &ConfigContext,
    ) -> AppResult<DocumentContextOutput> {
        let effective_operation = operation.unwrap_or(Operation::Outline);
        let document = normalize_document_path(&context.project, &path)?;
        let registry = AdapterRegistry::load(&context.project)?;
        let context = resolve_navigation_context(
            context_navigation_command(effective_operation, document.adapter_path.clone()),
            navigation_config_source_descriptors(&context.project),
            &registry,
        )
        .map_err(|error| AppError::new(error.into_diagnostic()))?;

        Ok(DocumentContextOutput {
            path: document.adapter_path.clone(),
            operation: Some(effective_operation),
            adapter: AdapterContextOutput {
                selected: Some(context.selection.adapter_id),
                source: context.selection.source,
                note: context.selection.note,
            },
            defaults: document_defaults(context.defaults),
            runtime_status: "static_adapter_registry_ready".to_owned(),
        })
    }
}

fn navigation_command(command: &DocumentCommand, document_path: String) -> NavigationCommand {
    NavigationCommand {
        operation: command.operation,
        document_path,
        ref_id: command.ref_id.clone(),
        query: command.query.clone(),
        page: command.page,
        pagination_enabled: command.pagination_enabled,
        limit: command.limit,
        output: command.output.map(navigation_output_mode),
        adapter: command.adapter.clone(),
        native_options: navigation_native_options(command),
    }
}

fn context_navigation_command(operation: Operation, document_path: String) -> NavigationCommand {
    NavigationCommand {
        operation,
        document_path,
        ref_id: None,
        query: None,
        page: None,
        pagination_enabled: None,
        limit: None,
        output: None,
        adapter: None,
        native_options: Vec::new(),
    }
}

fn navigation_native_options(command: &DocumentCommand) -> Vec<NavigationNativeOptionInput> {
    command
        .native_options
        .iter()
        .map(|input| NavigationNativeOptionInput {
            flag: input.flag.clone(),
            value: input.value.clone(),
        })
        .collect()
}

fn navigation_config_source_descriptors(
    project: &ProjectContext,
) -> NavigationConfigSourceDescriptors {
    NavigationConfigSourceDescriptors {
        project: NavigationConfigSourceDescriptor::default(project.project_config_path.clone()),
        user: NavigationConfigSourceDescriptor::default(project.user_config_path.clone()),
    }
}

fn navigation_output_mode(output: OutputMode) -> NavigationOutputMode {
    match output {
        OutputMode::ReadableView => NavigationOutputMode::ReadableView,
        OutputMode::ReadableJson => NavigationOutputMode::ReadableJson,
        OutputMode::ProtocolJson => NavigationOutputMode::ProtocolJson,
    }
}

fn output_mode(output: NavigationOutputMode) -> AppResult<OutputMode> {
    output
        .as_str()
        .parse()
        .map_err(|error| AppError::internal(format!("navigation-output-mode:{error}")))
}

impl DocumentRequest {
    pub fn from_command(command: DocumentCommand, project: ProjectContext) -> Self {
        let config_source_descriptors = navigation_config_source_descriptors(&project);
        Self {
            project,
            command,
            config_source_descriptors,
        }
    }
}

fn document_defaults(defaults: NavigationContextDefaults) -> ResolvedDocumentDefaults {
    ResolvedDocumentDefaults {
        adapter: resolved_value(defaults.adapter),
        pagination: defaults.pagination.map(pagination_defaults),
        output: resolved_value(defaults.output),
        page: defaults.page.map(resolved_value),
    }
}

fn pagination_defaults(defaults: NavigationPaginationDefaults) -> ResolvedPaginationDefaults {
    ResolvedPaginationDefaults {
        enabled: resolved_value(defaults.enabled),
        limit: resolved_value(defaults.limit),
    }
}

fn resolved_value(value: NavigationResolvedValue) -> ResolvedValue {
    ResolvedValue {
        value: value.value,
        source: value.source,
    }
}

#[cfg(test)]
mod tests;
