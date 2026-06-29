use docnav_cli_args::{scan_loose_args, LooseArgScan};
use docnav_protocol::Operation;

use crate::error::{AppError, AppResult};

use super::super::command_model::{CliCommand, DocumentCommand, ParsedCli};
use super::super::flags;
use super::argument_helpers::{
    clap_argv, is_flag, known_value_flag, loose_value_flags, missing_value_error,
    optional_explicit_output, optional_explicit_positive, optional_explicit_string,
    required_string, split_equals, warning_from_ignored_arg, ValueFlag,
};
use super::{arg_ids, document_clap_command};

pub(super) fn parse_document_command(
    operation: Operation,
    args: &[String],
) -> AppResult<ParsedCli> {
    let LooseDocumentArgs {
        clap_args,
        warnings,
    } = collect_document_args(operation, args)?;
    let matches = document_clap_command(operation)
        .try_get_matches_from(clap_argv(operation.as_str(), clap_args))
        .map_err(|_| document_parse_error(operation, args))?;

    Ok(ParsedCli::new(
        CliCommand::Document(document_command_from_matches(operation, &matches)?),
        warnings,
    ))
}

struct LooseDocumentArgs {
    clap_args: Vec<String>,
    warnings: Vec<super::super::warning::CliWarning>,
}

fn collect_document_args(operation: Operation, args: &[String]) -> AppResult<LooseDocumentArgs> {
    let known_value_flags = loose_value_flags(|flag| document_uses_flag(operation, flag));
    let scan = scan_loose_args(
        args,
        &LooseArgScan::new(operation.as_str(), 1, &known_value_flags),
    )
    .map_err(missing_value_error)?;

    Ok(LooseDocumentArgs {
        clap_args: scan.retained_args,
        warnings: scan
            .ignored
            .into_iter()
            .map(warning_from_ignored_arg)
            .collect(),
    })
}

fn document_command_from_matches(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
) -> AppResult<DocumentCommand> {
    Ok(DocumentCommand {
        operation,
        path: required_string(matches, arg_ids::PATH, "path")?,
        ref_id: parse_ref_id(operation, matches),
        query: parse_query(operation, matches),
        page: parse_page(operation, matches)?,
        limit: parse_limit(operation, matches)?,
        output: optional_explicit_output(matches)?,
        adapter: optional_explicit_string(matches, arg_ids::ADAPTER),
    })
}

fn parse_ref_id(operation: Operation, matches: &clap::parser::ArgMatches) -> Option<String> {
    (operation == Operation::Read)
        .then(|| optional_explicit_string(matches, arg_ids::REF))
        .flatten()
}

fn parse_query(operation: Operation, matches: &clap::parser::ArgMatches) -> Option<String> {
    (operation == Operation::Find)
        .then(|| optional_explicit_string(matches, arg_ids::QUERY))
        .flatten()
}

fn parse_page(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
) -> AppResult<Option<docnav_protocol::PositiveInteger>> {
    if operation == Operation::Info {
        return Ok(None);
    }
    optional_explicit_positive(matches, arg_ids::PAGE, flags::PAGE)
}

fn parse_limit(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
) -> AppResult<Option<docnav_protocol::PositiveInteger>> {
    if operation == Operation::Info {
        return Ok(None);
    }
    optional_explicit_positive(matches, arg_ids::LIMIT, flags::LIMIT)
}

fn document_parse_error(operation: Operation, args: &[String]) -> AppError {
    if !has_path_candidate(args) {
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

fn document_uses_flag(operation: Operation, flag: ValueFlag) -> bool {
    match flag {
        ValueFlag::Adapter | ValueFlag::Output => true,
        ValueFlag::Page | ValueFlag::Limit => operation != Operation::Info,
        ValueFlag::Ref => operation == Operation::Read,
        ValueFlag::Query => operation == Operation::Find,
        ValueFlag::Operation | ValueFlag::Path => false,
    }
}

fn has_path_candidate(args: &[String]) -> bool {
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if known_value_flag(token).is_some() {
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

#[derive(Clone, Copy)]
struct ValueFlagOccurrence<'a> {
    flag: ValueFlag,
    flag_token: &'a str,
    value: Option<&'a str>,
    consumed: usize,
}

fn value_flag_occurrence(args: &[String], index: usize) -> Option<ValueFlagOccurrence<'_>> {
    let token = &args[index];
    let flag = known_value_flag(token)?;
    let (flag_token, inline_value) = split_equals(token);
    Some(ValueFlagOccurrence {
        flag,
        flag_token,
        value: inline_value.or_else(|| args.get(index + 1).map(String::as_str)),
        consumed: if inline_value.is_some() { 1 } else { 2 },
    })
}

fn value_flag_error(occurrence: ValueFlagOccurrence<'_>) -> Option<AppError> {
    match (occurrence.flag, occurrence.value) {
        (_, None) => Some(AppError::invalid_request(
            occurrence.flag_token,
            "flag requires a value",
        )),
        (ValueFlag::Page, Some(value)) => positive_flag_error(flags::PAGE, value),
        (ValueFlag::Limit, Some(value)) => positive_flag_error(flags::LIMIT, value),
        (ValueFlag::Output, Some(value)) => output_flag_error(value),
        (ValueFlag::Ref, Some("")) => Some(empty_value_error(flags::REF)),
        (ValueFlag::Query, Some("")) => Some(empty_value_error(flags::QUERY)),
        _ => None,
    }
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
    Some(AppError::invalid_request(
        flag,
        format!("{flag} must be a positive integer"),
    ))
}

fn output_flag_error(value: &str) -> Option<AppError> {
    value
        .parse::<super::super::command_model::OutputMode>()
        .err()
        .map(|reason| {
            AppError::invalid_request(
                flags::OUTPUT,
                format!("invalid {}: {reason}", flags::OUTPUT),
            )
        })
}

fn empty_value_error(flag: &str) -> AppError {
    AppError::invalid_request(flag, format!("{flag} value must not be empty"))
}
