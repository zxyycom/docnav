use docnav_cli_args::{scan_arg_boundaries, ArgBoundaryScan};

use crate::error::{AppError, AppResult};

use super::super::command_model::{CliCommand, ConfigCommand, ConfigInspect, ParsedCli};
use super::super::flags;
use super::argument_helpers::{
    boundary_value_flags, clap_argv, config_path_args, error_from_rejected_arg,
    missing_value_error as scan_missing_value_error, missing_value_flag_error, split_equals,
    ValueFlag,
};
use super::{command_names, config_inspect_command};

pub(super) fn parse_config_command(args: &[String]) -> AppResult<ParsedCli> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(AppError::invalid_request(
            "config",
            "missing config subcommand",
        ));
    };

    match subcommand.as_str() {
        command_names::CONFIG_INSPECT => parse_config_inspect(rest),
        _ => Err(AppError::invalid_request(
            "config",
            format!("unknown config subcommand {subcommand:?}"),
        )),
    }
}

fn parse_config_inspect(args: &[String]) -> AppResult<ParsedCli> {
    let parsed = collect_config_args("config inspect", args)?;
    let matches = config_inspect_command()
        .try_get_matches_from(clap_argv(command_names::CONFIG_INSPECT, parsed.clap_args))
        .map_err(|_| config_inspect_error(args))?;
    Ok(ParsedCli::new(CliCommand::Config(ConfigCommand::Inspect(
        ConfigInspect {
            config_paths: config_path_args(&matches),
        },
    ))))
}

struct ParsedConfigCommon {
    clap_args: Vec<String>,
}

fn collect_config_args(command: &str, args: &[String]) -> AppResult<ParsedConfigCommon> {
    let known_value_flags = boundary_value_flags(|flag| {
        matches!(flag, ValueFlag::ProjectConfig | ValueFlag::UserConfig)
    });
    let scan = scan_arg_boundaries(args, &ArgBoundaryScan::new(command, 0, &known_value_flags))
        .map_err(scan_missing_value_error)?;

    if let Some(rejected) = scan.rejected.into_iter().next() {
        return Err(error_from_rejected_arg(rejected));
    }

    Ok(ParsedConfigCommon {
        clap_args: scan.retained_args,
    })
}

fn config_inspect_error(args: &[String]) -> AppError {
    first_missing_value_error(args)
        .unwrap_or_else(|| AppError::invalid_request("config inspect", "invalid arguments"))
}

fn first_missing_value_error(args: &[String]) -> Option<AppError> {
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if matches!(
            split_equals(token).0,
            flags::PROJECT_CONFIG | flags::USER_CONFIG
        ) {
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
