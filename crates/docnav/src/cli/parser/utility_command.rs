use docnav_cli_args::{scan_arg_boundaries, ArgBoundaryScan};

use crate::error::{AppError, AppResult};

use super::super::command_model::{CliCommand, ParsedCli};
use super::argument_helpers::{
    boundary_value_flags, clap_argv, config_path_args, error_from_rejected_arg,
    missing_value_error, project_config_path_args, ValueFlag,
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
    let known_value_flags = boundary_value_flags(|flag| {
        matches!(
            (label, flag),
            (super::command_names::INIT, ValueFlag::ProjectConfig)
                | (super::command_names::DOCTOR, ValueFlag::ProjectConfig)
                | (super::command_names::DOCTOR, ValueFlag::UserConfig)
        )
    });
    let scan = scan_arg_boundaries(args, &ArgBoundaryScan::new(label, 0, &known_value_flags))
        .map_err(missing_value_error)?;
    if let Some(rejected) = scan.rejected.into_iter().next() {
        return Err(error_from_rejected_arg(rejected));
    }

    let matches = utility_clap_command(label, about)
        .try_get_matches_from(clap_argv(label, scan.retained_args))
        .map_err(|_| AppError::invalid_request(label, "invalid command line arguments"))?;

    let command = match command {
        CliCommand::Init(_) => CliCommand::Init(project_config_path_args(&matches)),
        CliCommand::Doctor(_) => CliCommand::Doctor(config_path_args(&matches)),
        command => command,
    };

    Ok(ParsedCli::new(command))
}
