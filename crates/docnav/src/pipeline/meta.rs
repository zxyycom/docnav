use crate::error::AppResult;
use crate::output::CommandOutcome;

use super::MetaCommand;

pub(super) fn execute(command: MetaCommand) -> AppResult<CommandOutcome> {
    match command {
        MetaCommand::Version => Ok(CommandOutcome::plain_text(format!(
            "docnav {}",
            env!("CARGO_PKG_VERSION")
        ))),
        MetaCommand::Help(text) => Ok(CommandOutcome::plain_text(text)),
    }
}
