use serde_json::{Map, Value};

use docnav_typed_fields::{FieldDefSet, FieldDefSetBuildError, JsonFieldSet};

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
            if let Err(error) = JsonFieldSet::new(&fields).validate(JSON_CONTRACT_PROCESSING, value)
            {
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
        validate_strict_object(item, &item_path, spec, errors);
        validate_nested(item, &item_path, errors);
    }
}

pub(super) fn validate_object_at(
    value: &Value,
    path: &[&str],
    spec: ObjectArraySpec,
    validate_nested: impl Fn(&Value, &[String], &mut Vec<String>),
    errors: &mut Vec<String>,
) {
    let Some(object) = value_at(value, path) else {
        return;
    };
    let prefix = path
        .iter()
        .map(|segment| (*segment).to_string())
        .collect::<Vec<_>>();
    validate_strict_object(object, &prefix, spec, errors);
    validate_nested(object, &prefix, errors);
}

pub(super) fn validate_object_at_with_owned_prefix(
    value: &Value,
    path: &[&str],
    prefix: &[String],
    spec: ObjectArraySpec,
    errors: &mut Vec<String>,
) {
    let Some(object) = value_at(value, path) else {
        return;
    };
    let object_path = extend_owned_path(prefix, path);
    validate_strict_object(object, &object_path, spec, errors);
}

pub(super) fn validate_object_array_items_with_owned_prefix(
    value: &Value,
    path: &[&str],
    prefix: &[String],
    spec: ObjectArraySpec,
    errors: &mut Vec<String>,
) {
    let Some(Value::Array(items)) = value_at(value, path) else {
        return;
    };
    for (index, item) in items.iter().enumerate() {
        let mut item_path = extend_owned_path(prefix, path);
        item_path.push(index.to_string());
        validate_strict_object(item, &item_path, spec, errors);
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
            if let Err(error) = JsonFieldSet::new(&fields).validate(JSON_CONTRACT_PROCESSING, value)
            {
                push_owned_field_extraction_errors(errors, prefix, error);
            }
        }
        Err(error) => errors.push(format!(
            "{schema} contract field definitions failed: {error}"
        )),
    }
}

fn validate_strict_object(
    object: &Value,
    object_path: &[String],
    spec: ObjectArraySpec,
    errors: &mut Vec<String>,
) {
    validate_field_set_with_owned_prefix(spec.schema, spec.build, object, object_path, errors);
    reject_unknown_fields_with_owned_prefix(
        StrictObjectCheck {
            schema: spec.schema,
            build: spec.build,
            value: object,
            path: &[],
            prefix: object_path,
        },
        errors,
    );
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
            if let Err(error) = JsonFieldSet::new(&fields).validate(JSON_CONTRACT_PROCESSING, value)
            {
                push_wrapped_value_errors(errors, prefix, error);
            }
        }
        Err(error) => errors.push(format!(
            "{schema} contract field definitions failed: {error}"
        )),
    }
}

pub(super) fn reject_unknown_fields(
    schema: &'static str,
    build: FieldSetBuilder,
    value: &Value,
    path: &[&str],
    errors: &mut Vec<String>,
) {
    reject_unknown_fields_with_owned_prefix(
        StrictObjectCheck {
            schema,
            build,
            value,
            path,
            prefix: &[],
        },
        errors,
    );
}

struct StrictObjectCheck<'a> {
    schema: &'static str,
    build: FieldSetBuilder,
    value: &'a Value,
    path: &'a [&'a str],
    prefix: &'a [String],
}

fn reject_unknown_fields_with_owned_prefix(check: StrictObjectCheck<'_>, errors: &mut Vec<String>) {
    match (check.build)() {
        Ok(fields) => {
            match JsonFieldSet::new(&fields).unused_fields(
                JSON_CONTRACT_PROCESSING,
                check.value,
                check.path.iter().copied(),
            ) {
                Ok(Value::Object(unused)) => {
                    for key in unused.keys() {
                        let mut unknown_path = check.prefix.to_vec();
                        unknown_path
                            .extend(check.path.iter().map(|segment| (*segment).to_string()));
                        unknown_path.push(key.clone());
                        errors.push(format!(
                            "{}: additional property is not allowed",
                            json_pointer(&unknown_path)
                        ));
                    }
                }
                Ok(_) => {}
                Err(error) => errors.push(error.to_string()),
            }
        }
        Err(error) => errors.push(format!(
            "{} contract field definitions failed: {error}",
            check.schema
        )),
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

pub(super) fn extend_owned_path(prefix: &[String], path: &[&str]) -> Vec<String> {
    let mut extended = prefix.to_vec();
    extended.extend(path.iter().map(|segment| (*segment).to_string()));
    extended
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
