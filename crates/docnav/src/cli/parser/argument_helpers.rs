use std::num::NonZeroU32;

use clap::parser::{ArgMatches, ValueSource};
use docnav_cli_args::{KnownValueFlag as BoundaryKnownValueFlag, MissingValue, RejectedArg};
use docnav_protocol::{Operation, PositiveInteger};

use crate::error::{AppError, AppResult};

use super::super::command_model::{ConfigPathArgs, OutputMode};
use super::super::flags;
use super::arg_ids;

#[derive(Clone, Copy)]
pub(super) enum ValueFlag {
    Adapter,
    Limit,
    Operation,
    Output,
    Page,
    Pagination,
    Path,
    ProjectConfig,
    Query,
    Ref,
    UserConfig,
}

const VALUE_FLAGS: &[(&str, ValueFlag)] = &[
    (flags::ADAPTER, ValueFlag::Adapter),
    (flags::LIMIT, ValueFlag::Limit),
    (flags::OPERATION, ValueFlag::Operation),
    (flags::OUTPUT, ValueFlag::Output),
    (flags::PAGE, ValueFlag::Page),
    (flags::PAGINATION, ValueFlag::Pagination),
    (flags::PATH, ValueFlag::Path),
    (flags::PROJECT_CONFIG, ValueFlag::ProjectConfig),
    (flags::QUERY, ValueFlag::Query),
    (flags::REF, ValueFlag::Ref),
    (flags::USER_CONFIG, ValueFlag::UserConfig),
];

const UNKNOWN_ARGUMENT: &str = "unknown_argument";
const EXTRA_POSITIONAL: &str = "extra_positional";
const UNSUPPORTED_ARGUMENT: &str = "unsupported_argument";
const MISSING_VALUE: &str = "missing_value";
const INVALID_VALUE: &str = "invalid_value";

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
        flags::OPERATION => Some(ValueFlag::Operation),
        flags::OUTPUT => Some(ValueFlag::Output),
        flags::PAGE => Some(ValueFlag::Page),
        flags::PAGINATION => Some(ValueFlag::Pagination),
        flags::PATH => Some(ValueFlag::Path),
        flags::PROJECT_CONFIG => Some(ValueFlag::ProjectConfig),
        flags::QUERY => Some(ValueFlag::Query),
        flags::REF => Some(ValueFlag::Ref),
        flags::USER_CONFIG => Some(ValueFlag::UserConfig),
        _ => None,
    }
}

pub(super) fn error_from_rejected_arg(rejected: RejectedArg) -> AppError {
    match rejected {
        RejectedArg::UnknownFlag { token } => AppError::invalid_request_with_input_context(
            "argv",
            UNKNOWN_ARGUMENT,
            Some(token),
            ["supported option or positional argument for the selected command"],
            ["Remove the unknown argument or replace it with an option listed by --help."],
        ),
        RejectedArg::ExtraPositional { token } => AppError::invalid_request_with_input_context(
            "argv",
            EXTRA_POSITIONAL,
            Some(token),
            ["only the positional arguments defined by the selected command"],
            ["Remove the extra positional argument or pass the value with the option that owns it."],
        ),
        RejectedArg::UnusedValueFlag {
            flag,
            value,
            command,
        } => {
            let (flag_token, _inline_value) = split_equals(&flag);
            let received = received_value_flag(&flag, value.as_deref());
            AppError::invalid_request_with_input_context(
                flag_token.to_owned(),
                UNSUPPORTED_ARGUMENT,
                Some(received),
                unsupported_argument_expected(&command, flag_token),
                [format!(
                    "Remove {flag_token} for {command}, or use a command that supports {flag_token}."
                )],
            )
        }
    }
}

pub(super) fn missing_value_error(error: MissingValue) -> AppError {
    missing_value_flag_error(error.flag())
}

pub(super) fn missing_value_flag_error(flag: &str) -> AppError {
    AppError::invalid_request_with_input_context(
        flag,
        MISSING_VALUE,
        Some(flag.to_owned()),
        [format!("{flag} <value>"), format!("{flag}=<value>")],
        [format!(
            "Provide a value after {flag} or use {flag}=<value>."
        )],
    )
}

