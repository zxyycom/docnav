use crate::cli::DocumentCommand;
use crate::error::AppResult;
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
    project: ProjectContext,
}

impl DocumentPipelineContext {
    fn from_command(command: DocumentCommand) -> AppResult<Self> {
        let project = ProjectContext::discover_with_cli_config_paths(
            command.config_paths.project_config.as_deref(),
            command.config_paths.user_config.as_deref(),
        )?;
        Ok(Self { command, project })
    }

    fn into_request(self) -> DocumentRequest {
        DocumentRequest::from_command(self.command, self.project)
    }
}
