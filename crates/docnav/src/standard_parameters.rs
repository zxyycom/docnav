use std::num::NonZeroU32;

use docnav_adapter_contracts::NativeOptionSpec;
use docnav_protocol::{Operation, OptionEntry, Options, PositiveInteger};
use docnav_standard_parameters::{
    ids, StandardParameterResolution, StandardParameterSourceKind, MAX_PAGINATION_LIMIT,
};
use docnav_typed_fields::{FieldIdentity, TypedValue};
use serde_json::{json, Value};

use crate::cli::{DocumentCommand, OutputMode};
use crate::config::{ConfigContext, ResolvedValue};
use crate::error::{AppError, AppResult};
use crate::runtime::{ResolvedDocumentDefaults, ResolvedPaginationDefaults};

mod definitions;
mod diagnostics;
#[cfg(test)]
mod tests;

use ids::{
    ADAPTER as ID_ADAPTER, LIMIT as ID_LIMIT, OUTPUT as ID_OUTPUT, PAGE as ID_PAGE,
    PAGINATION_ENABLED as ID_PAGINATION_ENABLED, PATH as ID_PATH, QUERY as ID_QUERY, REF as ID_REF,
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
    pub(crate) options: Option<Options>,
    pub(crate) output: OutputMode,
    pub(crate) adapter: Option<String>,
    pub(crate) defaults: ResolvedDocumentDefaults,
}

pub(crate) fn resolve_core_document_parameters(
    command: &DocumentCommand,
    context: &ConfigContext,
) -> AppResult<ResolvedCoreDocumentParameters> {
    let resolution = definitions::resolve_core_for_operation(command, context)?;
    diagnostics::first_validation_error(&resolution)?;
    resolved_core_document_parameters_from_resolution(command.operation, &resolution, None)
}

pub(crate) fn resolve_registered_native_options(
    command: &DocumentCommand,
    context: &ConfigContext,
    native_options: &[NativeOptionSpec],
) -> AppResult<Option<Options>> {
    let mut options = Options::new();
    for option in native_option_keys_for_operation(command.operation, native_options) {
        let Some(resolved) = native_option_value(command, context, option.key) else {
            continue;
        };
        options.insert_entry(OptionEntry {
            identity: option.identity.to_owned(),
            owner: option.owner.to_owned(),
            namespace: option.namespace.to_owned(),
            key: option.key.to_owned(),
            source: resolved.source.to_owned(),
            type_variant: option.value_kind().as_str().to_owned(),
            value: resolved.value.clone(),
        });
    }
    Ok((!options.is_empty()).then_some(options))
}

struct NativeOptionResolvedValue {
    value: Value,
    source: &'static str,
}

fn native_option_keys_for_operation(
    operation: Operation,
    native_options: &[NativeOptionSpec],
) -> Vec<NativeOptionSpec> {
    native_options
        .iter()
        .copied()
        .filter(|option| option.applies_to(operation))
        .collect()
}

fn native_option_value(
    command: &DocumentCommand,
    context: &ConfigContext,
    key: &str,
) -> Option<NativeOptionResolvedValue> {
    if key == "max_heading_level" {
        if let Some(max_heading_level) = command.max_heading_level {
            return Some(NativeOptionResolvedValue {
                value: json!(max_heading_level.get()),
                source: "direct",
            });
        }
    }
    if let Some(value) = context.project_config.options.value_for_key(key) {
        return Some(NativeOptionResolvedValue {
            value: value.clone(),
            source: "project_config",
        });
    }
    context
        .user_config
        .options
        .value_for_key(key)
        .cloned()
        .map(|value| NativeOptionResolvedValue {
            value,
            source: "user_config",
        })
}

fn resolved_core_document_parameters_from_resolution(
    operation: Operation,
    resolution: &StandardParameterResolution,
    options: Option<Options>,
) -> AppResult<ResolvedCoreDocumentParameters> {
    Ok(ResolvedCoreDocumentParameters {
        path: required_string_value(resolution, ID_PATH)?,
        ref_id: optional_string_value(resolution, ID_REF)?,
        query: optional_string_value(resolution, ID_QUERY)?,
        page: optional_document_positive(operation, resolution, ID_PAGE)?,
        limit: optional_document_limit(operation, resolution)?,
        options,
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

fn optional_document_limit(
    operation: Operation,
    resolution: &StandardParameterResolution,
) -> AppResult<Option<PositiveInteger>> {
    if !uses_document_window(operation) {
        return Ok(None);
    }
    let enabled = required_bool_value(resolution, ID_PAGINATION_ENABLED)?;
    let limit = required_positive_value(resolution, ID_LIMIT)?;
    Ok(Some(if enabled {
        limit
    } else {
        max_pagination_limit()
    }))
}

fn resolved_document_defaults(
    operation: Operation,
    resolution: &StandardParameterResolution,
) -> AppResult<ResolvedDocumentDefaults> {
    Ok(ResolvedDocumentDefaults {
        adapter: resolved_value(resolution, ID_ADAPTER).unwrap_or_else(ResolvedValue::unset),
        pagination: optional_document_pagination_defaults(operation, resolution)?,
        output: required_resolved_value(resolution, ID_OUTPUT)?,
        page: optional_document_resolved_value(operation, resolution, ID_PAGE)?,
    })
}

fn optional_document_pagination_defaults(
    operation: Operation,
    resolution: &StandardParameterResolution,
) -> AppResult<Option<ResolvedPaginationDefaults>> {
    if !uses_document_window(operation) {
        return Ok(None);
    }
    Ok(Some(ResolvedPaginationDefaults {
        enabled: required_resolved_value(resolution, ID_PAGINATION_ENABLED)?,
        limit: required_resolved_value(resolution, ID_LIMIT)?,
    }))
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

fn required_bool_value(
    resolution: &StandardParameterResolution,
    identity: &str,
) -> AppResult<bool> {
    let value = resolution.value(&identity_key(identity)?).ok_or_else(|| {
        AppError::internal(format!("missing-resolved-standard-parameter:{identity}"))
    })?;
    let TypedValue::Boolean(value) = value.value else {
        return Err(AppError::internal(format!(
            "unexpected-standard-parameter-type:{identity}"
        )));
    };
    Ok(value)
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
        .ok_or_else(|| diagnostics::validation_error_for_identity(identity))?;
    Ok(value)
}

fn max_pagination_limit() -> PositiveInteger {
    NonZeroU32::new(MAX_PAGINATION_LIMIT).expect("u32::MAX is a positive integer")
}

fn required_output_value(resolution: &StandardParameterResolution) -> AppResult<OutputMode> {
    let output = required_string_value(resolution, ID_OUTPUT)?;
    output
        .parse()
        .map_err(|_| diagnostics::validation_error_for_identity(ID_OUTPUT))
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
