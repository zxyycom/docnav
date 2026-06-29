use std::num::NonZeroU32;

use docnav_diagnostics::{
    typed_codes, DiagnosticRecordDraft, DiagnosticSource, FieldReasonDetails,
};
use docnav_protocol::{Operation, PositiveInteger};
use docnav_standard_parameters::{
    ids, StandardParameterHandoff, StandardParameterResolution, StandardParameterSourceKind,
    StandardParameterValidationIssue,
};
use docnav_typed_fields::{FieldIdentity, TypedValue};
use serde_json::{json, Value};

use crate::cli::{DocumentCommand, OutputMode};
use crate::config::{ConfigContext, ResolvedValue};
use crate::error::{AppError, AppResult};
use crate::runtime::ResolvedDocumentDefaults;

mod definitions;
#[cfg(test)]
mod tests;

use ids::{
    ADAPTER as ID_ADAPTER, LIMIT as ID_LIMIT, OUTPUT as ID_OUTPUT, PAGE as ID_PAGE,
    PATH as ID_PATH, QUERY as ID_QUERY, REF as ID_REF,
};

pub(crate) const DEFAULT_LIMIT_TEXT: &str = "6000";
pub(crate) const DEFAULT_OUTPUT_TEXT: &str = "readable-view";
pub(crate) const DEFAULT_PAGE_TEXT: &str = "1";

impl docnav_typed_fields::FieldStringEnum for OutputMode {
    fn variants() -> &'static [Self] {
        &[Self::ReadableView, Self::ReadableJson, Self::ProtocolJson]
    }

    fn as_str(&self) -> &'static str {
        OutputMode::as_str(*self)
    }
}

pub(crate) struct ResolvedCoreDocumentParameters {
    pub(crate) path: String,
    pub(crate) ref_id: Option<String>,
    pub(crate) query: Option<String>,
    pub(crate) page: Option<PositiveInteger>,
    pub(crate) limit: Option<PositiveInteger>,
    pub(crate) output: OutputMode,
    pub(crate) adapter: Option<String>,
    pub(crate) defaults: ResolvedDocumentDefaults,
}

pub(crate) fn resolve_core_document_parameters(
    command: &DocumentCommand,
    context: &ConfigContext,
) -> AppResult<ResolvedCoreDocumentParameters> {
    let resolution = definitions::resolve_for_operation(command, context)?;
    first_validation_error(&resolution)?;
    resolved_core_document_parameters_from_resolution(command.operation, &resolution)
}

fn resolved_core_document_parameters_from_resolution(
    operation: Operation,
    resolution: &StandardParameterResolution,
) -> AppResult<ResolvedCoreDocumentParameters> {
    Ok(ResolvedCoreDocumentParameters {
        path: required_string_value(resolution, ID_PATH)?,
        ref_id: optional_string_value(resolution, ID_REF)?,
        query: optional_string_value(resolution, ID_QUERY)?,
        page: optional_document_positive(operation, resolution, ID_PAGE)?,
        limit: optional_document_positive(operation, resolution, ID_LIMIT)?,
        output: required_output_value(resolution)?,
        adapter: optional_string_value(resolution, ID_ADAPTER)?,
        defaults: resolved_document_defaults(operation, resolution)?,
    })
}

fn optional_document_positive(
    operation: Operation,
    resolution: &StandardParameterResolution,
    identity: &str,
) -> AppResult<Option<PositiveInteger>> {
    if !uses_document_window(operation) {
        return Ok(None);
    }
    Ok(Some(required_positive_value(resolution, identity)?))
}

fn resolved_document_defaults(
    operation: Operation,
    resolution: &StandardParameterResolution,
) -> AppResult<ResolvedDocumentDefaults> {
    Ok(ResolvedDocumentDefaults {
        adapter: resolved_value(resolution, ID_ADAPTER).unwrap_or_else(ResolvedValue::unset),
        limit: optional_document_resolved_value(operation, resolution, ID_LIMIT)?,
        output: required_resolved_value(resolution, ID_OUTPUT)?,
        page: optional_document_resolved_value(operation, resolution, ID_PAGE)?,
    })
}

fn optional_document_resolved_value(
    operation: Operation,
    resolution: &StandardParameterResolution,
    identity: &str,
) -> AppResult<Option<ResolvedValue>> {
    if !uses_document_window(operation) {
        return Ok(None);
    }
    Ok(Some(required_resolved_value(resolution, identity)?))
}

fn uses_document_window(operation: Operation) -> bool {
    operation != Operation::Info
}

fn first_validation_error(resolution: &StandardParameterResolution) -> AppResult<()> {
    if let Some(diagnostic) = resolution
        .diagnostics()
        .iter()
        .find_map(StandardParameterHandoff::as_validation)
    {
        return Err(validation_error(diagnostic));
    }
    Ok(())
}

