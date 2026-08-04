use docnav_navigation::{
    execute_prepared_navigation_command, prepare_navigation_command, NavigationCommand,
    NavigationConfigSourceDescriptors, NavigationError, NavigationOutputMode,
};
use std::time::Instant;

use crate::cli::{DocumentCommand, OutputMode};
use crate::config::{ConfigContext, CoreConfig};
use crate::error::{AppError, AppResult};
use crate::invocation_log::{
    DocumentInvocationLog, DocumentLogContext, InvocationLogDiagnostic, InvocationLogger,
};
use crate::output::{outcome_for_response, CommandOutcome};
use crate::parameter_catalog::document_parameter_catalog;
use crate::project_context::ProjectContext;
use crate::project_paths::{normalize_document_path, routing_document_pathname};
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
    #[cfg(test)]
    fn execute_document(&self, request: DocumentRequest) -> AppResult<CommandOutcome> {
        self.execute_document_with_diagnostics(request, &mut Vec::new())
    }

    fn execute_document_with_diagnostics(
        &self,
        request: DocumentRequest,
        invocation_log_diagnostics: &mut Vec<InvocationLogDiagnostic>,
    ) -> AppResult<CommandOutcome>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AdapterRuntime;

struct RuntimeLogContext<'a> {
    logger: InvocationLogger,
    started: Instant,
    diagnostics: &'a mut Vec<InvocationLogDiagnostic>,
}

impl<'a> RuntimeLogContext<'a> {
    fn from_request(
        request: &DocumentRequest,
        diagnostics: &'a mut Vec<InvocationLogDiagnostic>,
    ) -> Self {
        let logger = InvocationLogger::from_command(
            &request.command,
            &request.project,
            &request.project_config,
            &request.user_config,
        );
        Self {
            logger,
            started: request.started,
            diagnostics,
        }
    }

    fn document_context(
        &self,
        command: &DocumentCommand,
        project: &ProjectContext,
        absolute_path: Option<&std::path::Path>,
    ) -> DocumentLogContext {
        self.logger
            .document_context(command, project, absolute_path)
    }

    fn navigation_result<T>(
        &mut self,
        context: &DocumentLogContext,
        result: Result<T, NavigationError>,
    ) -> AppResult<T> {
        result.map_err(|error| {
            self.diagnostics.extend(self.logger.record_navigation_error(
                context,
                &error,
                self.started.elapsed(),
            ));
            AppError::new(error.into_diagnostic())
        })
    }

    fn operation_result<T>(
        &mut self,
        context: &DocumentLogContext,
        result: AppResult<T>,
    ) -> AppResult<T> {
        result.inspect_err(|error| {
            self.diagnostics.extend(self.logger.record_app_error(
                context,
                error,
                "operation",
                self.started.elapsed(),
            ));
        })
    }

    fn finish(self, context: DocumentLogContext) -> DocumentInvocationLog {
        DocumentInvocationLog::new(self.logger, context, self.started)
    }
}

impl DocnavRuntime for AdapterRuntime {
    fn execute_document_with_diagnostics(
        &self,
        request: DocumentRequest,
        invocation_log_diagnostics: &mut Vec<InvocationLogDiagnostic>,
    ) -> AppResult<CommandOutcome> {
        let mut logging = RuntimeLogContext::from_request(&request, invocation_log_diagnostics);
        let initial_log_context =
            logging.document_context(&request.command, &request.project, None);
        let routing_pathname = routing_document_pathname(&request.project, &request.command.path);
        let registry = AdapterRegistry::builtin();
        let prepared = logging.navigation_result(
            &initial_log_context,
            prepare_navigation_command(
                navigation_command(&request.command, routing_pathname),
                request.config_source_descriptors,
                &registry,
            ),
        )?;
        let document = logging.operation_result(
            &initial_log_context,
            normalize_document_path(&request.project, &request.command.path),
        )?;
        let document_log_context = logging.document_context(
            &request.command,
            &request.project,
            Some(&document.absolute_path),
        );
        let catalog = document_parameter_catalog().map_err(|error| {
            AppError::internal(format!(
                "document-parameter-catalog-build-failed:runtime:{error}"
            ))
        });
        let catalog = logging.operation_result(&document_log_context, catalog)?;
        let outcome = logging.navigation_result(
            &document_log_context,
            execute_prepared_navigation_command(prepared, document.adapter_path, &catalog),
        )?;
        let output = output_mode(outcome.output);
        let invocation_log = logging.finish(document_log_context);
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
