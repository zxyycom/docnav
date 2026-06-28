use serde_json::{Map, Value};

use docnav_typed_fields::{FieldDefSet, FieldDefSetBuildError};

use crate::SchemaValidationError;

use super::formatting::{
    json_pointer, json_pointer_str, push_field_extraction_errors,
    push_owned_field_extraction_errors, push_wrapped_value_errors,
};
use super::{JSON_CONTRACT_PROCESSING, VALUE_FIELD};

pub(super) type FieldSetBuilder = fn() -> Result<FieldDefSet, FieldDefSetBuildError>;

#[derive(Clone, Copy)]
pub(super) struct ObjectArraySpec {
    pub schema: &'static str,
    pub build: FieldSetBuilder,
    pub allowed_fields: &'static [&'static str],
}

#[derive(Clone, Copy)]
pub(super) struct ValueArraySpec {
    pub schema: &'static str,
    pub build: FieldSetBuilder,
}

pub(super) fn validate_field_set(
    schema: &'static str,
    build: FieldSetBuilder,
    value: &Value,
    prefix: &[&str],
    errors: &mut Vec<String>,
) {
    match build() {
        Ok(fields) => {
            if let Err(error) = fields.validate_json(JSON_CONTRACT_PROCESSING, value) {
                push_field_extraction_errors(errors, prefix, error);
            }
        }
        Err(error) => errors.push(format!(
            "{schema} contract field definitions failed: {error}"
        )),
    }
}

pub(super) fn validate_object_array_items(
    value: &Value,
    path: &[&str],
    spec: ObjectArraySpec,
    validate_nested: impl Fn(&Value, &[String], &mut Vec<String>),
    errors: &mut Vec<String>,
) {
    let Some(Value::Array(items)) = value_at(value, path) else {
        return;
    };
    for (index, item) in items.iter().enumerate() {
        let item_path = indexed_path(path, index);
        validate_field_set_with_owned_prefix(spec.schema, spec.build, item, &item_path, errors);
        reject_unknown_fields_owned(Some(item), &item_path, spec.allowed_fields, errors);
        validate_nested(item, &item_path, errors);
    }
}

pub(super) fn validate_value_array_items(
    value: &Value,
    path: &[&str],
    prefix: &[&str],
    spec: ValueArraySpec,
    errors: &mut Vec<String>,
) {
    let prefix = prefix
        .iter()
        .map(|segment| (*segment).to_string())
        .collect::<Vec<_>>();
    validate_value_array_items_with_owned_prefix(value, path, &prefix, spec, errors);
}

pub(super) fn validate_value_array_items_with_owned_prefix(
    value: &Value,
    path: &[&str],
    prefix: &[String],
    spec: ValueArraySpec,
    errors: &mut Vec<String>,
) {
    let Some(Value::Array(items)) = value_at(value, path) else {
        return;
    };
    for (index, item) in items.iter().enumerate() {
        let mut item_path = prefix.to_vec();
        item_path.extend(indexed_path(path, index));
        let wrapped = Value::Object(Map::from_iter([(VALUE_FIELD.to_string(), item.clone())]));
        validate_wrapped_value_field(spec.schema, spec.build, &wrapped, &item_path, errors);
    }
}

fn validate_field_set_with_owned_prefix(
    schema: &'static str,
    build: FieldSetBuilder,
    value: &Value,
    prefix: &[String],
    errors: &mut Vec<String>,
) {
    match build() {
        Ok(fields) => {
            if let Err(error) = fields.validate_json(JSON_CONTRACT_PROCESSING, value) {
                push_owned_field_extraction_errors(errors, prefix, error);
            }
        }
        Err(error) => errors.push(format!(
            "{schema} contract field definitions failed: {error}"
        )),
    }
}

fn validate_wrapped_value_field(
    schema: &'static str,
    build: FieldSetBuilder,
    value: &Value,
    prefix: &[String],
    errors: &mut Vec<String>,
) {
    match build() {
        Ok(fields) => {
            if let Err(error) = fields.validate_json(JSON_CONTRACT_PROCESSING, value) {
                push_wrapped_value_errors(errors, prefix, error);
            }
        }
        Err(error) => errors.push(format!(
            "{schema} contract field definitions failed: {error}"
        )),
    }
}

pub(super) fn reject_unknown_fields(
    value: Option<&Value>,
    path: &[&str],
    allowed: &[&str],
    errors: &mut Vec<String>,
) {
    let prefix = path
        .iter()
        .map(|segment| (*segment).to_string())
        .collect::<Vec<_>>();
    reject_unknown_fields_owned(value, &prefix, allowed, errors);
}

pub(super) fn reject_unknown_fields_owned(
    value: Option<&Value>,
    path: &[String],
    allowed: &[&str],
    errors: &mut Vec<String>,
) {
    let Some(Value::Object(object)) = value else {
        return;
    };
    for key in object.keys() {
        if !allowed.contains(&key.as_str()) {
            let mut unknown_path = path.to_vec();
            unknown_path.push(key.clone());
            errors.push(format!(
                "{}: additional property is not allowed",
                json_pointer(&unknown_path)
            ));
        }
    }
}

pub(super) fn expect_bool_value(
    value: &Value,
    path: &[&str],
    expected: bool,
    errors: &mut Vec<String>,
) {
    if value_at(value, path).and_then(Value::as_bool) == Some(expected) {
        return;
    }
    errors.push(format!(
        "{}: expected const {expected}",
        json_pointer_str(path)
    ));
}

pub(super) fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        let Value::Object(object) = current else {
            return None;
        };
        current = object.get(*segment)?;
    }
    Some(current)
}

pub(super) fn operation_name(value: &Value) -> Option<&str> {
    value.get("operation").and_then(Value::as_str)
}

fn indexed_path(path: &[&str], index: usize) -> Vec<String> {
    let mut item_path = path
        .iter()
        .map(|segment| (*segment).to_string())
        .collect::<Vec<_>>();
    item_path.push(index.to_string());
    item_path
}

pub(super) fn schema_result(
    schema: &'static str,
    errors: Vec<String>,
) -> Result<(), SchemaValidationError> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(SchemaValidationError { schema, errors })
    }
}
