use clap::parser::ValueSource;
use cli_config_resolution::{ProcessingId, SourceCandidate, SourceLocator};
use docnav_navigation::DocumentParameterCatalog;
use docnav_protocol::Operation;
use docnav_typed_fields::{
    CliBooleanEncoding, ProcessingLocator, ProcessingMetadataView, ValueKind,
};

use super::super::spec::DocumentProjectionError;

pub(super) fn extract_parameter_candidates(
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

pub(super) fn candidate_from_matches(
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

    let locator = cli_locator(metadata)?;
    candidate_value(matches, metadata, locator).map(Some)
}

fn cli_locator(
    metadata: &ProcessingMetadataView<'_>,
) -> Result<SourceLocator, DocumentProjectionError> {
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
    Ok(SourceLocator::CliFlag(flag.clone()))
}

fn candidate_value(
    matches: &clap::ArgMatches,
    metadata: &ProcessingMetadataView<'_>,
    locator: SourceLocator,
) -> Result<SourceCandidate, DocumentProjectionError> {
    match metadata.value_kind() {
        ValueKind::Boolean => boolean_candidate(matches, metadata, locator),
        ValueKind::String => string_candidate(matches, metadata, locator),
        ValueKind::Integer => integer_candidate(matches, metadata, locator),
        value_kind => Err(DocumentProjectionError::for_field(
            metadata.identity().clone(),
            format!(
                "field {} has unsupported CLI value kind {value_kind:?}",
                metadata.identity().as_str()
            ),
        )),
    }
}

fn boolean_candidate(
    matches: &clap::ArgMatches,
    metadata: &ProcessingMetadataView<'_>,
    locator: SourceLocator,
) -> Result<SourceCandidate, DocumentProjectionError> {
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
    let candidate = if raw == *true_token {
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
    };
    Ok(candidate)
}

fn string_candidate(
    matches: &clap::ArgMatches,
    metadata: &ProcessingMetadataView<'_>,
    locator: SourceLocator,
) -> Result<SourceCandidate, DocumentProjectionError> {
    Ok(SourceCandidate::value(
        metadata.identity().clone(),
        locator,
        read_one_string(matches, metadata)?.into(),
    ))
}

fn integer_candidate(
    matches: &clap::ArgMatches,
    metadata: &ProcessingMetadataView<'_>,
    locator: SourceLocator,
) -> Result<SourceCandidate, DocumentProjectionError> {
    let raw = read_one_string(matches, metadata)?;
    Ok(match raw.parse::<i64>() {
        Ok(value) => SourceCandidate::value(metadata.identity().clone(), locator, value.into()),
        Err(_) => SourceCandidate::invalid(
            metadata.identity().clone(),
            locator,
            raw.into(),
            "expected integer CLI value",
        ),
    })
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
