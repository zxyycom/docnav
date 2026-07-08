use docnav_cli_args::{scan_arg_boundaries, ArgBoundaryScan};
use docnav_protocol::Operation;

use crate::error::AppResult;

use super::super::command_model::{CliCommand, DocumentCommand, NativeOptionCliInput, ParsedCli};
use super::super::flags;
use super::argument_helpers::{
    clap_argv, config_path_args, error_from_rejected_arg, missing_value_error,
    optional_explicit_output, optional_explicit_positive, optional_explicit_string,
    required_string,
};
use super::{arg_ids, document_clap_command, ParserContext};

mod errors;
mod value_flags;

pub(super) fn parse_document_command(
    operation: Operation,
    args: &[String],
    context: &ParserContext,
) -> AppResult<ParsedCli> {
    let BoundaryDocumentArgs { clap_args } = collect_document_args(operation, args, context)?;
    let matches = document_clap_command(operation, context)
        .try_get_matches_from(clap_argv(operation.as_str(), clap_args))
        .map_err(|_| errors::document_parse_error(operation, args, context))?;

    Ok(ParsedCli::new(CliCommand::Document(
        document_command_from_matches(operation, &matches, context)?,
    )))
}

struct BoundaryDocumentArgs {
    clap_args: Vec<String>,
}

fn collect_document_args(
    operation: Operation,
    args: &[String],
    context: &ParserContext,
) -> AppResult<BoundaryDocumentArgs> {
    let known_value_flags = value_flags::document_value_flags(operation, context);
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
    context: &ParserContext,
) -> AppResult<DocumentCommand> {
    Ok(DocumentCommand {
        operation,
        path: required_string(matches, arg_ids::PATH, "path")?,
        ref_id: parse_ref_id(operation, matches),
        query: parse_query(operation, matches),
        page: parse_page(operation, matches)?,
        pagination_enabled: parse_pagination_enabled(operation, matches)?,
        limit: parse_limit(operation, matches)?,
        native_options: parse_native_options(operation, matches, context),
        output: optional_explicit_output(matches)?,
        adapter: optional_explicit_string(matches, arg_ids::ADAPTER),
        invocation_log: optional_explicit_string(matches, arg_ids::INVOCATION_LOG),
        invocation_log_content_root: optional_explicit_string(
            matches,
            arg_ids::INVOCATION_LOG_CONTENT_ROOT,
        ),
        config_paths: config_path_args(matches),
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
        super::spec::pagination_values::ENABLED => Ok(true),
        super::spec::pagination_values::DISABLED => Ok(false),
        _ => Err(errors::invalid_pagination_value_error(value)),
    }
}

fn parse_native_options(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
    context: &ParserContext,
) -> Vec<NativeOptionCliInput> {
    let mut inputs = Vec::new();
    for option in context.native_options().for_operation(operation) {
        if let Some(value) = optional_explicit_string(matches, option.arg_id()) {
            inputs.push(NativeOptionCliInput {
                flag: option.flag().to_owned(),
                value,
            });
        }
    }
    inputs
}
