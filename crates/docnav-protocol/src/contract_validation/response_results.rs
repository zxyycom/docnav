use serde_json::Value;

use crate::constants::schema_names;

use super::field_builders::operation_value_fields;
use super::helpers::{
    operation_name, reject_unknown_fields, validate_field_set, validate_object_array_items,
    validate_value_array_items, value_at, ObjectArraySpec, ValueArraySpec,
};
use super::response_fields::{
    response_entry_fields, response_find_result_fields, response_info_result_fields,
    response_outline_result_fields, response_read_result_fields,
};

pub(super) fn validate_success_result_shape(value: &Value, errors: &mut Vec<String>) {
    let result = value_at(value, &["result"]);
    let shape = result.and_then(success_result_shape);
    match shape.or_else(|| operation_name(value)) {
        Some("outline") => validate_outline_result(value, errors),
        Some("read") => validate_read_result(value, errors),
        Some("find") => validate_find_result(value, errors),
        Some("info") => validate_info_result(value, errors),
        _ => {}
    }
}

fn success_result_shape(result: &Value) -> Option<&'static str> {
    let Value::Object(object) = result else {
        return None;
    };
    if object.contains_key("entries") {
        Some("outline")
    } else if object.contains_key("matches") {
        Some("find")
    } else if ["ref", "content", "content_type", "cost"]
        .iter()
        .any(|field| object.contains_key(*field))
    {
        Some("read")
    } else if ["display", "capabilities"]
        .iter()
        .any(|field| object.contains_key(*field))
    {
        Some("info")
    } else {
        None
    }
}

fn validate_outline_result(value: &Value, errors: &mut Vec<String>) {
    validate_field_set(
        schema_names::PROTOCOL_RESPONSE,
        response_outline_result_fields,
        value,
        &[],
        errors,
    );
    reject_unknown_fields(
        value_at(value, &["result"]),
        &["result"],
        &["entries", "page"],
        errors,
    );
    validate_entry_array(value, &["result", "entries"], errors);
}

fn validate_read_result(value: &Value, errors: &mut Vec<String>) {
    validate_field_set(
        schema_names::PROTOCOL_RESPONSE,
        response_read_result_fields,
        value,
        &[],
        errors,
    );
    reject_unknown_fields(
        value_at(value, &["result"]),
        &["result"],
        &["ref", "content", "content_type", "cost", "page"],
        errors,
    );
}

fn validate_find_result(value: &Value, errors: &mut Vec<String>) {
    validate_field_set(
        schema_names::PROTOCOL_RESPONSE,
        response_find_result_fields,
        value,
        &[],
        errors,
    );
    reject_unknown_fields(
        value_at(value, &["result"]),
        &["result"],
        &["matches", "page"],
        errors,
    );
    validate_entry_array(value, &["result", "matches"], errors);
}

fn validate_info_result(value: &Value, errors: &mut Vec<String>) {
    validate_field_set(
        schema_names::PROTOCOL_RESPONSE,
        response_info_result_fields,
        value,
        &[],
        errors,
    );
    reject_unknown_fields(
        value_at(value, &["result"]),
        &["result"],
        &["display", "capabilities"],
        errors,
    );
    validate_value_array_items(
        value,
        &["result", "capabilities"],
        &[],
        ValueArraySpec {
            schema: schema_names::PROTOCOL_RESPONSE,
            build: operation_value_fields,
        },
        errors,
    );
}

fn validate_entry_array(value: &Value, path: &[&str], errors: &mut Vec<String>) {
    validate_object_array_items(
        value,
        path,
        ObjectArraySpec {
            schema: schema_names::PROTOCOL_RESPONSE,
            build: response_entry_fields,
            allowed_fields: &["ref", "display"],
        },
        |_, _, _| {},
        errors,
    );
}
