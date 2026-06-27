use docnav_cli_args::{scan_loose_args, LooseArgScan};

use crate::error::{AppError, AppResult};

use super::super::command_model::{CliCommand, ParsedCli};
use super::argument_helpers::{
    clap_argv, loose_value_flags, missing_value_error, warning_from_ignored_arg,
};
use super::utility_clap_command;

pub(super) fn parse_utility_command(
    command: CliCommand,
    label: &'static str,
    args: &[String],
) -> AppResult<ParsedCli> {
    let about = match label {
        super::command_names::INIT => "Initialize .docnav project configuration",
        super::command_names::DOCTOR => "Check Docnav project and adapter health",
        super::command_names::VERSION => "Print docnav version",
        _ => "Docnav command",
    };
    let known_value_flags = loose_value_flags(|_flag| false);
    let scan = scan_loose_args(args, &LooseArgScan::new(label, 0, &known_value_flags))
        .map_err(missing_value_error)?;
    let warnings = scan
        .ignored
        .into_iter()
        .map(warning_from_ignored_arg)
        .collect();

    utility_clap_command(label, about)
        .try_get_matches_from(clap_argv(label, Vec::new()))
        .map_err(|_| AppError::invalid_request(label, "invalid command line arguments"))?;

    Ok(ParsedCli::new(command, warnings))
}
