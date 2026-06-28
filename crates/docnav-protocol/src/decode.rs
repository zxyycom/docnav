use serde::de::DeserializeOwned;
use serde_json::Value;
use std::convert::Infallible;
use std::fmt;

use crate::{
    validate_manifest_value, validate_probe_result_value, validate_protocol_request_value,
    validate_protocol_response_value, Manifest, ManifestValidationError, ProbeResult,
    ProbeValidationError, ProtocolResponse, ProtocolValidationError, RawRequestEnvelope,
    SchemaValidationError,
};

pub type ProtocolRequestDecodeError = DecodePipelineError<RawRequestEnvelope, Infallible>;

pub fn decode_value<T, E>(
    value: Value,
    validate_schema: impl FnOnce(&Value) -> Result<(), SchemaValidationError>,
    validate_semantics: impl FnOnce(&T) -> Result<(), E>,
) -> Result<T, DecodePipelineError<T, E>>
where
    T: DeserializeOwned,
{
    validate_schema(&value).map_err(DecodePipelineError::Schema)?;
    let decoded = serde_json::from_value(value).map_err(DecodePipelineError::Deserialize)?;
    match validate_semantics(&decoded) {
        Ok(()) => Ok(decoded),
        Err(error) => Err(DecodePipelineError::Semantic {
            value: Box::new(decoded),
            error,
        }),
    }
}

pub fn decode_protocol_request_value(
    value: Value,
) -> Result<RawRequestEnvelope, ProtocolRequestDecodeError> {
    validate_protocol_request_value(&value).map_err(DecodePipelineError::Schema)?;
    serde_json::from_value(value).map_err(DecodePipelineError::Deserialize)
}

pub fn decode_protocol_response_value(
    value: Value,
) -> Result<ProtocolResponse, DecodePipelineError<ProtocolResponse, ProtocolValidationError>> {
    decode_value::<ProtocolResponse, ProtocolValidationError>(
        value,
        validate_protocol_response_value,
        |response| response.validate(),
    )
}

pub fn decode_manifest_value(
    value: Value,
) -> Result<Manifest, DecodePipelineError<Manifest, ManifestValidationError>> {
    decode_value::<Manifest, ManifestValidationError>(value, validate_manifest_value, |manifest| {
        manifest.validate_semantics()
    })
}

pub fn decode_probe_result_value(
    value: Value,
) -> Result<ProbeResult, DecodePipelineError<ProbeResult, ProbeValidationError>> {
    decode_value::<ProbeResult, ProbeValidationError>(value, validate_probe_result_value, |probe| {
        probe.validate_semantics()
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodePipelineStage {
    Schema,
    Deserialize,
    Semantic,
}

#[derive(Debug)]
pub enum DecodePipelineError<T, E> {
    Schema(SchemaValidationError),
    Deserialize(serde_json::Error),
    Semantic { value: Box<T>, error: E },
}

impl<T, E> DecodePipelineError<T, E> {
    pub const fn stage(&self) -> DecodePipelineStage {
        match self {
            Self::Schema(_) => DecodePipelineStage::Schema,
            Self::Deserialize(_) => DecodePipelineStage::Deserialize,
            Self::Semantic { .. } => DecodePipelineStage::Semantic,
        }
    }
}

impl<T, E> fmt::Display for DecodePipelineError<T, E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Schema(error) => error.fmt(formatter),
            Self::Deserialize(error) => error.fmt(formatter),
            Self::Semantic { error, .. } => error.fmt(formatter),
        }
    }
}

impl<T, E> std::error::Error for DecodePipelineError<T, E>
where
    T: fmt::Debug,
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Schema(error) => Some(error),
            Self::Deserialize(error) => Some(error),
            Self::Semantic { error, .. } => Some(error),
        }
    }
}
