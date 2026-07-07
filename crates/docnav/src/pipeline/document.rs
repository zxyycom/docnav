use std::time::Instant;

use crate::cli::DocumentCommand;
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
) -> AppResult<CommandOutcome> {
    let context = DocumentPipelineContext::from_command(command)?;
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

    fn into_request(self) -> DocumentRequest {
        DocumentRequest::from_config_context_started(self.command, self.context, self.started)
    }
}
