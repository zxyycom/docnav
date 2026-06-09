use crate::error::{AppError, AppResult};

use super::super::flags;
use super::super::types::{
    CliCommand, ConfigCommand, ConfigGet, ConfigList, ConfigSet, ConfigUnset, ParsedCli,
};
use super::super::warning::CliWarning;
use super::common::{
    clap_argv, is_flag, known_value_flag, optional_explicit_string, parse_operation,
    push_clap_value_arg, required_string, split_equals, warning_value, ValueFlag,
};
use super::{
    arg_ids, command_names, config_get_command, config_list_command, config_set_command,
    config_unset_command,
};

pub(super) fn parse_config_command(args: &[String]) -> AppResult<ParsedCli> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(AppError::invalid_request(
            "config",
            "missing config subcommand",
        ));
    };

    match subcommand.as_str() {
        command_names::CONFIG_GET => parse_config_get(rest),
        command_names::CONFIG_SET => parse_config_set(rest),
        command_names::CONFIG_UNSET => parse_config_unset(rest),
        command_names::CONFIG_LIST => parse_config_list(rest),
        _ => Err(AppError::invalid_request(
            "config",
            format!("unknown config subcommand {subcommand:?}"),
        )),
    }
}

fn parse_config_get(args: &[String]) -> AppResult<ParsedCli> {
    let parsed = collect_config_args("config get", args, ConfigFlagUsage::KeyOnly, 1)?;
    let matches = config_get_command()
        .try_get_matches_from(clap_argv(command_names::CONFIG_GET, parsed.clap_args))
        .map_err(|_| config_get_error(args))?;
    let key = required_string(&matches, arg_ids::KEY, "key")?;
    Ok(ParsedCli {
        command: CliCommand::Config(ConfigCommand::Get(ConfigGet {
            key,
            user: matches.get_flag(arg_ids::USER),
        })),
        warnings: parsed.warnings,
    })
}

fn parse_config_set(args: &[String]) -> AppResult<ParsedCli> {
    let parsed = collect_config_args("config set", args, ConfigFlagUsage::KeyOnly, 2)?;
    let matches = config_set_command()
        .try_get_matches_from(clap_argv(command_names::CONFIG_SET, parsed.clap_args))
        .map_err(|_| config_set_error(args))?;
    let key = required_string(&matches, arg_ids::KEY, "key")?;
    let value = required_string(&matches, arg_ids::VALUE, "value")?;
    Ok(ParsedCli {
        command: CliCommand::Config(ConfigCommand::Set(ConfigSet {
            key,
            value,
            user: matches.get_flag(arg_ids::USER),
        })),
        warnings: parsed.warnings,
    })
}

fn parse_config_unset(args: &[String]) -> AppResult<ParsedCli> {
    let parsed = collect_config_args("config unset", args, ConfigFlagUsage::KeyOnly, 1)?;
    let matches = config_unset_command()
        .try_get_matches_from(clap_argv(command_names::CONFIG_UNSET, parsed.clap_args))
        .map_err(|_| config_unset_error(args))?;
    let key = required_string(&matches, arg_ids::KEY, "key")?;
    Ok(ParsedCli {
        command: CliCommand::Config(ConfigCommand::Unset(ConfigUnset {
            key,
            user: matches.get_flag(arg_ids::USER),
        })),
        warnings: parsed.warnings,
    })
}

