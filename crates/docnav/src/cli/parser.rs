mod common;
mod config_command;
mod document;
mod nullary;

use docnav_protocol::Operation;

use crate::error::{AppError, AppResult};

use super::types::{CliCommand, ParsedCli};

pub fn parse<I, S>(args: I) -> AppResult<ParsedCli>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    let Some((command, rest)) = args.split_first() else {
        return Err(AppError::invalid_request("command", "missing command"));
    };

    match command.as_str() {
        "outline" => document::parse_document_command(Operation::Outline, rest),
        "read" => document::parse_document_command(Operation::Read, rest),
        "find" => document::parse_document_command(Operation::Find, rest),
        "info" => document::parse_document_command(Operation::Info, rest),
        "config" => config_command::parse_config_command(rest),
        "init" => nullary::parse_nullary_command(CliCommand::Init, "init", rest),
        "doctor" => nullary::parse_nullary_command(CliCommand::Doctor, "doctor", rest),
        "version" => nullary::parse_nullary_command(CliCommand::Version, "version", rest),
        _ => Err(AppError::invalid_request(
            "command",
            format!("unknown command {command:?}"),
        )),
    }
}
