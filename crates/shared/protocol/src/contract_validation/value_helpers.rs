use serde_json::Value;

use crate::SchemaValidationError;

use super::formatting::json_pointer_str;

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

pub(super) fn expect_string_value(
    value: &Value,
    path: &[&str],
    expected: &str,
    errors: &mut Vec<String>,
) {
    if value_at(value, path).and_then(Value::as_str) == Some(expected) {
        return;
    }
    errors.push(format!(
        "{}: expected const {expected:?}",
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
