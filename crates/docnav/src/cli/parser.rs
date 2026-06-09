mod common;
mod config_command;
mod document;
mod nullary;

use clap::builder::NonEmptyStringValueParser;
use clap::{Arg, ArgAction, Command};
use docnav_protocol::Operation;

use crate::error::{AppError, AppResult};

use super::types::{CliCommand, ParsedCli};

pub(super) mod command_names {
    pub(super) const CONFIG: &str = "config";
    pub(super) const CONFIG_GET: &str = "get";
    pub(super) const CONFIG_LIST: &str = "list";
    pub(super) const CONFIG_SET: &str = "set";
    pub(super) const CONFIG_UNSET: &str = "unset";
    pub(super) const DOCTOR: &str = "doctor";
    pub(super) const FIND: &str = "find";
    pub(super) const INFO: &str = "info";
    pub(super) const INIT: &str = "init";
    pub(super) const OUTLINE: &str = "outline";
    pub(super) const READ: &str = "read";
    pub(super) const VERSION: &str = "version";
}

pub(super) mod arg_ids {
    pub(super) const ADAPTER: &str = "adapter";
    pub(super) const KEY: &str = "key";
    pub(super) const LIMIT_CHARS: &str = "limit_chars";
    pub(super) const OPERATION: &str = "operation";
    pub(super) const OUTPUT: &str = "output";
    pub(super) const PAGE: &str = "page";
    pub(super) const PATH: &str = "path";
    pub(super) const QUERY: &str = "query";
    pub(super) const REF: &str = "ref";
    pub(super) const USER: &str = "user";
    pub(super) const VALUE: &str = "value";
}

mod defaults {
    pub(super) const LIMIT_CHARS: &str = "6000";
    pub(super) const OUTPUT: &str = super::output_values::TEXT;
    pub(super) const PAGE: &str = "1";
}

pub(super) mod output_values {
    pub(super) const PROTOCOL_JSON: &str = "protocol-json";
    pub(super) const READABLE_JSON: &str = "readable-json";
    pub(super) const TEXT: &str = "text";
}

pub(super) mod operation_values {
    pub(super) const FIND: &str = "find";
    pub(super) const INFO: &str = "info";
    pub(super) const OUTLINE: &str = "outline";
    pub(super) const READ: &str = "read";
}

pub fn parse<I, S>(args: I) -> AppResult<ParsedCli>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    if let Some(help) = help_text(&args) {
        return Ok(ParsedCli {
            command: CliCommand::Help(help),
            warnings: Vec::new(),
        });
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
            nullary::parse_nullary_command(CliCommand::Init, command_names::INIT, rest)
        }
        command_names::DOCTOR => {
            nullary::parse_nullary_command(CliCommand::Doctor, command_names::DOCTOR, rest)
        }
        command_names::VERSION => {
            nullary::parse_nullary_command(CliCommand::Version, command_names::VERSION, rest)
        }
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

fn is_known_root_command(command: &str) -> bool {
    cli_command().find_subcommand(command).is_some()
}

fn cli_command() -> Command {
    Command::new("docnav")
        .about("Structured document navigation CLI")
        .disable_help_subcommand(true)
        .subcommand(document_clap_command(Operation::Outline))
        .subcommand(document_clap_command(Operation::Read))
        .subcommand(document_clap_command(Operation::Find))
        .subcommand(document_clap_command(Operation::Info))
        .subcommand(config_command())
        .subcommand(nullary_clap_command(
            command_names::INIT,
            "Initialize .docnav project configuration",
        ))
        .subcommand(nullary_clap_command(
            command_names::DOCTOR,
            "Check Docnav project and adapter health",
        ))
        .subcommand(nullary_clap_command(
            command_names::VERSION,
            "Print docnav version",
        ))
}

pub(super) fn document_clap_command(operation: Operation) -> Command {
    match operation {
        Operation::Outline => paged_document_command(
            command_names::OUTLINE,
            "Return compact document outline entries",
        ),
        Operation::Read => {
            paged_document_command(command_names::READ, "Read a document region by adapter ref")
                .arg(ref_arg())
        }
        Operation::Find => {
            paged_document_command(command_names::FIND, "Find matching document regions")
                .arg(query_arg())
        }
        Operation::Info => document_command(command_names::INFO, "Return adapter document summary"),
    }
}

fn document_command(name: &'static str, about: &'static str) -> Command {
    Command::new(name)
        .about(about)
        .arg(path_arg())
        .arg(adapter_arg())
        .arg(output_arg())
}

fn paged_document_command(name: &'static str, about: &'static str) -> Command {
    document_command(name, about)
        .arg(page_arg())
        .arg(limit_chars_arg())
}

fn config_command() -> Command {
    Command::new(command_names::CONFIG)
        .about("Read and write docnav configuration")
        .subcommand(config_get_command())
        .subcommand(config_set_command())
        .subcommand(config_unset_command())
        .subcommand(config_list_command())
}

pub(super) fn config_get_command() -> Command {
    Command::new(command_names::CONFIG_GET)
        .about("Read an effective configuration key")
        .arg(key_arg())
        .arg(user_arg())
}

