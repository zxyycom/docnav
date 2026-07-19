use serde_json::Value;

use crate::constants::schema_names;

use super::super::helpers::{
    expect_string_value, extend_owned_path, operation_name, reject_unknown_fields,
    validate_field_set, validate_object_array_items, validate_object_array_items_with_owned_prefix,
    validate_object_at, validate_object_at_with_owned_prefix, value_at, ObjectArraySpec,
};
use super::fields::{
    response_auto_read_fields, response_cost_fields, response_cost_fields_allow_empty,
    response_entry_fields, response_find_result_fields, response_info_adapter_fields,
    response_info_document_fields, response_info_result_fields, response_location_fields,
    response_measurement_fields, response_read_result_fields,
    response_structured_outline_result_fields, response_unstructured_outline_result_fields,
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
    if matches!(
        object.get("kind").and_then(Value::as_str),
        Some("structured" | "unstructured")
    ) || object.contains_key("entries")
    {
        Some("outline")
    } else if object.contains_key("matches") {
        Some("find")
    } else if ["ref", "content", "content_type", "cost"]
        .iter()
        .any(|field| object.contains_key(*field))
    {
        Some("read")
    } else if ["document", "adapter", "metadata"]
        .iter()
        .any(|field| object.contains_key(*field))
    {
        Some("info")
    } else {
        None
    }
}

fn validate_outline_result(value: &Value, errors: &mut Vec<String>) {
    match outline_result_kind(value) {
        Some("unstructured") => validate_unstructured_outline_result(value, errors),
        _ => validate_structured_outline_result(value, errors),
    }
}

fn outline_result_kind(value: &Value) -> Option<&str> {
    value_at(value, &["result", "kind"]).and_then(Value::as_str)
}

fn validate_structured_outline_result(value: &Value, errors: &mut Vec<String>) {
    validate_field_set(
        schema_names::PROTOCOL_RESPONSE,
        response_structured_outline_result_fields,
        value,
        &[],
        errors,
    );
    reject_unknown_fields(
        schema_names::PROTOCOL_RESPONSE,
        response_structured_outline_result_fields,
        value,
        &["result"],
        errors,
    );
    expect_string_value(value, &["result", "kind"], "structured", errors);
    validate_entry_array(value, &["result", "entries"], errors);
    validate_auto_read(value, &["result", "auto_read"], errors);
}

fn validate_unstructured_outline_result(value: &Value, errors: &mut Vec<String>) {
    validate_field_set(
        schema_names::PROTOCOL_RESPONSE,
        response_unstructured_outline_result_fields,
        value,
        &[],
        errors,
    );
    reject_unknown_fields(
        schema_names::PROTOCOL_RESPONSE,
        response_unstructured_outline_result_fields,
        value,
        &["result"],
        errors,
    );
    expect_string_value(value, &["result", "kind"], "unstructured", errors);
    validate_cost_with(
        response_cost_fields_allow_empty,
        value,
        &["result", "cost"],
        errors,
    );
}

fn validate_read_result(value: &Value, errors: &mut Vec<String>) {
    validate_object_at(
        value,
        &["result"],
        ObjectArraySpec {
            schema: schema_names::PROTOCOL_RESPONSE,
            build: response_read_result_fields,
        },
        validate_read_cost,
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
        schema_names::PROTOCOL_RESPONSE,
        response_find_result_fields,
        value,
        &["result"],
        errors,
    );
    validate_entry_array(value, &["result", "matches"], errors);
    validate_auto_read(value, &["result", "auto_read"], errors);
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
        schema_names::PROTOCOL_RESPONSE,
        response_info_result_fields,
        value,
        &["result"],
        errors,
    );
    validate_info_document(value, &["result", "document"], errors);
    validate_info_adapter(value, &["result", "adapter"], errors);
}

fn validate_entry_array(value: &Value, path: &[&str], errors: &mut Vec<String>) {
    validate_object_array_items(
        value,
        path,
        ObjectArraySpec {
            schema: schema_names::PROTOCOL_RESPONSE,
            build: response_entry_fields,
        },
        |entry, prefix, errors| {
            validate_location_in_entry(entry, prefix, errors);
            validate_cost_in_entry(entry, prefix, errors);
        },
        errors,
    );
}

fn validate_location_in_entry(entry: &Value, prefix: &[String], errors: &mut Vec<String>) {
    validate_object_at_with_owned_prefix(
        entry,
        &["location"],
        prefix,
        ObjectArraySpec {
            schema: schema_names::PROTOCOL_RESPONSE,
            build: response_location_fields,
        },
        errors,
    );
}

fn validate_cost_in_entry(entry: &Value, prefix: &[String], errors: &mut Vec<String>) {
    validate_object_at_with_owned_prefix(
        entry,
        &["cost"],
        prefix,
        ObjectArraySpec {
            schema: schema_names::PROTOCOL_RESPONSE,
            build: response_cost_fields,
        },
        errors,
    );
    if let Some(cost) = value_at(entry, &["cost"]) {
        let cost_prefix = extend_owned_path(prefix, &["cost"]);
        validate_measurements(cost, &cost_prefix, errors);
    }
}

fn validate_cost_with(
    build: super::super::helpers::FieldSetBuilder,
    value: &Value,
    path: &[&str],
    errors: &mut Vec<String>,
) {
    validate_object_at(
        value,
        path,
        ObjectArraySpec {
            schema: schema_names::PROTOCOL_RESPONSE,
            build,
        },
        validate_measurements,
        errors,
    );
}

fn validate_measurements(cost: &Value, prefix: &[String], errors: &mut Vec<String>) {
    validate_object_array_items_with_owned_prefix(
        cost,
        &["measurements"],
        prefix,
        ObjectArraySpec {
            schema: schema_names::PROTOCOL_RESPONSE,
            build: response_measurement_fields,
        },
        errors,
    );
}

fn validate_auto_read(value: &Value, path: &[&str], errors: &mut Vec<String>) {
    validate_object_at(
        value,
        path,
        ObjectArraySpec {
            schema: schema_names::PROTOCOL_RESPONSE,
            build: response_auto_read_fields,
        },
        |auto_read, auto_read_prefix, errors| {
            validate_object_at_with_owned_prefix(
                auto_read,
                &["read"],
                auto_read_prefix,
                ObjectArraySpec {
                    schema: schema_names::PROTOCOL_RESPONSE,
                    build: response_read_result_fields,
                },
                errors,
            );
            let Some(read) = value_at(auto_read, &["read"]) else {
                return;
            };
            let read_prefix = extend_owned_path(auto_read_prefix, &["read"]);
            validate_read_cost(read, &read_prefix, errors);
        },
        errors,
    );
}

fn validate_read_cost(read: &Value, read_prefix: &[String], errors: &mut Vec<String>) {
    validate_object_at_with_owned_prefix(
        read,
        &["cost"],
        read_prefix,
        ObjectArraySpec {
            schema: schema_names::PROTOCOL_RESPONSE,
            build: response_cost_fields,
        },
        errors,
    );
    if let Some(cost) = value_at(read, &["cost"]) {
        let cost_prefix = extend_owned_path(read_prefix, &["cost"]);
        validate_measurements(cost, &cost_prefix, errors);
    }
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
