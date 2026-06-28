use docnav_typed_fields::{
    ActualValueKind, FieldExtractionError, ValidationFailure, ValidationReason, ValueKind,
};
use serde_json::Value;

pub(super) fn push_field_extraction_errors(
    errors: &mut Vec<String>,
    prefix: &[&str],
    error: FieldExtractionError,
) {
    let owned_prefix = prefix
        .iter()
        .map(|segment| (*segment).to_string())
        .collect::<Vec<_>>();
    push_owned_field_extraction_errors(errors, &owned_prefix, error);
}

pub(super) fn push_owned_field_extraction_errors(
    errors: &mut Vec<String>,
    prefix: &[String],
    error: FieldExtractionError,
) {
    match error {
        FieldExtractionError::Validation(validation_errors) => {
            for failure in validation_errors.into_failures() {
                errors.push(format_validation_failure(prefix, &failure));
            }
        }
        error => errors.push(error.to_string()),
    }
}

pub(super) fn push_wrapped_value_errors(
    errors: &mut Vec<String>,
    prefix: &[String],
    error: FieldExtractionError,
) {
    match error {
        FieldExtractionError::Validation(validation_errors) => {
            for failure in validation_errors.into_failures() {
                errors.push(format!(
                    "{}: {}",
                    json_pointer(prefix),
                    validation_reason_text(&failure.reason)
                ));
            }
        }
        error => errors.push(error.to_string()),
    }
}

fn format_validation_failure(prefix: &[String], failure: &ValidationFailure) -> String {
    let mut path = prefix.to_vec();
    path.extend(failure.path.segments().into_iter().map(str::to_string));
    format!(
        "{}: {}",
        json_pointer(&path),
        validation_reason_text(&failure.reason)
    )
}

fn validation_reason_text(reason: &ValidationReason) -> String {
    match reason {
        ValidationReason::MissingRequired => "missing required field".to_string(),
        ValidationReason::WrongType { expected, actual } => {
            format!(
                "expected {}, got {}",
                value_kind_name(*expected),
                actual_kind_name(*actual)
            )
        }
        ValidationReason::DisallowedEnumValue { allowed } => {
            format!("value is not one of {}", Value::Array(allowed.clone()))
        }
        ValidationReason::BelowMinimum { minimum } => {
            format!("value is below minimum {}", numeric_bound_value(minimum))
        }
        ValidationReason::AboveMaximum { maximum } => {
            format!("value is above maximum {}", numeric_bound_value(maximum))
        }
        ValidationReason::BelowMinimumLength { minimum } => {
            format!("length is below minimum {}", minimum.value)
        }
        ValidationReason::AboveMaximumLength { maximum } => {
            format!("length is above maximum {}", maximum.value)
        }
        ValidationReason::RegexMismatch { pattern } => {
            format!("value does not match pattern {pattern:?}")
        }
        ValidationReason::DuplicateArrayItem {
            first_index,
            duplicate_index,
        } => format!("array item at index {duplicate_index} duplicates index {first_index}"),
    }
}

fn value_kind_name(kind: ValueKind) -> &'static str {
    match kind {
        ValueKind::String => "string",
        ValueKind::Integer => "integer",
        ValueKind::Number => "number",
        ValueKind::Boolean => "boolean",
        ValueKind::Array => "array",
        ValueKind::Object => "object",
    }
}

fn actual_kind_name(kind: ActualValueKind) -> &'static str {
    match kind {
        ActualValueKind::String => "string",
        ActualValueKind::Integer => "integer",
        ActualValueKind::Number => "number",
        ActualValueKind::Boolean => "boolean",
        ActualValueKind::Array => "array",
        ActualValueKind::Object => "object",
        ActualValueKind::Null => "null",
    }
}

fn numeric_bound_value(bound: &docnav_typed_fields::FieldNumericBound) -> String {
    match bound {
        docnav_typed_fields::FieldNumericBound::Integer(value) => value.value.to_string(),
        docnav_typed_fields::FieldNumericBound::Number(value) => value.value.to_string(),
    }
}

pub(super) fn json_pointer(path: &[String]) -> String {
    if path.is_empty() {
        return "/".to_string();
    }
    path.iter()
        .map(|segment| format!("/{}", escape_json_pointer_segment(segment)))
        .collect::<String>()
}

pub(super) fn json_pointer_str(path: &[&str]) -> String {
    let owned = path
        .iter()
        .map(|segment| (*segment).to_string())
        .collect::<Vec<_>>();
    json_pointer(&owned)
}

fn escape_json_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}
