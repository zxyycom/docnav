use std::num::NonZeroU32;

use docnav_parameter_resolution::{
    ids, ParameterResolution, ParameterSourceKind, MAX_PAGINATION_LIMIT,
};
use docnav_protocol::{Operation, PositiveInteger};
use docnav_typed_fields::{FieldIdentity, TypedValue, ValidationReason};
use serde_json::{json, Value};

use crate::{NavigationError, NavigationOutputMode};

pub(super) fn validation_error_for_identity(identity: &str) -> NavigationError {
    NavigationError::invalid_request(field_label(identity), validation_reason(identity))
}

pub(super) fn optional_document_positive(
    operation: Operation,
    resolution: &ParameterResolution,
    identity: &str,
) -> Result<Option<PositiveInteger>, NavigationError> {
    if !uses_document_window(operation) {
        return Ok(None);
    }
    Ok(Some(required_positive_value(resolution, identity)?))
}

pub(super) fn optional_document_limit(
    operation: Operation,
    resolution: &ParameterResolution,
) -> Result<Option<PositiveInteger>, NavigationError> {
    if !uses_document_window(operation) {
        return Ok(None);
    }
    let enabled = required_bool_value(resolution, ids::PAGINATION_ENABLED)?;
    let limit = required_positive_value(resolution, ids::LIMIT)?;
    Ok(Some(if enabled {
        limit
    } else {
        max_pagination_limit()
    }))
}

pub(super) fn uses_document_window(operation: Operation) -> bool {
    operation != Operation::Info
}

pub(super) fn required_string_value(
    resolution: &ParameterResolution,
    identity: &str,
) -> Result<String, NavigationError> {
    optional_string_value(resolution, identity)?
        .ok_or_else(|| NavigationError::internal("missing-resolved-parameter"))
}

pub(super) fn optional_string_value(
    resolution: &ParameterResolution,
    identity: &str,
) -> Result<Option<String>, NavigationError> {
    let Some(value) = resolution.value(&identity_key(identity)?) else {
        return Ok(None);
    };
    match &value.value {
        TypedValue::String(value) => Ok(Some(value.clone())),
        TypedValue::Null => Ok(None),
        _ => Err(NavigationError::internal("unexpected-parameter-type")),
    }
}

pub(super) fn required_output_value(
    resolution: &ParameterResolution,
) -> Result<NavigationOutputMode, NavigationError> {
    let output = required_string_value(resolution, ids::OUTPUT)?;
    NavigationOutputMode::parse(&output).map_err(|_| validation_error_for_identity(ids::OUTPUT))
}

pub(super) fn resolved_source_label(
    resolution: &ParameterResolution,
    identity: &str,
) -> Option<&'static str> {
    resolution
        .value(&identity_key(identity).ok()?)
        .map(|value| source_label(value.source.kind))
}

pub(super) fn identity_key(identity: &str) -> Result<FieldIdentity, NavigationError> {
    FieldIdentity::new(identity)
        .map_err(|_| NavigationError::internal("invalid-parameter-identity"))
}

pub(super) fn source_label(source: ParameterSourceKind) -> &'static str {
    match source {
        ParameterSourceKind::DirectInput => "explicit",
        ParameterSourceKind::ProjectConfig => "project",
        ParameterSourceKind::UserConfig => "user",
        ParameterSourceKind::Default => "built_in",
    }
}

pub(super) fn typed_value_to_json(value: &TypedValue) -> Value {
    match value {
        TypedValue::String(value) => json!(value),
        TypedValue::Integer(value) => json!(value),
        TypedValue::Number(value) => json!(value),
        TypedValue::Boolean(value) => json!(value),
        TypedValue::Array(value) => Value::Array(value.clone()),
        TypedValue::Object(value) => Value::Object(value.clone()),
        TypedValue::Json(value) => value.clone(),
        TypedValue::Null => Value::Null,
    }
}

pub(super) fn validation_reason_code(reason: &ValidationReason) -> &'static str {
    match reason {
        ValidationReason::WrongType { .. } => "type_mismatch",
        ValidationReason::BelowMinimum { .. } | ValidationReason::AboveMaximum { .. } => {
            "range_invalid"
        }
        ValidationReason::MissingRequired => "missing_required",
        ValidationReason::DisallowedEnumValue { .. } => "enum_invalid",
        ValidationReason::BelowMinimumLength { .. }
        | ValidationReason::AboveMaximumLength { .. } => "length_invalid",
        ValidationReason::RegexMismatch { .. } => "pattern_invalid",
        ValidationReason::DuplicateArrayItem { .. } => "duplicate_item",
    }
}

fn required_bool_value(
    resolution: &ParameterResolution,
    identity: &str,
) -> Result<bool, NavigationError> {
    let value = resolution
        .value(&identity_key(identity)?)
        .ok_or_else(|| NavigationError::internal("missing-resolved-parameter"))?;
    let TypedValue::Boolean(value) = value.value else {
        return Err(NavigationError::internal("unexpected-parameter-type"));
    };
    Ok(value)
}

fn required_positive_value(
    resolution: &ParameterResolution,
    identity: &str,
) -> Result<PositiveInteger, NavigationError> {
    let value = resolution
        .value(&identity_key(identity)?)
        .ok_or_else(|| NavigationError::internal("missing-resolved-parameter"))?;
    let TypedValue::Integer(value) = value.value else {
        return Err(NavigationError::internal("unexpected-parameter-type"));
    };
    u32::try_from(value)
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or_else(|| validation_error_for_identity(identity))
}

fn max_pagination_limit() -> PositiveInteger {
    NonZeroU32::new(MAX_PAGINATION_LIMIT).expect("u32::MAX is a positive integer")
}

fn field_label(identity: &str) -> &'static str {
    match identity {
        ids::ADAPTER => "--adapter",
        ids::LIMIT => "--limit",
        ids::OUTPUT => "--output",
        ids::PAGE => "--page",
        ids::PAGINATION_ENABLED => "--pagination",
        ids::PATH => "path",
        ids::QUERY => "--query",
        ids::REF => "--ref",
        _ => "parameter",
    }
}

fn validation_reason(identity: &str) -> String {
    match identity {
        ids::LIMIT => "--limit must be a positive integer".to_owned(),
        ids::OUTPUT => format!(
            "invalid --output: accepted values: {}",
            NavigationOutputMode::ACCEPTED_VALUES.join(", ")
        ),
        ids::PAGE => "--page must be a positive integer".to_owned(),
        ids::PAGINATION_ENABLED => "--pagination must be enabled or disabled".to_owned(),
        ids::PATH => "path value must not be empty".to_owned(),
        ids::QUERY => "--query value must not be empty".to_owned(),
        ids::REF => "--ref value must not be empty".to_owned(),
        ids::ADAPTER => "--adapter value must not be empty".to_owned(),
        _ => "parameter resolution validation failed".to_owned(),
    }
}
