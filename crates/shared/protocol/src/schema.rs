use serde_json::Value;
use std::fmt;

pub fn validate_protocol_request_value(value: &Value) -> Result<(), SchemaValidationError> {
    crate::contract_validation::validate_protocol_request_contract_value(value)
}

pub fn validate_protocol_response_value(value: &Value) -> Result<(), SchemaValidationError> {
    crate::contract_validation::validate_protocol_response_contract_value(value)
}

pub fn validate_manifest_value(value: &Value) -> Result<(), SchemaValidationError> {
    crate::contract_validation::validate_manifest_contract_value(value)
}

pub fn validate_probe_result_value(value: &Value) -> Result<(), SchemaValidationError> {
    crate::contract_validation::validate_probe_result_contract_value(value)
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
