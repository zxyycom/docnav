use docnav_navigation::{
    execute_navigation_command, NavigationCommand, NavigationConfigSourceDescriptors,
    NavigationOutputMode,
};
use std::time::Instant;

use crate::cli::{DocumentCommand, OutputMode};
use crate::config::{ConfigContext, CoreConfig};
use crate::error::{AppError, AppResult};
use crate::invocation_log::{DocumentInvocationLog, InvocationLogger};
use crate::output::{outcome_for_response, CommandOutcome};
use crate::parameter_catalog::document_parameter_catalog;
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

pub trait DocnavRuntime {
    fn execute_document(&self, request: DocumentRequest) -> AppResult<CommandOutcome>;
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
        let registry = AdapterRegistry::builtin();
        let catalog = match document_parameter_catalog() {
            Ok(catalog) => catalog,
            Err(error) => {
                let error = AppError::internal(format!(
                    "document-parameter-catalog-build-failed:runtime:{error}"
                ));
                logger.record_app_error(&log_context, &error, "operation", started.elapsed());
                return Err(error);
            }
        };
        let outcome = match execute_navigation_command(
            navigation_command(&request.command, document.adapter_path),
            request.config_source_descriptors,
            &catalog,
            &registry,
        ) {
            Ok(outcome) => outcome,
            Err(error) => {
                logger.record_navigation_error(&log_context, &error, started.elapsed());
                return Err(AppError::new(error.into_diagnostic()));
            }
        };
        let output = output_mode(outcome.output);
        let invocation_log = DocumentInvocationLog::new(logger, log_context, started);
        outcome_for_response(outcome, output, Some(invocation_log))
    }
}

fn navigation_command(command: &DocumentCommand, document_path: String) -> NavigationCommand {
    NavigationCommand {
        operation: command.operation,
        document_path,
        ref_id: command.ref_id.clone(),
        query: command.query.clone(),
        cli_source: command.cli_source.as_ref().clone(),
    }
}

fn output_mode(output: NavigationOutputMode) -> OutputMode {
    match output {
        NavigationOutputMode::ReadableView => OutputMode::ReadableView,
        NavigationOutputMode::ReadableJson => OutputMode::ReadableJson,
        NavigationOutputMode::ProtocolJson => OutputMode::ProtocolJson,
    }
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

#[cfg(test)]
mod tests;
