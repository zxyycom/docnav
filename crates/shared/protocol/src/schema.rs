use serde_json::Value;
use std::fmt;

use crate::{ProtocolResponse, ProtocolValidationError};

pub fn validate_protocol_request_value(value: &Value) -> Result<(), SchemaValidationError> {
    crate::contract_validation::validate_protocol_request_contract_value(value)
}

pub fn validate_protocol_response_value(value: &Value) -> Result<(), SchemaValidationError> {
    crate::contract_validation::validate_protocol_response_contract_value(value)
}

pub(crate) fn validate_protocol_response(
    response: &ProtocolResponse,
) -> Result<(), ProtocolResponseContractError> {
    let value =
        serde_json::to_value(response).map_err(ProtocolResponseContractError::Serialization)?;
    validate_protocol_response_value(&value).map_err(ProtocolResponseContractError::Schema)?;
    response
        .validate()
        .map_err(ProtocolResponseContractError::Semantic)
}

pub fn validate_manifest_value(value: &Value) -> Result<(), SchemaValidationError> {
    crate::contract_validation::validate_manifest_contract_value(value)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaValidationError {
    pub schema: &'static str,
    pub errors: Vec<String>,
}

impl fmt::Display for SchemaValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} failed validation", self.schema)?;
        if !self.errors.is_empty() {
            write!(formatter, ": {}", self.errors.join("; "))?;
        }
        Ok(())
    }
}

impl std::error::Error for SchemaValidationError {}

/// Failure while validating one typed response against the complete protocol contract.
#[derive(Debug)]
pub enum ProtocolResponseContractError {
    Serialization(serde_json::Error),
    Schema(SchemaValidationError),
    Semantic(ProtocolValidationError),
}

impl fmt::Display for ProtocolResponseContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization(error) => {
                write!(formatter, "protocol response serialization failed: {error}")
            }
            Self::Schema(error) => error.fmt(formatter),
            Self::Semantic(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ProtocolResponseContractError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Serialization(error) => Some(error),
            Self::Schema(error) => Some(error),
            Self::Semantic(error) => Some(error),
        }
    }
}