fn validation_error(diagnostic: &StandardParameterValidationIssue) -> AppError {
    validation_error_for_identity(diagnostic.identity.as_str())
}

fn field_label(identity: &str) -> &'static str {
    match identity {
        ID_ADAPTER => "--adapter",
        ID_LIMIT => "--limit",
        ID_OUTPUT => "--output",
        ID_PAGE => "--page",
        ID_PATH => "path",
        ID_QUERY => "--query",
        ID_REF => "--ref",
        _ => "parameter",
    }
}

fn validation_reason(identity: &str) -> String {
    match identity {
        ID_LIMIT => "--limit must be a positive integer".to_owned(),
        ID_OUTPUT => format!(
            "invalid --output: accepted values: {}",
            OutputMode::ACCEPTED_VALUES.join(", ")
        ),
        ID_PAGE => "--page must be a positive integer".to_owned(),
        ID_PATH => "path value must not be empty".to_owned(),
        ID_QUERY => "--query value must not be empty".to_owned(),
        ID_REF => "--ref value must not be empty".to_owned(),
        ID_ADAPTER => "--adapter value must not be empty".to_owned(),
        _ => "standard parameter validation failed".to_owned(),
    }
}

fn required_string_value(
    resolution: &StandardParameterResolution,
    identity: &str,
) -> AppResult<String> {
    optional_string_value(resolution, identity)?.ok_or_else(|| {
        AppError::internal(format!("missing-resolved-standard-parameter:{identity}"))
    })
}

fn optional_string_value(
    resolution: &StandardParameterResolution,
    identity: &str,
) -> AppResult<Option<String>> {
    let Some(value) = resolution.value(&identity_key(identity)?) else {
        return Ok(None);
    };
    match &value.value {
        TypedValue::String(value) => Ok(Some(value.clone())),
        TypedValue::Null => Ok(None),
        _ => Err(AppError::internal(format!(
            "unexpected-standard-parameter-type:{identity}"
        ))),
    }
}

fn required_positive_value(
    resolution: &StandardParameterResolution,
    identity: &str,
) -> AppResult<PositiveInteger> {
    let value = resolution.value(&identity_key(identity)?).ok_or_else(|| {
        AppError::internal(format!("missing-resolved-standard-parameter:{identity}"))
    })?;
    let TypedValue::Integer(value) = value.value else {
        return Err(AppError::internal(format!(
            "unexpected-standard-parameter-type:{identity}"
        )));
    };
    let value = u32::try_from(value)
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or_else(|| validation_error_for_identity(identity))?;
    Ok(value)
}

fn required_output_value(resolution: &StandardParameterResolution) -> AppResult<OutputMode> {
    let output = required_string_value(resolution, ID_OUTPUT)?;
    output
        .parse()
        .map_err(|_| validation_error_for_identity(ID_OUTPUT))
}

fn required_resolved_value(
    resolution: &StandardParameterResolution,
    identity: &str,
) -> AppResult<ResolvedValue> {
    resolved_value(resolution, identity).ok_or_else(|| {
        AppError::internal(format!("missing-resolved-standard-parameter:{identity}"))
    })
}

fn resolved_value(
    resolution: &StandardParameterResolution,
    identity: &str,
) -> Option<ResolvedValue> {
    let value = resolution.value(&identity_key(identity).ok()?)?;
    let json = typed_value_to_json(&value.value);
    Some(match value.source.kind {
        StandardParameterSourceKind::DirectInput => ResolvedValue::explicit(json),
        StandardParameterSourceKind::ProjectConfig => ResolvedValue::project(json),
        StandardParameterSourceKind::UserConfig => ResolvedValue::user(json),
        StandardParameterSourceKind::Default => ResolvedValue::built_in(json),
    })
}

fn typed_value_to_json(value: &TypedValue) -> Value {
    match value {
        TypedValue::String(value) => json!(value),
        TypedValue::Integer(value) => json!(value),
        TypedValue::Number(value) => json!(value),
        TypedValue::Boolean(value) => json!(value),
        TypedValue::Array(value) => Value::Array(value.clone()),
        TypedValue::Object(value) => Value::Object(value.clone()),
        TypedValue::Null => Value::Null,
    }
}

fn identity_key(identity: &str) -> AppResult<FieldIdentity> {
    FieldIdentity::new(identity)
        .map_err(|error| AppError::internal(format!("invalid-standard-parameter-identity:{error}")))
}

fn validation_error_for_identity(identity: &str) -> AppError {
    AppError::new(invalid_request_record(
        field_label(identity),
        validation_reason(identity),
        DiagnosticSource::with_stage("docnav", "standard-parameters"),
    ))
}

fn invalid_request_record(
    field: &str,
    reason: String,
    source: DiagnosticSource,
) -> DiagnosticRecordDraft {
    DiagnosticRecordDraft::new::<typed_codes::protocol::InvalidRequest>(
        reason.clone(),
        FieldReasonDetails::new(field, reason),
        source,
    )
}
