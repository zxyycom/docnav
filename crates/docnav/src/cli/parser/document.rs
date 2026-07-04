use docnav_cli_args::{scan_arg_boundaries, ArgBoundaryScan};
use docnav_protocol::Operation;

use crate::error::{AppError, AppResult};
use crate::registry;

use super::super::command_model::{CliCommand, DocumentCommand, NativeOptionCliInput, ParsedCli};
use super::super::flags;
use super::argument_helpers::{
    boundary_value_flags, clap_argv, error_from_rejected_arg, invalid_output_value_error,
    invalid_positive_value_error, invalid_value_error, is_flag, known_value_flag,
    missing_value_error, missing_value_flag_error, optional_explicit_output,
    optional_explicit_positive, optional_explicit_string, required_string, split_equals, ValueFlag,
};
use super::{arg_ids, document_clap_command, spec};

const PAGINATION_ACCEPTED_VALUES: [&str; 2] = [
    spec::pagination_values::ENABLED,
    spec::pagination_values::DISABLED,
];
const PAGINATION_GUIDANCE: [&str; 1] = ["Use --pagination enabled or --pagination disabled."];

pub(super) fn parse_document_command(
    operation: Operation,
    args: &[String],
) -> AppResult<ParsedCli> {
    let BoundaryDocumentArgs { clap_args } = collect_document_args(operation, args)?;
    let matches = document_clap_command(operation)
        .try_get_matches_from(clap_argv(operation.as_str(), clap_args))
        .map_err(|_| document_parse_error(operation, args))?;

    Ok(ParsedCli::new(CliCommand::Document(
        document_command_from_matches(operation, &matches)?,
    )))
}

struct BoundaryDocumentArgs {
    clap_args: Vec<String>,
}

fn collect_document_args(operation: Operation, args: &[String]) -> AppResult<BoundaryDocumentArgs> {
    let known_value_flags = document_value_flags(operation);
    let scan = scan_arg_boundaries(
        args,
        &ArgBoundaryScan::new(operation.as_str(), 1, &known_value_flags),
    )
    .map_err(missing_value_error)?;
    if let Some(rejected) = scan.rejected.into_iter().next() {
        return Err(error_from_rejected_arg(rejected));
    }

    Ok(BoundaryDocumentArgs {
        clap_args: scan.retained_args,
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
        pagination_enabled: parse_pagination_enabled(operation, matches)?,
        limit: parse_limit(operation, matches)?,
        native_options: parse_native_options(operation, matches),
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

fn parse_pagination_enabled(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
) -> AppResult<Option<bool>> {
    if operation == Operation::Info {
        return Ok(None);
    }
    optional_explicit_string(matches, arg_ids::PAGINATION)
        .map(|value| pagination_enabled_from_cli(&value))
        .transpose()
}

fn pagination_enabled_from_cli(value: &str) -> AppResult<bool> {
    match value {
        spec::pagination_values::ENABLED => Ok(true),
        spec::pagination_values::DISABLED => Ok(false),
        _ => Err(invalid_pagination_value_error(value)),
    }
}

fn document_parse_error(operation: Operation, args: &[String]) -> AppError {
    if !has_path_candidate(operation, args) {
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
        ValueFlag::Page | ValueFlag::Pagination | ValueFlag::Limit => operation != Operation::Info,
        ValueFlag::Ref => operation == Operation::Read,
        ValueFlag::Query => operation == Operation::Find,
        ValueFlag::Operation | ValueFlag::Path => false,
    }
}

fn parse_native_options(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
) -> Vec<NativeOptionCliInput> {
    let mut flags = Vec::new();
    let mut inputs = Vec::new();
    for option in registry::native_options_for(operation) {
        let Some(flag) = option.cli_flag() else {
            continue;
        };
        let flag = flag.to_owned();
        if flags.contains(&flag) {
            continue;
        }
        flags.push(flag.clone());
        let Some(arg_id) = option.cli_arg_id().map(str::to_owned) else {
            continue;
        };
        if let Some(value) = optional_explicit_string(matches, &arg_id) {
            inputs.push(NativeOptionCliInput { flag, value });
        }
    }
    inputs
}

fn has_path_candidate(operation: Operation, args: &[String]) -> bool {
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if is_known_document_value_flag(operation, token) {
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

fn invalid_pagination_value_error(value: &str) -> AppError {
    invalid_value_error(
        flags::PAGINATION,
        value,
        PAGINATION_ACCEPTED_VALUES,
        PAGINATION_GUIDANCE,
    )
}

fn document_value_flags(operation: Operation) -> Vec<docnav_cli_args::KnownValueFlag<'static>> {
    let mut flags = boundary_value_flags(|flag| document_uses_flag(operation, flag));
    for option in registry::native_options_for(Operation::Outline)
        .into_iter()
        .chain(registry::native_options_for(Operation::Read))
        .chain(registry::native_options_for(Operation::Find))
        .chain(registry::native_options_for(Operation::Info))
    {
        let Some(flag) = option.cli_flag() else {
            continue;
        };
        let used = option.applies_to(operation);
        if flags.iter().any(|existing| existing.flag == flag) {
            continue;
        }
        flags.push(docnav_cli_args::KnownValueFlag { flag, used });
    }
    flags
}

fn is_known_document_value_flag(operation: Operation, token: &str) -> bool {
    let (flag, _value) = split_equals(token);
    document_value_flags(operation)
        .iter()
        .any(|known| known.flag == flag)
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
        .parse::<super::super::command_model::OutputMode>()
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
