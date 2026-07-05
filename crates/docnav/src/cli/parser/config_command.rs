use docnav_cli_args::{scan_arg_boundaries, ArgBoundaryScan};

use crate::error::{AppError, AppResult};

use super::super::command_model::{
    CliCommand, ConfigCommand, ConfigGet, ConfigList, ConfigSet, ConfigUnset, ParsedCli,
};
use super::super::flags;
use super::argument_helpers::{
    boundary_value_flags, clap_argv, config_path_args, error_from_rejected_arg, is_flag,
    known_value_flag, missing_value_error as scan_missing_value_error, missing_value_flag_error,
    optional_explicit_string, parse_operation, required_string, split_equals, ValueFlag,
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
    Ok(ParsedCli::new(CliCommand::Config(ConfigCommand::Get(
        ConfigGet {
            key,
            user: matches.get_flag(arg_ids::USER),
            config_paths: config_path_args(&matches),
        },
    ))))
}

fn parse_config_set(args: &[String]) -> AppResult<ParsedCli> {
    let parsed = collect_config_args("config set", args, ConfigFlagUsage::KeyOnly, 2)?;
    let matches = config_set_command()
        .try_get_matches_from(clap_argv(command_names::CONFIG_SET, parsed.clap_args))
        .map_err(|_| config_set_error(args))?;
    let key = required_string(&matches, arg_ids::KEY, "key")?;
    let value = required_string(&matches, arg_ids::VALUE, "value")?;
    Ok(ParsedCli::new(CliCommand::Config(ConfigCommand::Set(
        ConfigSet {
            key,
            value,
            user: matches.get_flag(arg_ids::USER),
            config_paths: config_path_args(&matches),
        },
    ))))
}

fn parse_config_unset(args: &[String]) -> AppResult<ParsedCli> {
    let parsed = collect_config_args("config unset", args, ConfigFlagUsage::KeyOnly, 1)?;
    let matches = config_unset_command()
        .try_get_matches_from(clap_argv(command_names::CONFIG_UNSET, parsed.clap_args))
        .map_err(|_| config_unset_error(args))?;
    let key = required_string(&matches, arg_ids::KEY, "key")?;
    Ok(ParsedCli::new(CliCommand::Config(ConfigCommand::Unset(
        ConfigUnset {
            key,
            user: matches.get_flag(arg_ids::USER),
            config_paths: config_path_args(&matches),
        },
    ))))
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

    Ok(ParsedCli::new(CliCommand::Config(ConfigCommand::List(
        ConfigList {
            user: matches.get_flag(arg_ids::USER),
            path,
            operation,
            config_paths: config_path_args(&matches),
        },
    ))))
}

#[derive(Clone, Copy)]
enum ConfigFlagUsage {
    KeyOnly,
    PathContext,
}

struct ParsedConfigCommon {
    clap_args: Vec<String>,
}

fn collect_config_args(
    command: &str,
    args: &[String],
    usage: ConfigFlagUsage,
    expected_positionals: usize,
) -> AppResult<ParsedConfigCommon> {
    let list_has_path = matches!(usage, ConfigFlagUsage::PathContext) && has_path_flag(args);
    let known_value_flags = boundary_value_flags(|flag| match flag {
        ValueFlag::ProjectConfig | ValueFlag::UserConfig => true,
        ValueFlag::Path | ValueFlag::Operation => {
            matches!(usage, ConfigFlagUsage::PathContext) && list_has_path
        }
        _ => false,
    });
    let scan = scan_arg_boundaries(
        args,
        &ArgBoundaryScan::new(command, expected_positionals, &known_value_flags)
            .with_known_switch_flags(&[flags::USER]),
    )
    .map_err(scan_missing_value_error)?;

    if let Some(rejected) = scan.rejected.into_iter().next() {
        return Err(error_from_rejected_arg(rejected));
    }

    Ok(ParsedConfigCommon {
        clap_args: scan.retained_args,
    })
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
    first_missing_value_error(args)
        .unwrap_or_else(|| AppError::invalid_request("key", "config get requires <key>"))
}

fn config_set_error(args: &[String]) -> AppError {
    first_missing_value_error(args).unwrap_or_else(|| {
        if args.iter().any(|token| !is_flag(token)) {
            AppError::invalid_request("value", "config set requires <value>")
        } else {
            AppError::invalid_request("key", "config set requires <key>")
        }
    })
}

fn config_unset_error(args: &[String]) -> AppError {
    first_missing_value_error(args)
        .unwrap_or_else(|| AppError::invalid_request("key", "config unset requires <key>"))
}

fn config_list_error(args: &[String]) -> AppError {
    first_missing_value_error(args).unwrap_or_else(|| {
        invalid_operation_error(args).unwrap_or_else(|| {
            AppError::invalid_request("config list", "invalid config list arguments")
        })
    })
}

fn first_missing_value_error(args: &[String]) -> Option<AppError> {
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if known_value_flag(token).is_some() {
            let (flag, inline_value) = split_equals(token);
            if inline_value.is_none() && args.get(index + 1).is_none() {
                return Some(missing_value_flag_error(flag));
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
            if let Err(error) = parse_operation(value) {
                return Some(error);
            }
            index += if inline_value.is_some() { 1 } else { 2 };
        } else {
            index += 1;
        }
    }
    None
}