pub(super) fn config_set_command() -> Command {
    Command::new(command_names::CONFIG_SET)
        .about("Set a project or user configuration key")
        .arg(key_arg())
        .arg(positional_value_arg(arg_ids::VALUE, "value"))
        .arg(user_arg())
}

pub(super) fn config_unset_command() -> Command {
    Command::new(command_names::CONFIG_UNSET)
        .about("Remove a project or user configuration key")
        .arg(key_arg())
        .arg(user_arg())
}

pub(super) fn config_list_command() -> Command {
    Command::new(command_names::CONFIG_LIST)
        .about("List effective configuration")
        .arg(user_arg())
        .arg(value_arg(arg_ids::PATH, "path", "path"))
        .arg(operation_arg())
}

pub(super) fn nullary_clap_command(name: &'static str, about: &'static str) -> Command {
    Command::new(name).about(about)
}

fn path_arg() -> Arg {
    Arg::new(arg_ids::PATH)
        .value_name("path")
        .required(true)
        .value_parser(NonEmptyStringValueParser::new())
}

fn key_arg() -> Arg {
    positional_value_arg(arg_ids::KEY, "key")
}

fn adapter_arg() -> Arg {
    value_arg(arg_ids::ADAPTER, "adapter", "adapter-id")
}

fn page_arg() -> Arg {
    value_arg(arg_ids::PAGE, "page", "positive integer")
        .default_value(defaults::PAGE)
        .value_parser(clap::value_parser!(u32))
}

fn limit_chars_arg() -> Arg {
    value_arg(arg_ids::LIMIT_CHARS, "limit-chars", "positive integer")
        .default_value(defaults::LIMIT_CHARS)
        .value_parser(clap::value_parser!(u32))
}

fn output_arg() -> Arg {
    value_arg(
        arg_ids::OUTPUT,
        "output",
        "text|readable-json|protocol-json",
    )
    .default_value(defaults::OUTPUT)
    .value_parser([
        output_values::TEXT,
        output_values::READABLE_JSON,
        output_values::PROTOCOL_JSON,
    ])
}

fn query_arg() -> Arg {
    value_arg(arg_ids::QUERY, "query", "text").required(true)
}

fn ref_arg() -> Arg {
    value_arg(arg_ids::REF, "ref", "ref").required(true)
}

fn operation_arg() -> Arg {
    value_arg(arg_ids::OPERATION, "operation", "outline|read|find|info").value_parser([
        operation_values::OUTLINE,
        operation_values::READ,
        operation_values::FIND,
        operation_values::INFO,
    ])
}

fn user_arg() -> Arg {
    Arg::new(arg_ids::USER)
        .long("user")
        .action(ArgAction::SetTrue)
}

fn value_arg(id: &'static str, long: &'static str, value_name: &'static str) -> Arg {
    Arg::new(id)
        .long(long)
        .value_name(value_name)
        .num_args(1)
        .allow_hyphen_values(true)
        .value_parser(NonEmptyStringValueParser::new())
}

fn positional_value_arg(id: &'static str, value_name: &'static str) -> Arg {
    Arg::new(id)
        .value_name(value_name)
        .required(true)
        .value_parser(NonEmptyStringValueParser::new())
}

#[cfg(test)]
mod tests {
    use super::super::warning::{CliWarningDetails, CliWarningEffect, CliWarningId};
    use super::*;
    use crate::cli::{CliCommand, OutputMode};

    #[test]
    fn help_returns_typed_help_command() {
        let parsed = parse(["outline", "--help"]).expect("parse help");

        assert!(parsed.warnings.is_empty());
        match parsed.command {
            CliCommand::Help(text) => {
                assert!(text.contains("Usage:"));
                assert!(text.contains("--output"));
                assert!(text.contains("outline"));
            }
            command => panic!("expected help command, got {command:?}"),
        }
    }

    #[test]
    fn used_known_argument_stays_strict() {
        let error = parse(["outline", "doc.md", "--page", "0"]).expect_err("page is invalid");

        assert_eq!(
            error
                .error()
                .details
                .get("field")
                .and_then(serde_json::Value::as_str),
            Some("--page")
        );
        assert!(error
            .error()
            .details
            .get("reason")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|reason| reason.contains("positive integer")));
    }

    #[test]
    fn unused_known_argument_value_is_not_eagerly_typed() {
        let parsed = parse([
            "info",
            "doc.md",
            "--page",
            "nope",
            "--output",
            "readable-json",
        ])
        .expect("unused page should not fail info");

        match parsed.command {
            CliCommand::Document(command) => {
                assert_eq!(command.operation, Operation::Info);
                assert_eq!(command.output, Some(OutputMode::ReadableJson));
                assert!(command.page.is_none());
                assert!(command.limit_chars.is_none());
            }
            command => panic!("expected document command, got {command:?}"),
        }
        assert_eq!(parsed.warnings.len(), 1);
        let warning = &parsed.warnings[0];
        assert_eq!(warning.id, CliWarningId::CliArgvIgnored);
        assert_eq!(warning.effect, CliWarningEffect::OperationContinued);
        match &warning.details {
            CliWarningDetails::CliArgv { tokens } => {
                assert!(tokens.contains(&"--page".to_owned()));
                assert!(tokens.contains(&"nope".to_owned()));
            }
            details => panic!("expected CLI argv details, got {details:?}"),
        }
    }
}
