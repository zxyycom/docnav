use std::num::NonZeroU32;

use docnav_protocol::{Operation, PositiveInteger};

use crate::error::{AppError, AppResult};

use super::super::flags;

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
    match token {
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

pub(super) fn parse_positive(value: &str, flag: &str) -> AppResult<PositiveInteger> {
    let parsed = value.parse::<u32>().map_err(|_| {
        AppError::invalid_request(flag, format!("{flag} must be a positive integer"))
    })?;
    NonZeroU32::new(parsed).ok_or_else(|| {
        AppError::invalid_request(flag, format!("{flag} must be a positive integer"))
    })
}

pub(super) fn non_empty_value(flag: &str, value: &str) -> AppResult<String> {
    if value.is_empty() {
        Err(AppError::invalid_request(
            flag,
            format!("{flag} value must not be empty"),
        ))
    } else {
        Ok(value.to_owned())
    }
}

pub(super) fn required_positional(
    positionals: &[String],
    field: &str,
    reason: &str,
) -> AppResult<String> {
    positionals
        .first()
        .cloned()
        .ok_or_else(|| AppError::invalid_request(field, reason))
}

pub(super) fn parse_operation(value: &str) -> AppResult<Operation> {
    value.parse().map_err(|_| {
        AppError::invalid_request("--operation", "expected outline, read, find, or info")
    })
}
