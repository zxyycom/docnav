use crate::cli::{AdapterCommand, CliCommand, ParsedCli};
use crate::error::{AppError, AppResult};

use super::command_names;

pub(super) fn parse_adapter_command(args: &[String]) -> AppResult<ParsedCli> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(AppError::invalid_request(
            "adapter",
            "adapter requires subcommand list",
        ));
    };

    match (subcommand.as_str(), rest) {
        (command_names::ADAPTER_LIST, []) => {
            Ok(ParsedCli::new(CliCommand::Adapter(AdapterCommand::List)))
        }
        (command_names::ADAPTER_LIST, _) => Err(AppError::invalid_request(
            "adapter list",
            "adapter list does not accept arguments",
        )),
        (unsupported, _) => Err(AppError::invalid_request(
            "adapter",
            format!("unsupported adapter command {unsupported:?}"),
        )),
    }
}
