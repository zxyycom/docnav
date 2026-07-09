mod adapter_command;
mod argument_helpers;
mod config_command;
mod document;
mod native_options;
mod spec;
mod utility_command;

use docnav_protocol::Operation;

use crate::error::{AppError, AppResult};

use super::command_model::{CliCommand, ConfigPathArgs, ParsedCli};

use native_options::NativeOptionCatalog;
use spec::{cli_command, is_known_root_command};

pub(super) use spec::{
    arg_ids, command_names, config_inspect_command, document_clap_command, utility_clap_command,
};

#[derive(Clone, Debug, Default)]
pub(super) struct ParserContext {
    native_options: NativeOptionCatalog,
}

impl ParserContext {
    fn native_options(&self) -> &NativeOptionCatalog {
        &self.native_options
    }
}

pub fn parse<I, S>(args: I) -> AppResult<ParsedCli>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let context = ParserContext::default();
    parse_with_context(args, &context)
}

fn parse_with_context<I, S>(args: I, context: &ParserContext) -> AppResult<ParsedCli>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    if let Some(help) = help_text(&args, context) {
        return Ok(ParsedCli::new(CliCommand::Help(help)));
    }

    let Some((command, rest)) = args.split_first() else {
        return Err(AppError::invalid_request("command", "missing command"));
    };

    if !is_known_root_command(command, context) {
        return Err(AppError::invalid_request(
            "command",
            format!("unknown command {command:?}"),
        ));
    }

    match command.as_str() {
        command_names::OUTLINE => {
            document::parse_document_command(Operation::Outline, rest, context)
        }
        command_names::READ => document::parse_document_command(Operation::Read, rest, context),
        command_names::FIND => document::parse_document_command(Operation::Find, rest, context),
        command_names::INFO => document::parse_document_command(Operation::Info, rest, context),
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

fn help_text(args: &[String], context: &ParserContext) -> Option<String> {
    if !args.iter().any(|arg| is_help_flag(arg)) {
        return None;
    }
    let mut root = cli_command(context);
    let Some(first) = args.first().map(String::as_str) else {
        return Some(root.render_long_help().to_string());
    };
    if is_help_flag(first) {
        return Some(root.render_long_help().to_string());
    }
    let Some(command) = root.find_subcommand_mut(first) else {
        return Some(root.render_long_help().to_string());
    };
    if let Some(second) = nested_help_subcommand(first, args) {
        if let Some(subcommand) = command.find_subcommand_mut(second) {
            return Some(subcommand.render_long_help().to_string());
        }
        return None;
    }
    Some(command.render_long_help().to_string())
}

fn nested_help_subcommand<'a>(first: &str, args: &'a [String]) -> Option<&'a str> {
    if !matches!(first, command_names::CONFIG | command_names::ADAPTER) {
        return None;
    }
    let second = args.get(1).map(String::as_str)?;
    (!is_help_flag(second)).then_some(second)
}

fn is_help_flag(arg: &str) -> bool {
    arg == "--help" || arg == "-h"
}

#[cfg(test)]
mod tests;
