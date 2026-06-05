use serde_json::Value;
use std::fmt;

use crate::constants::schema_names;

const PROTOCOL_REQUEST_SCHEMA: &str =
    include_str!("../../../docs/schemas/protocol-request.schema.json");
const PROTOCOL_RESPONSE_SCHEMA: &str =
    include_str!("../../../docs/schemas/protocol-response.schema.json");
const MANIFEST_SCHEMA: &str = include_str!("../../../docs/schemas/manifest.schema.json");
const PROBE_RESULT_SCHEMA: &str = include_str!("../../../docs/schemas/probe-result.schema.json");

pub fn validate_protocol_request_value(value: &Value) -> Result<(), SchemaValidationError> {
    validate_value_with_schema(
        schema_names::PROTOCOL_REQUEST,
        PROTOCOL_REQUEST_SCHEMA,
        value,
    )
}

pub fn validate_protocol_response_value(value: &Value) -> Result<(), SchemaValidationError> {
    validate_value_with_schema(
        schema_names::PROTOCOL_RESPONSE,
        PROTOCOL_RESPONSE_SCHEMA,
        value,
    )
}

pub fn validate_manifest_value(value: &Value) -> Result<(), SchemaValidationError> {
    validate_value_with_schema(schema_names::MANIFEST, MANIFEST_SCHEMA, value)
}

pub fn validate_probe_result_value(value: &Value) -> Result<(), SchemaValidationError> {
    validate_value_with_schema(schema_names::PROBE_RESULT, PROBE_RESULT_SCHEMA, value)
}

fn validate_value_with_schema(
    schema_name: &'static str,
    schema_source: &str,
    value: &Value,
) -> Result<(), SchemaValidationError> {
    let schema = serde_json::from_str::<Value>(schema_source).map_err(|error| {
        SchemaValidationError::compile(schema_name, format!("schema JSON parse failed: {error}"))
    })?;
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .map_err(|error| {
            SchemaValidationError::compile(schema_name, format!("schema compile failed: {error}"))
        })?;

    let errors = validator
        .iter_errors(value)
        .map(|error| {
            let path = error.instance_path().as_str();
            if path.is_empty() {
                error.to_string()
            } else {
                format!("{path}: {error}")
            }
        })
        .collect::<Vec<_>>();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(SchemaValidationError {
            schema: schema_name,
            errors,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaValidationError {
    pub schema: &'static str,
    pub errors: Vec<String>,
}

impl SchemaValidationError {
    fn compile(schema: &'static str, message: String) -> Self {
        Self {
            schema,
            errors: vec![message],
        }
    }
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
