use std::time::Instant;

use crate::cli::{DocumentCommand, OutputMode};
use crate::config::{load_context_for_project, ConfigContext};
use crate::error::AppResult;
use crate::invocation_log::InvocationLogger;
use crate::output::CommandOutcome;
use crate::project_context::ProjectContext;
use crate::runtime::{DocnavRuntime, DocumentRequest};

use super::PipelineContext;

pub(super) fn execute<T: DocnavRuntime>(
    command: DocumentCommand,
    pipeline: &PipelineContext<'_, T>,
    error_output_mode: &mut OutputMode,
) -> AppResult<CommandOutcome> {
    let context = DocumentPipelineContext::from_command(command)?;
    // Navigation owns the final output mode; this hint only preserves config
    // precedence when execution fails before navigation returns an outcome.
    if let Some(output_mode) = context.error_output_mode() {
        *error_output_mode = output_mode;
    }
    pipeline
        .services()
        .runtime()
        .execute_document(context.into_request())
}

struct DocumentPipelineContext {
    command: DocumentCommand,
    context: ConfigContext,
    started: Instant,
}

impl DocumentPipelineContext {
    fn from_command(command: DocumentCommand) -> AppResult<Self> {
        let started = Instant::now();
        let project = ProjectContext::discover_with_cli_config_paths(
            command.config_paths.project_config.as_deref(),
            command.config_paths.user_config.as_deref(),
        )?;
        let logger = InvocationLogger::explicit_cli(&command, &project);
        let context = match load_context_for_project(project.clone()) {
            Ok(context) => context,
            Err(error) => {
                let log_context = logger.document_context(&command, &project, None);
                logger.record_app_error(&log_context, &error, "config", started.elapsed());
                return Err(error);
            }
        };
        Ok(Self {
            command,
            context,
            started,
        })
    }

    fn error_output_mode(&self) -> Option<OutputMode> {
        if let Some(output_mode) = self.command.output_mode() {
            return Some(output_mode);
        }
        if self.command.has_output_candidate() {
            return None;
        }
        configured_output_mode(&self.context)
    }

    fn into_request(self) -> DocumentRequest {
        DocumentRequest::from_config_context_started(self.command, self.context, self.started)
    }
}

fn configured_output_mode(context: &ConfigContext) -> Option<OutputMode> {
    if let Some(value) = context.project_config.defaults.output.as_deref() {
        return value.parse().ok();
    }
    context
        .user_config
        .defaults
        .output
        .as_deref()
        .and_then(|value| value.parse().ok())
}
