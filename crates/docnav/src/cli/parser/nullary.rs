use crate::error::{AppError, AppResult};

use super::super::flags;
use super::super::types::{CliCommand, ParsedCli};
use super::super::warning::CliWarning;
use super::common::{is_flag, known_value_flag};

pub(super) fn parse_nullary_command(
    command: CliCommand,
    label: &str,
    args: &[String],
) -> AppResult<ParsedCli> {
    let mut warnings = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if let Some(_flag) = known_value_flag(token) {
            let value = args
                .get(index + 1)
                .ok_or_else(|| AppError::invalid_request(token, "flag requires a value"))?;
            warnings.push(CliWarning::unused_operation_flag(token, Some(value), label));
            index += 2;
        } else if token == flags::USER || is_flag(token) {
            warnings.push(CliWarning::unknown_flag(token));
            index += 1;
        } else {
            warnings.push(CliWarning::extra_positional(token));
            index += 1;
        }
    }

    Ok(ParsedCli { command, warnings })
}
