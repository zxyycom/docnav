use crate::cli::AdapterCommand;
use crate::error::AppResult;
use crate::output::CommandOutcome;

pub(super) fn execute(command: AdapterCommand) -> AppResult<CommandOutcome> {
    crate::registry::execute(command)
}
