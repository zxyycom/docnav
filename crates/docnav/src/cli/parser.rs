mod common;
mod config_command;
mod document;
mod nullary;

use clap::{Arg, ArgAction, Command};
use docnav_protocol::Operation;

use crate::error::{AppError, AppResult};

use super::types::{CliCommand, ParsedCli};

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
        "outline" => document::parse_document_command(Operation::Outline, rest),
        "read" => document::parse_document_command(Operation::Read, rest),
        "find" => document::parse_document_command(Operation::Find, rest),
        "info" => document::parse_document_command(Operation::Info, rest),
        "config" => config_command::parse_config_command(rest),
        "init" => nullary::parse_nullary_command(CliCommand::Init, "init", rest),
        "doctor" => nullary::parse_nullary_command(CliCommand::Doctor, "doctor", rest),
        "version" => nullary::parse_nullary_command(CliCommand::Version, "version", rest),
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
    if first == "config" {
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
        .subcommand(paged_document_command(
            "outline",
            "Return compact document outline entries",
        ))
        .subcommand(
            paged_document_command("read", "Read a document region by adapter ref").arg(ref_arg()),
        )
        .subcommand(
            paged_document_command("find", "Find matching document regions").arg(query_arg()),
        )
        .subcommand(document_command("info", "Return adapter document summary"))
        .subcommand(config_command())
        .subcommand(Command::new("init").about("Initialize .docnav project configuration"))
        .subcommand(Command::new("doctor").about("Check Docnav project and adapter health"))
        .subcommand(Command::new("version").about("Print docnav version"))
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
    Command::new("config")
        .about("Read and write docnav configuration")
        .subcommand(
            Command::new("get")
                .about("Read an effective configuration key")
                .arg(key_arg())
                .arg(user_arg()),
        )
        .subcommand(
            Command::new("set")
                .about("Set a project or user configuration key")
                .arg(key_arg())
                .arg(Arg::new("value").value_name("value"))
                .arg(user_arg()),
        )
        .subcommand(
            Command::new("unset")
                .about("Remove a project or user configuration key")
                .arg(key_arg())
                .arg(user_arg()),
        )
        .subcommand(
            Command::new("list")
                .about("List effective configuration")
                .arg(user_arg())
                .arg(value_arg("path", "path", "path"))
                .arg(value_arg(
                    "operation",
                    "operation",
                    "outline|read|find|info",
                )),
        )
}

fn path_arg() -> Arg {
    Arg::new("path").value_name("path")
}

fn key_arg() -> Arg {
    Arg::new("key").value_name("key")
}

fn adapter_arg() -> Arg {
    value_arg("adapter", "adapter", "adapter-id")
}

fn page_arg() -> Arg {
    value_arg("page", "page", "positive integer")
}

fn limit_chars_arg() -> Arg {
    value_arg("limit_chars", "limit-chars", "positive integer")
}

fn output_arg() -> Arg {
    value_arg("output", "output", "text|readable-json|protocol-json")
}

fn query_arg() -> Arg {
    value_arg("query", "query", "text")
}

fn ref_arg() -> Arg {
    value_arg("ref", "ref", "ref")
}

fn user_arg() -> Arg {
    Arg::new("user").long("user").action(ArgAction::SetTrue)
}

fn value_arg(id: &'static str, long: &'static str, value_name: &'static str) -> Arg {
    Arg::new(id)
        .long(long)
        .value_name(value_name)
        .num_args(1)
        .allow_hyphen_values(true)
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
