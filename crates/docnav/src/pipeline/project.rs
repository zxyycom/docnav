use crate::error::AppResult;
use crate::output::CommandOutcome;

use super::ProjectCommand;

pub(super) fn execute(command: ProjectCommand) -> AppResult<CommandOutcome> {
    match command {
        ProjectCommand::Init(config_paths) => crate::config::init_project(config_paths),
        ProjectCommand::Doctor(config_paths) => crate::config::doctor(config_paths),
    }
}
