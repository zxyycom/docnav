use std::num::NonZeroU32;

use cli_config_resolution::{
    CandidateInvalidReason, DiagnosticReason, FieldDefSet, FieldIdentity, FieldResolution,
    ResolutionDiagnostic, ResolutionResult, TypedValue, ValidationReason, ValueKind,
};
use docnav_protocol::{Operation, PositiveInteger};
use serde_json::{json, Value};

use crate::{NavigationError, NavigationOutputMode};

use super::{ids, resolution, MAX_PAGINATION_LIMIT};

pub(super) fn validation_error_for_identity(identity: &str) -> NavigationError {
    NavigationError::invalid_request(field_label(identity), validation_reason(identity))
}

pub(super) fn uses_document_window(operation: Operation) -> bool {
    operation != Operation::Info
}

pub(super) fn required_string_value(
    fields: &FieldDefSet,
    result: &ResolutionResult,
    identity: &str,
) -> Result<String, NavigationError> {
    optional_string_value(fields, result, identity)?
        .ok_or_else(|| NavigationError::internal("missing-resolved-parameter"))
}

pub(super) fn optional_string_value(
    fields: &FieldDefSet,
    result: &ResolutionResult,
    identity: &str,
) -> Result<Option<String>, NavigationError> {
    let Some(value) = resolved_value(fields, result, identity)? else {
        return Ok(None);
    };
    match value {
        TypedValue::String(value) => Ok(Some(value.clone())),
        TypedValue::Null => Ok(None),
        _ => Err(NavigationError::internal("unexpected-parameter-type")),
    }
}

pub(super) fn required_output_value_for_identity(
    fields: &FieldDefSet,
    result: &ResolutionResult,
    identity: &str,
) -> Result<NavigationOutputMode, NavigationError> {
    let output = required_string_value(fields, result, identity)?;
    NavigationOutputMode::parse(&output).map_err(|_| validation_error_for_identity(identity))
}

pub(super) fn optional_integer_value(
    fields: &FieldDefSet,
    result: &ResolutionResult,
    identity: &str,
) -> Result<Option<i64>, NavigationError> {
    let Some(value) = resolved_value(fields, result, identity)? else {
        return Ok(None);
    };
    match value {
        TypedValue::Integer(value) => Ok(Some(*value)),
        TypedValue::Null => Ok(None),
        _ => Err(NavigationError::internal("unexpected-parameter-type")),
    }
}

pub(super) fn resolved_source_label(
    result: &ResolutionResult,
    identity: &str,
) -> Option<&'static str> {
    let field = result.fields().get(&identity_key(identity).ok()?)?;
    field_source_label(field)
}

pub(super) fn field_source_label(field: &FieldResolution) -> Option<&'static str> {
    let candidate = field
        .trace()
        .selected
        .as_ref()
        .or(field.trace().default_fallback.as_ref())?;
    Some(resolution::source_label(
        &candidate.source_id,
        &candidate.source_kind,
    ))
}

pub(super) fn diagnostic_source_label(diagnostic: &ResolutionDiagnostic) -> Option<&'static str> {
    Some(resolution::source_label(
        diagnostic.source_id.as_ref()?,
        diagnostic.source_kind.as_ref()?,
    ))
}

pub(super) fn resolution_reason_code(diagnostic: &ResolutionDiagnostic) -> &'static str {
    match &diagnostic.reason {
        DiagnosticReason::InvalidCandidate(CandidateInvalidReason::Decode(_))
        | DiagnosticReason::InvalidCandidate(CandidateInvalidReason::Shape { .. }) => {
            "type_mismatch"
        }
        DiagnosticReason::InvalidCandidate(CandidateInvalidReason::Validation(failure))
        | DiagnosticReason::FinalValidation(failure)
        | DiagnosticReason::MissingRequired(failure) => validation_reason_code(&failure.reason),
        DiagnosticReason::MergeConflict(_) => "conflict",
    }
}

pub(super) fn identity_key(identity: &str) -> Result<FieldIdentity, NavigationError> {
    FieldIdentity::new(identity)
        .map_err(|_| NavigationError::internal("invalid-parameter-identity"))
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

pub(super) fn projected_field_value<'a>(
    fields: &FieldDefSet,
    identity: &FieldIdentity,
    resolved: &'a FieldResolution,
) -> Option<&'a TypedValue> {
    let value = resolved.value()?;
    let field = fields
        .field(identity)
        .expect("resolution fields belong to the canonical field set");
    if matches!(value, TypedValue::Null)
        && !field.constraints().required
        && field.value_kind() != ValueKind::Json
    {
        None
    } else {
        Some(value)
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

fn resolved_value<'a>(
    fields: &FieldDefSet,
    result: &'a ResolutionResult,
    identity: &str,
) -> Result<Option<&'a TypedValue>, NavigationError> {
    let identity = identity_key(identity)?;
    Ok(result
        .fields()
        .get(&identity)
        .and_then(|resolved| projected_field_value(fields, &identity, resolved)))
}

pub(super) fn required_bool_value(
    fields: &FieldDefSet,
    result: &ResolutionResult,
    identity: &str,
) -> Result<bool, NavigationError> {
    let value = resolved_value(fields, result, identity)?
        .ok_or_else(|| NavigationError::internal("missing-resolved-parameter"))?;
    let TypedValue::Boolean(value) = value else {
        return Err(NavigationError::internal("unexpected-parameter-type"));
    };
    Ok(*value)
}

pub(super) fn required_positive_value(
    fields: &FieldDefSet,
    result: &ResolutionResult,
    identity: &str,
) -> Result<PositiveInteger, NavigationError> {
    let value = resolved_value(fields, result, identity)?
        .ok_or_else(|| NavigationError::internal("missing-resolved-parameter"))?;
    let TypedValue::Integer(value) = value else {
        return Err(NavigationError::internal("unexpected-parameter-type"));
    };
    u32::try_from(*value)
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or_else(|| validation_error_for_identity(identity))
}

pub(super) fn max_pagination_limit() -> PositiveInteger {
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
