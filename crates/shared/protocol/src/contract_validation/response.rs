use serde_json::Value;

use crate::constants::schema_names;
use crate::SchemaValidationError;

mod fields;
mod results;

use super::field_builders::string_value_fields;
use super::helpers::{
    expect_bool_value, reject_unknown_fields, schema_result, validate_field_set,
    validate_value_array_items, value_at, ValueArraySpec,
};
use fields::{
    response_common_fields, response_failure_fields, response_success_fields,
    response_unknown_shape_fields,
};
use results::validate_success_result_shape;

pub(crate) fn validate_protocol_response_contract_value(
    value: &Value,
) -> Result<(), SchemaValidationError> {
    let mut errors = Vec::new();
    match value.get("ok").and_then(Value::as_bool) {
        Some(true) => validate_success_response(value, &mut errors),
        Some(false) => validate_failure_response(value, &mut errors),
        _ => validate_unknown_response_shape(value, &mut errors),
    }
    schema_result(schema_names::PROTOCOL_RESPONSE, errors)
}

fn validate_unknown_response_shape(value: &Value, errors: &mut Vec<String>) {
    reject_unknown_fields(
        schema_names::PROTOCOL_RESPONSE,
        response_unknown_shape_fields,
        value,
        &[],
        errors,
    );
    validate_field_set(
        schema_names::PROTOCOL_RESPONSE,
        response_common_fields,
        value,
        &[],
        errors,
    );
}

fn validate_success_response(value: &Value, errors: &mut Vec<String>) {
    reject_unknown_fields(
        schema_names::PROTOCOL_RESPONSE,
        response_success_fields,
        value,
        &[],
        errors,
    );
    validate_field_set(
        schema_names::PROTOCOL_RESPONSE,
        response_success_fields,
        value,
        &[],
        errors,
    );
    expect_bool_value(value, &["ok"], true, errors);
    validate_success_result_shape(value, errors);
}

fn validate_failure_response(value: &Value, errors: &mut Vec<String>) {
    reject_unknown_fields(
        schema_names::PROTOCOL_RESPONSE,
        response_failure_fields,
        value,
        &[],
        errors,
    );
    validate_field_set(
        schema_names::PROTOCOL_RESPONSE,
        response_failure_fields,
        value,
        &[],
        errors,
    );
    expect_bool_value(value, &["ok"], false, errors);
    reject_unknown_fields(
        schema_names::PROTOCOL_RESPONSE,
        response_failure_fields,
        value,
        &["error"],
        errors,
    );
    if let Some(error) = value_at(value, &["error"]) {
        validate_value_array_items(
            error,
            &["guidance"],
            &["error"],
            ValueArraySpec {
                schema: schema_names::PROTOCOL_RESPONSE,
                build: string_value_fields,
            },
            errors,
        );
    }
}
