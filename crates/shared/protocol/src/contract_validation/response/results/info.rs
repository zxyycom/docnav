use serde_json::Value;

use crate::constants::schema_names;

use super::super::super::helpers::{
    reject_unknown_fields, validate_field_set, validate_object_at,
    validate_object_at_with_owned_prefix, ObjectArraySpec,
};
use super::super::fields::{
    response_info_adapter_fields, response_info_document_fields, response_info_result_fields,
    response_measurement_fields,
};

pub(super) fn validate_info_result(value: &Value, errors: &mut Vec<String>) {
    validate_field_set(
        schema_names::PROTOCOL_RESPONSE,
        response_info_result_fields,
        value,
        &[],
        errors,
    );
    reject_unknown_fields(
        schema_names::PROTOCOL_RESPONSE,
        response_info_result_fields,
        value,
        &["result"],
        errors,
    );
    validate_info_document(value, &["result", "document"], errors);
    validate_info_adapter(value, &["result", "adapter"], errors);
}

fn validate_info_document(value: &Value, path: &[&str], errors: &mut Vec<String>) {
    validate_object_at(
        value,
        path,
        ObjectArraySpec {
            schema: schema_names::PROTOCOL_RESPONSE,
            build: response_info_document_fields,
        },
        |document, prefix, errors| {
            validate_object_at_with_owned_prefix(
                document,
                &["size"],
                prefix,
                ObjectArraySpec {
                    schema: schema_names::PROTOCOL_RESPONSE,
                    build: response_measurement_fields,
                },
                errors,
            );
        },
        errors,
    );
}

fn validate_info_adapter(value: &Value, path: &[&str], errors: &mut Vec<String>) {
    validate_object_at(
        value,
        path,
        ObjectArraySpec {
            schema: schema_names::PROTOCOL_RESPONSE,
            build: response_info_adapter_fields,
        },
        |_, _, _| {},
        errors,
    );
}
