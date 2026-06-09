use crate::error::{AppError, AppResult};

use super::super::flags;
use super::super::types::{CliCommand, ParsedCli};
use super::super::warning::CliWarning;
use super::common::{clap_argv, is_flag, known_value_flag, split_equals, warning_value};
use super::nullary_clap_command;

pub(super) fn parse_nullary_command(
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
    let mut warnings = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if let Some(_flag) = known_value_flag(token) {
            let (flag_token, _inline_value) = split_equals(token);
            let value = warning_value(args, &mut index, flag_token)?;
            warnings.push(CliWarning::unused_operation_flag(token, value, label));
        } else if token == flags::USER || is_flag(token) {
            warnings.push(CliWarning::unknown_flag(token));
            index += 1;
        } else {
            warnings.push(CliWarning::extra_positional(token));
            index += 1;
        }
    }

    nullary_clap_command(label, about)
        .try_get_matches_from(clap_argv(label, Vec::new()))
        .map_err(|_| AppError::invalid_request(label, "invalid command line arguments"))?;

    Ok(ParsedCli { command, warnings })
}
