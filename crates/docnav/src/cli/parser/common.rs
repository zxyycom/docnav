use std::num::NonZeroU32;

use clap::parser::{ArgMatches, ValueSource};
use docnav_protocol::{Operation, PositiveInteger};

use crate::error::{AppError, AppResult};

use super::super::flags;
use super::super::types::OutputMode;
use super::arg_ids;

#[derive(Clone, Copy)]
pub(super) enum ValueFlag {
    Adapter,
    LimitChars,
    Operation,
    Output,
    Page,
    Path,
    Query,
    Ref,
}

pub(super) fn known_value_flag(token: &str) -> Option<ValueFlag> {
    let (flag, _value) = split_equals(token);
    match flag {
        flags::ADAPTER => Some(ValueFlag::Adapter),
        flags::LIMIT_CHARS => Some(ValueFlag::LimitChars),
        flags::OPERATION => Some(ValueFlag::Operation),
        flags::OUTPUT => Some(ValueFlag::Output),
        flags::PAGE => Some(ValueFlag::Page),
        flags::PATH => Some(ValueFlag::Path),
        flags::QUERY => Some(ValueFlag::Query),
        flags::REF => Some(ValueFlag::Ref),
        _ => None,
    }
}

pub(super) fn is_flag(token: &str) -> bool {
    token.starts_with("--")
}

pub(super) fn split_equals(token: &str) -> (&str, Option<&str>) {
    token
        .split_once('=')
        .map_or((token, None), |(flag, value)| (flag, Some(value)))
}

pub(super) fn push_clap_value_arg(
    clap_args: &mut Vec<String>,
    args: &[String],
    index: &mut usize,
    flag: &str,
) -> AppResult<String> {
    let token = &args[*index];
    if let Some((_flag, value)) = token.split_once('=') {
        clap_args.push(token.clone());
        *index += 1;
        return Ok(value.to_owned());
    }

    let value = args
        .get(*index + 1)
        .ok_or_else(|| AppError::invalid_request(flag, "flag requires a value"))?
        .clone();
    clap_args.push(token.clone());
    clap_args.push(value.clone());
    *index += 2;
    Ok(value)
}

pub(super) fn warning_value<'a>(
    args: &'a [String],
    index: &mut usize,
    flag: &str,
) -> AppResult<Option<&'a str>> {
    let token = &args[*index];
    if let Some((_flag, value)) = token.split_once('=') {
        *index += 1;
        return Ok(Some(value));
    }

    let value = args
        .get(*index + 1)
        .ok_or_else(|| AppError::invalid_request(flag, "flag requires a value"))?;
    *index += 2;
    Ok(Some(value.as_str()))
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