fn parse_config_list(args: &[String]) -> AppResult<ParsedCli> {
    let parsed = collect_config_args("config list", args, ConfigFlagUsage::PathContext, 0)?;
    let matches = config_list_command()
        .try_get_matches_from(clap_argv(command_names::CONFIG_LIST, parsed.clap_args))
        .map_err(|_| config_list_error(args))?;
    let path = optional_explicit_string(&matches, arg_ids::PATH);
    let operation = optional_explicit_string(&matches, arg_ids::OPERATION)
        .map(|raw| parse_operation(&raw))
        .transpose()?;

    Ok(ParsedCli {
        command: CliCommand::Config(ConfigCommand::List(ConfigList {
            user: matches.get_flag(arg_ids::USER),
            path,
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
    clap_args: Vec<String>,
    warnings: Vec<CliWarning>,
}

fn collect_config_args(
    command: &str,
    args: &[String],
    usage: ConfigFlagUsage,
    expected_positionals: usize,
) -> AppResult<ParsedConfigCommon> {
    let mut parsed = ParsedConfigCommon {
        clap_args: Vec::new(),
        warnings: Vec::new(),
    };

    let list_has_path = matches!(usage, ConfigFlagUsage::PathContext) && has_path_flag(args);
    let mut positional_count = 0;
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if token == flags::USER {
            parsed.clap_args.push(token.clone());
            index += 1;
        } else if let Some(flag) = known_value_flag(token) {
            match (usage, flag) {
                (ConfigFlagUsage::PathContext, ValueFlag::Path) => {
                    let (flag_token, _inline_value) = split_equals(token);
                    push_clap_value_arg(&mut parsed.clap_args, args, &mut index, flag_token)?;
                }
                (ConfigFlagUsage::PathContext, ValueFlag::Operation) if list_has_path => {
                    let (flag_token, _inline_value) = split_equals(token);
                    push_clap_value_arg(&mut parsed.clap_args, args, &mut index, flag_token)?;
                }
                _ => {
                    let (flag_token, _inline_value) = split_equals(token);
                    let value = warning_value(args, &mut index, flag_token)?;
                    parsed
                        .warnings
                        .push(CliWarning::unused_operation_flag(token, value, command));
                }
            }
        } else if is_flag(token) {
            parsed.warnings.push(CliWarning::unknown_flag(token));
            index += 1;
        } else {
            if positional_count < expected_positionals {
                parsed.clap_args.push(token.clone());
            } else {
                parsed.warnings.push(CliWarning::extra_positional(token));
            }
            positional_count += 1;
            index += 1;
        }
    }

    Ok(parsed)
}

fn has_path_flag(args: &[String]) -> bool {
    args.iter().any(|token| {
        let (flag, value) = split_equals(token);
        flag == flags::PATH && value.is_some()
    }) || args
        .windows(2)
        .any(|window| window.first().is_some_and(|token| token == flags::PATH))
}

fn config_get_error(args: &[String]) -> AppError {
    missing_value_error(args)
        .unwrap_or_else(|| AppError::invalid_request("key", "config get requires <key>"))
}

fn config_set_error(args: &[String]) -> AppError {
    missing_value_error(args).unwrap_or_else(|| {
        if args.iter().any(|token| !is_flag(token)) {
            AppError::invalid_request("value", "config set requires <value>")
        } else {
            AppError::invalid_request("key", "config set requires <key>")
        }
    })
}

fn config_unset_error(args: &[String]) -> AppError {
    missing_value_error(args)
        .unwrap_or_else(|| AppError::invalid_request("key", "config unset requires <key>"))
}

fn config_list_error(args: &[String]) -> AppError {
    missing_value_error(args).unwrap_or_else(|| {
        invalid_operation_error(args).unwrap_or_else(|| {
            AppError::invalid_request("config list", "invalid config list arguments")
        })
    })
}

fn missing_value_error(args: &[String]) -> Option<AppError> {
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if known_value_flag(token).is_some() {
            let (flag, inline_value) = split_equals(token);
            if inline_value.is_none() && args.get(index + 1).is_none() {
                return Some(AppError::invalid_request(flag, "flag requires a value"));
            }
            index += if inline_value.is_some() { 1 } else { 2 };
        } else {
            index += 1;
        }
    }
    None
}

fn invalid_operation_error(args: &[String]) -> Option<AppError> {
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        let (flag, inline_value) = split_equals(token);
        if flag == flags::OPERATION {
            let value = inline_value.or_else(|| args.get(index + 1).map(String::as_str))?;
            if parse_operation(value).is_err() {
                return Some(AppError::invalid_request(
                    flags::OPERATION,
                    "expected outline, read, find, or info",
                ));
            }
            index += if inline_value.is_some() { 1 } else { 2 };
        } else {
            index += 1;
        }
    }
    None
}