pub(super) fn invalid_value_error(
    field: impl Into<String>,
    received: impl Into<String>,
    accepted: impl IntoIterator<Item = impl Into<String>>,
    guidance: impl IntoIterator<Item = impl Into<String>>,
) -> AppError {
    AppError::invalid_request_with_input_context(
        field,
        INVALID_VALUE,
        Some(received.into()),
        accepted,
        guidance,
    )
}

pub(super) fn invalid_positive_value_error(flag: &str, value: &str) -> AppError {
    invalid_value_error(
        flag,
        value,
        ["positive integer"],
        [format!("Provide a positive integer value for {flag}.")],
    )
}

pub(super) fn invalid_output_value_error(value: &str) -> AppError {
    invalid_value_error(
        flags::OUTPUT,
        value,
        OutputMode::ACCEPTED_VALUES.iter().copied(),
        [format!(
            "Use one of these output modes: {}.",
            OutputMode::ACCEPTED_VALUES.join(", ")
        )],
    )
}

pub(super) fn is_flag(token: &str) -> bool {
    token.starts_with("--")
}

pub(super) fn split_equals(token: &str) -> (&str, Option<&str>) {
    token
        .split_once('=')
        .map_or((token, None), |(flag, value)| (flag, Some(value)))
}

fn received_value_flag(flag: &str, value: Option<&str>) -> String {
    if flag.contains('=') {
        return flag.to_owned();
    }
    value.map_or_else(|| flag.to_owned(), |value| format!("{flag} {value}"))
}

fn unsupported_argument_expected(command: &str, flag: &str) -> Vec<String> {
    let command_shape = match command {
        "outline" => {
            "outline <path> [--page <n>] [--limit <n>] [--pagination <enabled|disabled>] [--adapter <id>] [--output <mode>]"
        }
        "read" => {
            "read <path> --ref <ref> [--page <n>] [--limit <n>] [--pagination <enabled|disabled>] [--adapter <id>] [--output <mode>]"
        }
        "find" => {
            "find <path> --query <text> [--page <n>] [--limit <n>] [--pagination <enabled|disabled>] [--adapter <id>] [--output <mode>]"
        }
        "info" => "info <path> [--adapter <id>] [--output <mode>]",
        _ => return vec![format!("{command} without {flag}")],
    };

    vec![
        command_shape.to_owned(),
        format!("a command that supports {flag}"),
    ]
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

pub(super) fn config_path_args(matches: &ArgMatches) -> ConfigPathArgs {
    ConfigPathArgs {
        project_config: optional_explicit_string(matches, arg_ids::PROJECT_CONFIG),
        user_config: optional_explicit_string(matches, arg_ids::USER_CONFIG),
    }
}

pub(super) fn project_config_path_args(matches: &ArgMatches) -> ConfigPathArgs {
    ConfigPathArgs {
        project_config: optional_explicit_string(matches, arg_ids::PROJECT_CONFIG),
        user_config: None,
    }
}

pub(super) fn optional_explicit_output(matches: &ArgMatches) -> AppResult<Option<OutputMode>> {
    optional_explicit_string(matches, arg_ids::OUTPUT)
        .map(|value| {
            value
                .parse()
                .map_err(|_reason: String| invalid_output_value_error(&value))
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
        .ok_or_else(|| missing_value_flag_error(flag))?;
    positive_from_u32(parsed, flag).map(Some)
}

pub(super) fn positive_from_u32(value: u32, flag: &str) -> AppResult<PositiveInteger> {
    let parsed = value;
    NonZeroU32::new(parsed).ok_or_else(|| invalid_positive_value_error(flag, &parsed.to_string()))
}

pub(super) fn parse_operation(value: &str) -> AppResult<Operation> {
    value.parse().map_err(|_| {
        invalid_value_error(
            "--operation",
            value,
            ["outline", "read", "find", "info"],
            ["Use outline, read, find, or info for --operation."],
        )
    })
}
