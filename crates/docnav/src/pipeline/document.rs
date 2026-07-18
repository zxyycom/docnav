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
    let context = DocumentPipelineContext::from_command(command, error_output_mode)?;
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
    fn from_command(
        command: DocumentCommand,
        error_output_mode: &mut OutputMode,
    ) -> AppResult<Self> {
        let started = Instant::now();
        // Navigation owns the final mode. This hint only preserves source
        // precedence when failure prevents navigation from returning.
        let mut output_precedence_locked = command.has_output_candidate();
        if let Some(output_mode) = command.output_mode() {
            *error_output_mode = output_mode;
        }
        let project = ProjectContext::discover_with_cli_config_paths(
            command.config_paths.project_config.as_deref(),
            command.config_paths.user_config.as_deref(),
        )?;
        let logger = InvocationLogger::explicit_cli(&command, &project);
        let context = match load_context_for_project(project.clone(), |config| {
            if output_precedence_locked {
                return;
            }
            let Some(value) = config.defaults.output.as_deref() else {
                return;
            };
            output_precedence_locked = true;
            if let Ok(output_mode) = value.parse() {
                *error_output_mode = output_mode;
            }
        }) {
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

    fn into_request(self) -> DocumentRequest {
        DocumentRequest::from_config_context_started(self.command, self.context, self.started)
    }
}
