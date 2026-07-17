use clap::parser::ValueSource;
use cli_config_resolution::{
    ProcessingId, Source, SourceCandidate, SourceId, SourceKind, SourceLocator,
};
use docnav_cli_args::{scan_arg_boundaries, ArgBoundaryScan, RejectedArg};
use docnav_navigation::{
    DocumentParameterCatalog, DOCUMENT_CLI_SOURCE_ID, DOCUMENT_CLI_SOURCE_PRIORITY,
};
use docnav_protocol::Operation;
use docnav_typed_fields::{
    CliBooleanEncoding, ProcessingLocator, ProcessingMetadataView, ValueKind,
};

use crate::error::AppResult;

use super::super::command_model::{CliCommand, DocumentCommand, ParsedCli};
use super::argument_helpers::{
    clap_argv, config_path_args, error_from_rejected_arg, optional_explicit_string, required_string,
};
use super::spec::DocumentProjectionError;
use super::{arg_ids, document_clap_command};

mod errors;
mod value_flags;

pub(super) fn parse_document_command(
    operation: Operation,
    args: &[String],
) -> AppResult<ParsedCli> {
    let spec = document_clap_command(operation)?;
    let value_flags = value_flags::DocumentValueFlags::new(operation, &spec.command);
    let BoundaryDocumentArgs { clap_args } = collect_document_args(operation, args, &value_flags)?;
    let matches = spec
        .command
        .try_get_matches_from(clap_argv(operation.as_str(), clap_args))
        .map_err(|error| errors::document_parse_error(operation, args, &value_flags, &error))?;
    let processing_id = ProcessingId::new("cli").expect("document CLI processing id is valid");
    let mut candidates = spec
        .routing_fields
        .processing_metadata(&processing_id)
        .into_iter()
        .map(|metadata| candidate_from_matches(&matches, &metadata))
        .filter_map(Result::transpose)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| super::spec::document_projection_error(operation, "navigation", error))?;
    candidates.extend(
        extract_parameter_candidates(&matches, &spec.parameters, operation, &processing_id)
            .map_err(|error| {
                super::spec::document_projection_error(operation, "core-catalog", error)
            })?,
    );
    let cli_source = Source::new(
        SourceId::new(DOCUMENT_CLI_SOURCE_ID).expect("document CLI source id is valid"),
        SourceKind::Cli,
        DOCUMENT_CLI_SOURCE_PRIORITY,
        candidates,
    )
    .map_err(|error| {
        super::spec::document_projection_error(
            operation,
            "core",
            DocumentProjectionError::source(error),
        )
    })?;

    Ok(ParsedCli::new(CliCommand::Document(
        document_command_from_matches(operation, &matches, cli_source)?,
    )))
}

fn extract_parameter_candidates(
    matches: &clap::ArgMatches,
    catalog: &DocumentParameterCatalog,
    operation: Operation,
    processing_id: &ProcessingId,
) -> Result<Vec<SourceCandidate>, DocumentProjectionError> {
    catalog
        .operation_fields(operation)
        .filter_map(|field| field.processing_metadata(processing_id))
        .map(|metadata| candidate_from_matches(matches, &metadata))
        .filter_map(Result::transpose)
        .collect()
}

fn candidate_from_matches(
    matches: &clap::ArgMatches,
    metadata: &ProcessingMetadataView<'_>,
) -> Result<Option<SourceCandidate>, DocumentProjectionError> {
    let argument_id = metadata.identity().as_str().to_owned();
    matches
        .try_contains_id(&argument_id)
        .map_err(|error| match_read_error(metadata, error.to_string()))?;
    if matches.value_source(&argument_id) != Some(ValueSource::CommandLine) {
        return Ok(None);
    }
    let ProcessingLocator::CliFlag(flag) = &metadata.locator else {
        return Err(DocumentProjectionError::for_field(
            metadata.identity().clone(),
            format!(
                "field {} uses non-CLI processing locator {:?}",
                metadata.identity().as_str(),
                metadata.locator
            ),
        ));
    };
    let locator = SourceLocator::CliFlag(flag.clone());
    let candidate = match metadata.value_kind() {
        ValueKind::Boolean => {
            let encoding = metadata
                .cli
                .as_ref()
                .and_then(|metadata| metadata.boolean_encoding.as_ref());
            let Some(CliBooleanEncoding::Explicit {
                true_token: Some(true_token),
                false_token: Some(false_token),
            }) = encoding
            else {
                return Err(match_read_error(
                    metadata,
                    "canonical Boolean CLI token mapping is incomplete",
                ));
            };
            let raw = read_one_string(matches, metadata)?;
            if raw == *true_token {
                SourceCandidate::value(metadata.identity().clone(), locator, true.into())
            } else if raw == *false_token {
                SourceCandidate::value(metadata.identity().clone(), locator, false.into())
            } else {
                SourceCandidate::invalid(
                    metadata.identity().clone(),
                    locator,
                    raw.into(),
                    format!("expected Boolean CLI token {true_token:?} or {false_token:?}"),
                )
            }
        }
        ValueKind::String => SourceCandidate::value(
            metadata.identity().clone(),
            locator,
            read_one_string(matches, metadata)?.into(),
        ),
        ValueKind::Integer => {
            let raw = read_one_string(matches, metadata)?;
            match raw.parse::<i64>() {
                Ok(value) => {
                    SourceCandidate::value(metadata.identity().clone(), locator, value.into())
                }
                Err(_) => SourceCandidate::invalid(
                    metadata.identity().clone(),
                    locator,
                    raw.into(),
                    "expected integer CLI value",
                ),
            }
        }
        value_kind => {
            return Err(DocumentProjectionError::for_field(
                metadata.identity().clone(),
                format!(
                    "field {} has unsupported CLI value kind {value_kind:?}",
                    metadata.identity().as_str()
                ),
            ));
        }
    };
    Ok(Some(candidate))
}

fn read_one_string(
    matches: &clap::ArgMatches,
    metadata: &ProcessingMetadataView<'_>,
) -> Result<String, DocumentProjectionError> {
    matches
        .try_get_one::<String>(metadata.identity().as_str())
        .map_err(|error| match_read_error(metadata, error.to_string()))?
        .cloned()
        .ok_or_else(|| match_read_error(metadata, "explicit flag has no value"))
}

fn match_read_error(
    metadata: &ProcessingMetadataView<'_>,
    reason: impl Into<String>,
) -> DocumentProjectionError {
    let flag = metadata
        .locator
        .cli_flag()
        .unwrap_or("<invalid-cli-locator>");
    DocumentProjectionError::for_field(
        metadata.identity().clone(),
        format!(
            "could not read CLI flag {flag} for field {}: {}",
            metadata.identity().as_str(),
            reason.into()
        ),
    )
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
