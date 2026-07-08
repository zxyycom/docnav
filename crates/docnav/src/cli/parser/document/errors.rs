use docnav_protocol::Operation;

use crate::cli::command_model::OutputMode;
use crate::error::AppError;

use super::super::super::flags;
use super::super::argument_helpers::{
    invalid_output_value_error, invalid_positive_value_error, invalid_value_error, is_flag,
    missing_value_flag_error, split_equals, ValueFlag,
};
use super::super::{spec, ParserContext};
use super::value_flags::{
    document_uses_flag, is_known_document_value_flag, value_flag_occurrence, ValueFlagOccurrence,
};

const PAGINATION_ACCEPTED_VALUES: [&str; 2] = [
    spec::pagination_values::ENABLED,
    spec::pagination_values::DISABLED,
];
const PAGINATION_GUIDANCE: [&str; 1] = ["Use --pagination enabled or --pagination disabled."];

pub(super) fn document_parse_error(
    operation: Operation,
    args: &[String],
    context: &ParserContext,
) -> AppError {
    if !has_path_candidate(operation, args, context) {
        return AppError::invalid_request(
            "path",
            format!("{} requires <path>", operation.as_str()),
        );
    }
    match operation {
        Operation::Read if !has_value_flag(args, flags::REF) => {
            AppError::invalid_request(flags::REF, "read requires --ref <ref>")
        }
        Operation::Find if !has_value_flag(args, flags::QUERY) => {
            AppError::invalid_request(flags::QUERY, "find requires --query <text>")
        }
        _ => first_invalid_used_flag(operation, args)
            .unwrap_or_else(|| AppError::invalid_request("argv", "invalid command line arguments")),
    }
}

pub(super) fn invalid_pagination_value_error(value: &str) -> AppError {
    invalid_value_error(
        flags::PAGINATION,
        value,
        PAGINATION_ACCEPTED_VALUES,
        PAGINATION_GUIDANCE,
    )
}

fn has_path_candidate(operation: Operation, args: &[String], context: &ParserContext) -> bool {
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if is_known_document_value_flag(operation, token, context) {
            let (_flag, inline_value) = split_equals(token);
            index += if inline_value.is_some() { 1 } else { 2 };
        } else if is_flag(token) {
            index += 1;
        } else {
            return true;
        }
    }
    false
}

fn has_value_flag(args: &[String], expected: &str) -> bool {
    args.iter().any(|token| {
        let (flag, value) = split_equals(token);
        flag == expected && value.is_some()
    }) || args
        .windows(2)
        .any(|window| window.first().is_some_and(|token| token == expected))
}

fn first_invalid_used_flag(operation: Operation, args: &[String]) -> Option<AppError> {
    let mut index = 0;
    while index < args.len() {
        let Some(occurrence) = value_flag_occurrence(args, index) else {
            index += 1;
            continue;
        };
        if document_uses_flag(operation, occurrence.flag) {
            if let Some(error) = value_flag_error(occurrence) {
                return Some(error);
            }
        }
        index += occurrence.consumed;
    }
    None
}

fn value_flag_error(occurrence: ValueFlagOccurrence<'_>) -> Option<AppError> {
    match (occurrence.flag, occurrence.value) {
        (_, None) => Some(missing_value_flag_error(occurrence.flag_token)),
        (ValueFlag::Page, Some(value)) => positive_flag_error(flags::PAGE, value),
        (ValueFlag::Limit, Some(value)) => positive_flag_error(flags::LIMIT, value),
        (ValueFlag::Pagination, Some(value)) => pagination_flag_error(value),
        (ValueFlag::Output, Some(value)) => output_flag_error(value),
        (ValueFlag::Ref, Some("")) => Some(empty_value_error(flags::REF)),
        (ValueFlag::Query, Some("")) => Some(empty_value_error(flags::QUERY)),
        _ => None,
    }
}

fn pagination_flag_error(value: &str) -> Option<AppError> {
    if matches!(
        value,
        spec::pagination_values::ENABLED | spec::pagination_values::DISABLED
    ) {
        return None;
    }
    Some(invalid_pagination_value_error(value))
}

fn positive_flag_error(flag: &str, value: &str) -> Option<AppError> {
    if value
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
        .is_some()
    {
        return None;
    }
    Some(invalid_positive_value_error(flag, value))
}

fn output_flag_error(value: &str) -> Option<AppError> {
    value
        .parse::<OutputMode>()
        .err()
        .map(|_reason| invalid_output_value_error(value))
}

fn empty_value_error(flag: &str) -> AppError {
    invalid_value_error(
        flag,
        "",
        [format!("non-empty value for {flag}")],
        [format!("Provide a non-empty value for {flag}.")],
    )
}
