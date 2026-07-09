use crate::cli::ConfigCommand;
use crate::error::AppResult;
use crate::output::CommandOutcome;
use crate::runtime::DocnavRuntime;

use super::PipelineContext;

pub(super) fn execute<T: DocnavRuntime>(
    command: ConfigCommand,
    _pipeline: &PipelineContext<'_, T>,
) -> AppResult<CommandOutcome> {
    crate::config::execute(command)
}
