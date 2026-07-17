#![forbid(unsafe_code)]
//! `serde_json::Value` config extraction for `cli-config-resolution`.

use std::fmt;

use cli_config_resolution::{
    FieldDefSet, FieldPath, ProcessingId, ProcessingLocator, Source, SourceCandidate, SourceError,
    SourceId, SourceKind, SourceLocator,
};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub enum ConfigExtractionError {
    UnsupportedLocator {
        processing_id: ProcessingId,
        locator: ProcessingLocator,
    },
    Source(SourceError),
}

impl fmt::Display for ConfigExtractionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedLocator {
                processing_id,
                locator,
            } => write!(
                formatter,
                "processing {processing_id} uses non-config locator {locator:?}"
            ),
            Self::Source(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ConfigExtractionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnsupportedLocator { .. } => None,
            Self::Source(error) => Some(error),
        }
    }
}

impl From<SourceError> for ConfigExtractionError {
    fn from(error: SourceError) -> Self {
        Self::Source(error)
    }
}

pub fn extract_config(
    root: &Value,
    fields: &FieldDefSet,
    processing_id: &ProcessingId,
    source_id: SourceId,
    priority: i32,
) -> Result<Source, ConfigExtractionError> {
    let mut candidates = Vec::new();
    for metadata in fields.processing_metadata(processing_id) {
        let identity = metadata.identity().clone();
        let ProcessingLocator::ConfigPath(path) = metadata.locator else {
            return Err(ConfigExtractionError::UnsupportedLocator {
                processing_id: metadata.processing_id,
                locator: metadata.locator,
            });
        };
        let Some(value) = value_at_path(root, &path) else {
            continue;
        };
        candidates.push(SourceCandidate::value(
            identity,
            SourceLocator::ConfigPath(path),
            value.clone(),
        ));
    }
    Source::new(source_id, SourceKind::Config, priority, candidates).map_err(Into::into)
}

fn value_at_path<'a>(root: &'a Value, path: &FieldPath) -> Option<&'a Value> {
    let mut value = root;
    for segment in path.segments() {
        value = value.as_object()?.get(segment)?;
    }
    Some(value)
}

// @case WB-PARAM-SERDE-001
#[cfg(test)]
mod tests;
