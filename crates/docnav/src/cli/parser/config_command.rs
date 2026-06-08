use crate::error::{AppError, AppResult};

use super::super::flags;
use super::super::types::{
    CliCommand, ConfigCommand, ConfigGet, ConfigList, ConfigSet, ConfigUnset, ParsedCli,
};
use super::super::warning::CliWarning;
use super::common::{is_flag, known_value_flag, parse_operation, required_positional, ValueFlag};

pub(super) fn parse_config_command(args: &[String]) -> AppResult<ParsedCli> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(AppError::invalid_request(
            "config",
            "missing config subcommand",
        ));
    };

    match subcommand.as_str() {
        "get" => parse_config_get(rest),
        "set" => parse_config_set(rest),
        "unset" => parse_config_unset(rest),
        "list" => parse_config_list(rest),
        _ => Err(AppError::invalid_request(
            "config",
            format!("unknown config subcommand {subcommand:?}"),
        )),
    }
}

fn parse_config_get(args: &[String]) -> AppResult<ParsedCli> {
    let parsed = parse_config_common("config get", args, ConfigFlagUsage::KeyOnly, 1)?;
    let key = required_positional(&parsed.positionals, "key", "config get requires <key>")?;
    Ok(ParsedCli {
        command: CliCommand::Config(ConfigCommand::Get(ConfigGet {
            key,
            user: parsed.user,
        })),
        warnings: parsed.warnings,
    })
}

fn parse_config_set(args: &[String]) -> AppResult<ParsedCli> {
    let parsed = parse_config_common("config set", args, ConfigFlagUsage::KeyOnly, 2)?;
    let key = required_positional(&parsed.positionals, "key", "config set requires <key>")?;
    let value = parsed
        .positionals
        .get(1)
        .cloned()
        .ok_or_else(|| AppError::invalid_request("value", "config set requires <value>"))?;
    Ok(ParsedCli {
        command: CliCommand::Config(ConfigCommand::Set(ConfigSet {
            key,
            value,
            user: parsed.user,
        })),
        warnings: parsed.warnings,
    })
}

fn parse_config_unset(args: &[String]) -> AppResult<ParsedCli> {
    let parsed = parse_config_common("config unset", args, ConfigFlagUsage::KeyOnly, 1)?;
    let key = required_positional(&parsed.positionals, "key", "config unset requires <key>")?;
    Ok(ParsedCli {
        command: CliCommand::Config(ConfigCommand::Unset(ConfigUnset {
            key,
            user: parsed.user,
        })),
        warnings: parsed.warnings,
    })
}

fn parse_config_list(args: &[String]) -> AppResult<ParsedCli> {
    let mut parsed = parse_config_common("config list", args, ConfigFlagUsage::PathContext, 0)?;
    if parsed.path.is_none() {
        if let Some(raw) = parsed.operation_raw.take() {
            parsed.warnings.push(CliWarning::unused_operation_flag(
                flags::OPERATION,
                Some(&raw),
                "config list",
            ));
        }
    }
    let operation = match parsed.operation_raw {
        Some(raw) => Some(parse_operation(&raw)?),
        None => None,
    };

    Ok(ParsedCli {
        command: CliCommand::Config(ConfigCommand::List(ConfigList {
            user: parsed.user,
            path: parsed.path,
            operation,
        })),
        warnings: parsed.warnings,
    })
}

#[derive(Clone, Copy)]
enum ConfigFlagUsage {
    KeyOnly,
    PathContext,
}

struct ParsedConfigCommon {
    user: bool,
    path: Option<String>,
    operation_raw: Option<String>,
    positionals: Vec<String>,
    warnings: Vec<CliWarning>,
}

fn parse_config_common(
    command: &str,
    args: &[String],
    usage: ConfigFlagUsage,
    expected_positionals: usize,
) -> AppResult<ParsedConfigCommon> {
    let mut parsed = ParsedConfigCommon {
        user: false,
        path: None,
        operation_raw: None,
        positionals: Vec::new(),
        warnings: Vec::new(),
    };

    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if token == flags::USER {
            parsed.user = true;
            index += 1;
        } else if let Some(flag) = known_value_flag(token) {
            let value = args
                .get(index + 1)
                .ok_or_else(|| AppError::invalid_request(token, "flag requires a value"))?;
            match (usage, flag) {
                (ConfigFlagUsage::PathContext, ValueFlag::Path) => {
                    parsed.path = Some(value.clone())
                }
                (ConfigFlagUsage::PathContext, ValueFlag::Operation) => {
                    parsed.operation_raw = Some(value.clone())
                }
                _ => parsed.warnings.push(CliWarning::unused_operation_flag(
                    token,
                    Some(value),
                    command,
                )),
            }
            index += 2;
        } else if is_flag(token) {
            parsed.warnings.push(CliWarning::unknown_flag(token));
            index += 1;
        } else {
            parsed.positionals.push(token.clone());
            index += 1;
        }
    }

    if parsed.positionals.len() > expected_positionals {
        for token in parsed.positionals.drain(expected_positionals..) {
            parsed.warnings.push(CliWarning::extra_positional(&token));
        }
    }

    Ok(parsed)
}
