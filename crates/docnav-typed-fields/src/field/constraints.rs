use serde_json::Value;

use crate::metadata::{ActualValueKind, BuildError, FieldConstraints, FieldPath, TypedValue};
use crate::range::{FieldBound, FieldBoundKind, FieldNumericBound, FieldNumericRange, FieldRange};

pub(super) fn value_at_path<'a>(root: &'a Value, path: &FieldPath) -> Option<&'a Value> {
    let mut current = root;
    for segment in path.raw_segments() {
        let Value::Object(object) = current else {
            return None;
        };
        current = object.get(segment)?;
    }
    Some(current)
}

pub(super) fn validate_numeric_range(constraints: &FieldConstraints) -> Result<(), BuildError> {
    match &constraints.numeric_range {
        FieldNumericRange::None => Ok(()),
        FieldNumericRange::Integer(range) => validate_range(range),
        FieldNumericRange::Number(range) => {
            if [range.minimum, range.maximum]
                .into_iter()
                .flatten()
                .any(|bound| !bound.value.is_finite())
            {
                return Err(BuildError::NonFiniteRangeBound);
            }
            validate_range(range)
        }
    }
}

pub(super) fn validate_length_range(constraints: &FieldConstraints) -> Result<(), BuildError> {
    constraints
        .length_range
        .as_ref()
        .map_or(Ok(()), validate_range)
}

fn validate_range<T>(range: &FieldRange<T>) -> Result<(), BuildError>
where
    T: Copy + PartialOrd + PartialEq,
{
    let (Some(minimum), Some(maximum)) = (range.minimum, range.maximum) else {
        return Ok(());
    };
    if minimum.value > maximum.value {
        return Err(BuildError::InvalidRange);
    }
    if minimum.value == maximum.value && !(minimum.is_closed() && maximum.is_closed()) {
        return Err(BuildError::InvalidRange);
    }
    Ok(())
}

pub(super) fn compile_regex_pattern(
    constraints: &FieldConstraints,
) -> Result<Option<regex::Regex>, BuildError> {
    let Some(pattern) = &constraints.regex else {
        return Ok(None);
    };
    regex::Regex::new(pattern)
        .map(Some)
        .map_err(|error| BuildError::InvalidRegexPattern {
            pattern: pattern.clone(),
            error: error.to_string(),
        })
}

pub(super) fn actual_value_kind(value: &Value) -> ActualValueKind {
    match value {
        Value::String(_) => ActualValueKind::String,
        Value::Number(number) if number.as_i64().is_some() => ActualValueKind::Integer,
        Value::Number(_) => ActualValueKind::Number,
        Value::Bool(_) => ActualValueKind::Boolean,
        Value::Array(_) => ActualValueKind::Array,
        Value::Object(_) => ActualValueKind::Object,
        Value::Null => ActualValueKind::Null,
    }
}

pub(super) enum NumericRangeViolation {
    Below(FieldNumericBound),
    Above(FieldNumericBound),
}

pub(super) fn numeric_range_violation(
    constraints: &FieldConstraints,
    value: &TypedValue,
) -> Option<NumericRangeViolation> {
    match (&constraints.numeric_range, value) {
        (FieldNumericRange::None, _) => None,
        (FieldNumericRange::Integer(range), TypedValue::Integer(value)) => {
            range_violation(*value, range, FieldNumericBound::Integer)
        }
        (FieldNumericRange::Number(range), TypedValue::Number(value)) => {
            range_violation(*value, range, FieldNumericBound::Number)
        }
        _ => None,
    }
}

fn range_violation<T>(
    value: T,
    range: &FieldRange<T>,
    bound: fn(FieldBound<T>) -> FieldNumericBound,
) -> Option<NumericRangeViolation>
where
    T: Copy + PartialOrd + PartialEq,
{
    if let Some(minimum) = range.minimum {
        if below_minimum(value, minimum) {
            return Some(NumericRangeViolation::Below(bound(minimum)));
        }
    }
    if let Some(maximum) = range.maximum {
        if above_maximum(value, maximum) {
            return Some(NumericRangeViolation::Above(bound(maximum)));
        }
    }
    None
}

pub(super) fn value_length(value: &TypedValue) -> Option<u64> {
    match value {
        TypedValue::String(value) => Some(value.chars().count() as u64),
        TypedValue::Array(value) => Some(value.len() as u64),
        _ => None,
    }
}

pub(super) fn below_minimum<T>(value: T, minimum: FieldBound<T>) -> bool
where
    T: PartialOrd + PartialEq,
{
    value < minimum.value || (value == minimum.value && minimum.kind == FieldBoundKind::Open)
}

pub(super) fn above_maximum<T>(value: T, maximum: FieldBound<T>) -> bool
where
    T: PartialOrd + PartialEq,
{
    value > maximum.value || (value == maximum.value && maximum.kind == FieldBoundKind::Open)
}
