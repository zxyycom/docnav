mod argument_helpers;
mod config_command;
mod document;
mod spec;
mod utility_command;

use docnav_protocol::Operation;

use crate::error::{AppError, AppResult};

use super::command_model::{CliCommand, ParsedCli};

use spec::{cli_command, is_known_root_command};

pub(super) use spec::{
    arg_ids, command_names, config_get_command, config_list_command, config_set_command,
    config_unset_command, document_clap_command, utility_clap_command,
};

pub fn parse<I, S>(args: I) -> AppResult<ParsedCli>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    if let Some(help) = help_text(&args) {
        return Ok(ParsedCli::new(CliCommand::Help(help), Vec::new()));
    }

    let Some((command, rest)) = args.split_first() else {
        return Err(AppError::invalid_request("command", "missing command"));
    };

    if !is_known_root_command(command) {
        return Err(AppError::invalid_request(
            "command",
            format!("unknown command {command:?}"),
        ));
    }

    match command.as_str() {
        command_names::OUTLINE => document::parse_document_command(Operation::Outline, rest),
        command_names::READ => document::parse_document_command(Operation::Read, rest),
        command_names::FIND => document::parse_document_command(Operation::Find, rest),
        command_names::INFO => document::parse_document_command(Operation::Info, rest),
        command_names::CONFIG => config_command::parse_config_command(rest),
        command_names::INIT => {
            utility_command::parse_utility_command(CliCommand::Init, command_names::INIT, rest)
        }
        command_names::DOCTOR => {
            utility_command::parse_utility_command(CliCommand::Doctor, command_names::DOCTOR, rest)
        }
        command_names::VERSION => utility_command::parse_utility_command(
            CliCommand::Version,
            command_names::VERSION,
            rest,
        ),
        _ => unreachable!("known root commands are handled above"),
    }
}

fn help_text(args: &[String]) -> Option<String> {
    if !args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return None;
    }
    let mut root = cli_command();
    let Some(first) = args.first().map(String::as_str) else {
        return Some(root.render_long_help().to_string());
    };
    if first == "--help" || first == "-h" {
        return Some(root.render_long_help().to_string());
    }
    let Some(command) = root.find_subcommand_mut(first) else {
        return Some(root.render_long_help().to_string());
    };
    if first == command_names::CONFIG {
        if let Some(second) = args.get(1).map(String::as_str) {
            if second != "--help" && second != "-h" {
                if let Some(subcommand) = command.find_subcommand_mut(second) {
                    return Some(subcommand.render_long_help().to_string());
                }
            }
        }
    }
    Some(command.render_long_help().to_string())
}

#[cfg(test)]
mod tests;
