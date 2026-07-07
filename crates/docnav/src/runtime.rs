use docnav_navigation::{
    execute_navigation_command, resolve_navigation_context, NavigationCommand,
    NavigationConfigSourceDescriptors, NavigationContextDefaults, NavigationNativeOptionInput,
    NavigationOutputMode, NavigationPaginationDefaults, NavigationResolvedValue,
};
use docnav_protocol::Operation;
use serde::Serialize;
use std::time::Instant;

use crate::cli::{DocumentCommand, OutputMode};
use crate::config::{ConfigContext, CoreConfig, ResolvedValue};
use crate::error::{AppError, AppResult};
use crate::invocation_log::{DocumentInvocationLog, InvocationLogger};
use crate::output::{outcome_for_response, CommandOutcome};
use crate::project_context::ProjectContext;
use crate::project_paths::normalize_document_path;
use crate::registry::AdapterRegistry;

#[derive(Clone, Debug, PartialEq)]
pub struct DocumentRequest {
    pub project: ProjectContext,
    pub command: DocumentCommand,
    pub config_source_descriptors: NavigationConfigSourceDescriptors,
    pub(crate) project_config: CoreConfig,
    pub(crate) user_config: CoreConfig,
    pub(crate) started: Instant,
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
        project: &ProjectContext,
    ) -> AppResult<DocumentContextOutput>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AdapterRuntime;

impl DocnavRuntime for AdapterRuntime {
    fn execute_document(&self, request: DocumentRequest) -> AppResult<CommandOutcome> {
        let logger = InvocationLogger::from_command(
            &request.command,
            &request.project,
            &request.project_config,
            &request.user_config,
        );
        let started = request.started;
        let document = match normalize_document_path(&request.project, &request.command.path) {
            Ok(document) => document,
            Err(error) => {
                let context = logger.document_context(&request.command, &request.project, None);
                logger.record_app_error(&context, &error, "operation", started.elapsed());
                return Err(error);
            }
        };
        let log_context = logger.document_context(
            &request.command,
            &request.project,
            Some(&document.absolute_path),
        );
        let registry = match AdapterRegistry::load(&request.project) {
            Ok(registry) => registry,
            Err(error) => {
                logger.record_app_error(&log_context, &error, "operation", started.elapsed());
                return Err(error);
            }
        };
        let outcome = match execute_navigation_command(
            navigation_command(&request.command, document.adapter_path),
            request.config_source_descriptors,
            &registry,
        ) {
            Ok(outcome) => outcome,
            Err(error) => {
                logger.record_navigation_error(&log_context, &error, started.elapsed());
                return Err(AppError::new(error.into_diagnostic()));
            }
        };
        let output = match output_mode(outcome.output) {
            Ok(output) => output,
            Err(error) => {
                logger.record_app_error(&log_context, &error, "operation", started.elapsed());
                return Err(error);
            }
        };
        let invocation_log = DocumentInvocationLog::new(logger, log_context, started);
        outcome_for_response(outcome, output, Some(invocation_log))
    }

    fn describe_document_context(
        &self,
        path: String,
        operation: Option<Operation>,
        project: &ProjectContext,
    ) -> AppResult<DocumentContextOutput> {
        let effective_operation = operation.unwrap_or(Operation::Outline);
        let document = normalize_document_path(project, &path)?;
        let registry = AdapterRegistry::load(project)?;
        let context = resolve_navigation_context(
            context_navigation_command(effective_operation, document.adapter_path.clone()),
            project.navigation_config_source_descriptors(),
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
        .map_err(|_| AppError::internal("navigation-output-mode-invalid"))
}

impl DocumentRequest {
    #[cfg(test)]
    pub(crate) fn from_config_context(command: DocumentCommand, context: ConfigContext) -> Self {
        Self::from_config_context_started(command, context, Instant::now())
    }

    pub(crate) fn from_config_context_started(
        command: DocumentCommand,
        context: ConfigContext,
        started: Instant,
    ) -> Self {
        let config_source_descriptors = context.project.navigation_config_source_descriptors();
        Self {
            project: context.project,
            command,
            config_source_descriptors,
            project_config: context.project_config,
            user_config: context.user_config,
            started,
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
