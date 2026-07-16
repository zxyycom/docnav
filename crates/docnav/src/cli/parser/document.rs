use cli_config_resolution::{ProcessingId, SourceId};
use cli_config_resolution_clap::extract_cli;
use docnav_cli_args::{scan_arg_boundaries, ArgBoundaryScan, RejectedArg};
use docnav_navigation::{
    NavigationAdapterRegistry, DOCUMENT_CLI_SOURCE_ID, DOCUMENT_CLI_SOURCE_PRIORITY,
};
use docnav_protocol::Operation;

use crate::error::AppResult;

use super::super::command_model::{CliCommand, DocumentCommand, ParsedCli};
use super::argument_helpers::{
    clap_argv, config_path_args, error_from_rejected_arg, optional_explicit_string, required_string,
};
use super::{arg_ids, document_clap_command};

mod errors;
mod value_flags;

pub(super) fn parse_document_command<R>(
    operation: Operation,
    args: &[String],
    registry: &R,
) -> AppResult<ParsedCli>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    let (command, fields) = document_clap_command(operation, registry)?;
    let value_flags = value_flags::DocumentValueFlags::new(operation, registry, &command);
    let BoundaryDocumentArgs { clap_args } = collect_document_args(operation, args, &value_flags)?;
    let matches = command
        .try_get_matches_from(clap_argv(operation.as_str(), clap_args))
        .map_err(|error| errors::document_parse_error(operation, args, &value_flags, &error))?;
    let processing_id = ProcessingId::new("cli").expect("document CLI processing id is valid");
    let cli_source = extract_cli(
        &matches,
        fields.fields(),
        &processing_id,
        SourceId::new(DOCUMENT_CLI_SOURCE_ID).expect("document CLI source id is valid"),
        DOCUMENT_CLI_SOURCE_PRIORITY,
    )
    .map_err(|error| super::spec::document_projection_error(operation, &fields, error))?;

    Ok(ParsedCli::new(CliCommand::Document(
        document_command_from_matches(operation, &matches, cli_source)?,
    )))
}

struct BoundaryDocumentArgs {
    clap_args: Vec<String>,
}

fn collect_document_args(
    operation: Operation,
    args: &[String],
    value_flags: &value_flags::DocumentValueFlags,
) -> AppResult<BoundaryDocumentArgs> {
    let known_value_flags = value_flags.known_value_flags();
    let known_switch_flags = value_flags.known_switch_flags();
    let scan = match scan_arg_boundaries(
        args,
        &ArgBoundaryScan::new(operation.as_str(), 1, &known_value_flags)
            .with_known_switch_flags(&known_switch_flags),
    ) {
        Ok(scan) => scan,
        Err(_) => {
            return Ok(BoundaryDocumentArgs {
                clap_args: args.to_vec(),
            })
        }
    };
    if scan
        .rejected
        .iter()
        .any(|rejected| matches!(rejected, RejectedArg::UnknownFlag { .. }))
    {
        return Ok(BoundaryDocumentArgs {
            clap_args: args.to_vec(),
        });
    }
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
    cli_source: cli_config_resolution::Source,
) -> AppResult<DocumentCommand> {
    Ok(DocumentCommand {
        operation,
        path: required_string(matches, arg_ids::PATH, "path")?,
        ref_id: parse_ref_id(operation, matches),
        query: parse_query(operation, matches),
        cli_source: Box::new(cli_source),
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
