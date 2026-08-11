mod adapter_command;
mod argument_helpers;
mod config_command;
mod document;
mod spec;
mod utility_command;

use clap::error::ErrorKind;
use docnav_protocol::Operation;

use crate::error::{AppError, AppResult};

use super::command_model::{CliCommand, ConfigPathArgs, ParsedCli};

use spec::{cli_command, is_known_root_command};

pub(super) use spec::{
    arg_ids, command_names, config_inspect_command, document_clap_command, utility_clap_command,
};

pub fn parse<I, S>(args: I) -> AppResult<ParsedCli>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    let Some((command, rest)) = args.split_first() else {
        return Err(AppError::invalid_request("command", "missing command"));
    };

    if is_help_flag(command) {
        return Ok(ParsedCli::new(CliCommand::Help(
            cli_command().render_long_help().to_string(),
        )));
    }

    if !is_known_root_command(command) {
        return Err(AppError::invalid_request(
            "command",
            format!("unknown command {command:?}"),
        ));
    }

    if let Some(help) = help_text(command, rest)? {
        return Ok(ParsedCli::new(CliCommand::Help(help)));
    }

    match command.as_str() {
        command_names::OUTLINE => document::parse_document_command(Operation::Outline, rest),
        command_names::READ => document::parse_document_command(Operation::Read, rest),
        command_names::FIND => document::parse_document_command(Operation::Find, rest),
        command_names::INFO => document::parse_document_command(Operation::Info, rest),
        command_names::ADAPTER => adapter_command::parse_adapter_command(rest),
        command_names::CONFIG => config_command::parse_config_command(rest),
        command_names::INIT => utility_command::parse_utility_command(
            CliCommand::Init(ConfigPathArgs::default()),
            command_names::INIT,
            rest,
        ),
        command_names::DOCTOR => utility_command::parse_utility_command(
            CliCommand::Doctor(ConfigPathArgs::default()),
            command_names::DOCTOR,
            rest,
        ),
        command_names::VERSION => utility_command::parse_utility_command(
            CliCommand::Version,
            command_names::VERSION,
            rest,
        ),
        _ => unreachable!("known root commands are handled above"),
    }
}

fn help_text(command: &str, args: &[String]) -> AppResult<Option<String>> {
    let command_shape = if let Some(operation) = document_operation(command) {
        document_clap_command(operation)?.command
    } else {
        cli_command()
            .find_subcommand(command)
            .cloned()
            .expect("known root command has a clap command shape")
    };
    match command_shape.try_get_matches_from(argument_helpers::clap_argv(command, args.to_vec())) {
        Err(error) if error.kind() == ErrorKind::DisplayHelp => Ok(Some(error.to_string())),
        Ok(_) | Err(_) => Ok(None),
    }
}

fn document_operation(command: &str) -> Option<Operation> {
    match command {
        command_names::OUTLINE => Some(Operation::Outline),
        command_names::READ => Some(Operation::Read),
        command_names::FIND => Some(Operation::Find),
        command_names::INFO => Some(Operation::Info),
        _ => None,
    }
}

fn is_help_flag(arg: &str) -> bool {
    arg == "--help" || arg == "-h"
}

#[cfg(test)]
mod tests;
