use std::num::NonZeroU32;

use clap::parser::{ArgMatches, ValueSource};
use docnav_cli_args::{KnownValueFlag as BoundaryKnownValueFlag, MissingValue, RejectedArg};
use docnav_protocol::{Operation, PositiveInteger};

use crate::error::{AppError, AppResult};

use super::super::command_model::OutputMode;
use super::super::flags;
use super::arg_ids;

#[derive(Clone, Copy)]
pub(super) enum ValueFlag {
    Adapter,
    Limit,
    MaxHeadingLevel,
    Operation,
    Output,
    Page,
    Pagination,
    Path,
    Query,
    Ref,
}

const VALUE_FLAGS: &[(&str, ValueFlag)] = &[
    (flags::ADAPTER, ValueFlag::Adapter),
    (flags::LIMIT, ValueFlag::Limit),
    (flags::MAX_HEADING_LEVEL, ValueFlag::MaxHeadingLevel),
    (flags::OPERATION, ValueFlag::Operation),
    (flags::OUTPUT, ValueFlag::Output),
    (flags::PAGE, ValueFlag::Page),
    (flags::PAGINATION, ValueFlag::Pagination),
    (flags::PATH, ValueFlag::Path),
    (flags::QUERY, ValueFlag::Query),
    (flags::REF, ValueFlag::Ref),
];

pub(super) fn boundary_value_flags(
    uses_flag: impl Fn(ValueFlag) -> bool,
) -> Vec<BoundaryKnownValueFlag<'static>> {
    VALUE_FLAGS
        .iter()
        .map(|(flag, value_flag)| BoundaryKnownValueFlag {
            flag,
            used: uses_flag(*value_flag),
        })
        .collect()
}

pub(super) fn known_value_flag(token: &str) -> Option<ValueFlag> {
    let (flag, _value) = split_equals(token);
    match flag {
        flags::ADAPTER => Some(ValueFlag::Adapter),
        flags::LIMIT => Some(ValueFlag::Limit),
        flags::MAX_HEADING_LEVEL => Some(ValueFlag::MaxHeadingLevel),
        flags::OPERATION => Some(ValueFlag::Operation),
        flags::OUTPUT => Some(ValueFlag::Output),
        flags::PAGE => Some(ValueFlag::Page),
        flags::PAGINATION => Some(ValueFlag::Pagination),
        flags::PATH => Some(ValueFlag::Path),
        flags::QUERY => Some(ValueFlag::Query),
        flags::REF => Some(ValueFlag::Ref),
        _ => None,
    }
}

pub(super) fn error_from_rejected_arg(rejected: RejectedArg) -> AppError {
    match rejected {
        RejectedArg::UnknownFlag { token } => {
            AppError::invalid_request("argv", format!("unknown argument {token}"))
        }
        RejectedArg::ExtraPositional { token } => {
            AppError::invalid_request("argv", format!("extra positional argument {token}"))
        }
        RejectedArg::UnusedValueFlag {
            flag,
            value,
            command,
        } => {
            let value = value.map_or(String::new(), |value| format!(" {value}"));
            let reason = format!("{flag}{value} is not used by {command} command");
            AppError::invalid_request(flag, reason)
        }
    }
}

pub(super) fn missing_value_error(error: MissingValue) -> AppError {
    AppError::invalid_request(error.flag(), "flag requires a value")
}

pub(super) fn is_flag(token: &str) -> bool {
    token.starts_with("--")
}

pub(super) fn split_equals(token: &str) -> (&str, Option<&str>) {
    token
        .split_once('=')
        .map_or((token, None), |(flag, value)| (flag, Some(value)))
}

pub(super) fn clap_argv(command: &str, args: Vec<String>) -> Vec<String> {
    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push(command.to_owned());
    argv.extend(args);
    argv
}

pub(super) fn is_command_line(matches: &ArgMatches, id: &str) -> bool {
    matches.value_source(id) == Some(ValueSource::CommandLine)
}

pub(super) fn required_string(matches: &ArgMatches, id: &str, field: &str) -> AppResult<String> {
    matches
        .get_one::<String>(id)
        .cloned()
        .ok_or_else(|| AppError::invalid_request(field, format!("missing {field}")))
}

pub(super) fn optional_explicit_string(matches: &ArgMatches, id: &str) -> Option<String> {
    is_command_line(matches, id)
        .then(|| matches.get_one::<String>(id).cloned())
        .flatten()
}

pub(super) fn optional_explicit_output(matches: &ArgMatches) -> AppResult<Option<OutputMode>> {
    optional_explicit_string(matches, arg_ids::OUTPUT)
        .map(|value| {
            value
                .parse()
                .map_err(|reason: String| AppError::invalid_request(flags::OUTPUT, reason))
        })
        .transpose()
}

pub(super) fn optional_explicit_positive(
    matches: &ArgMatches,
    id: &str,
    flag: &str,
) -> AppResult<Option<PositiveInteger>> {
    if !is_command_line(matches, id) {
        return Ok(None);
    }
    let parsed = matches
        .get_one::<u32>(id)
        .copied()
        .ok_or_else(|| AppError::invalid_request(flag, "flag requires a value"))?;
    positive_from_u32(parsed, flag).map(Some)
}

pub(super) fn positive_from_u32(value: u32, flag: &str) -> AppResult<PositiveInteger> {
    let parsed = value;
    NonZeroU32::new(parsed).ok_or_else(|| {
        AppError::invalid_request(flag, format!("{flag} must be a positive integer"))
    })
}

pub(super) fn parse_operation(value: &str) -> AppResult<Operation> {
    value.parse().map_err(|_| {
        AppError::invalid_request("--operation", "expected outline, read, find, or info")
    })
}
