use std::path::Path;

use docnav_diagnostics::{typed_codes, DiagnosticSource, FieldReasonDetails};
use docnav_protocol::protocol_error_record_draft_with_summary;
use serde_json::{json, Value};

use crate::cli::OutputMode;
use crate::error::{AppError, AppResult};
use crate::project_paths::path_to_slash;

use super::model::{
    ConfigContext, ConfigSource, CoreConfig, ResolvedValue, DEFAULT_LIMIT_CHARS, DEFAULT_OUTPUT,
    SUPPORTED_KEYS,
};

pub fn resolve_adapter(explicit: Option<&str>, context: &ConfigContext) -> ResolvedValue {
    if let Some(adapter) = explicit {
        return ResolvedValue::explicit(json!(adapter));
    }
    if let Some(adapter) = &context.project_config.defaults.adapter {
        return ResolvedValue::project(json!(adapter));
    }
    if let Some(adapter) = &context.user_config.defaults.adapter {
        return ResolvedValue::user(json!(adapter));
    }
    ResolvedValue::unset()
}

pub fn resolve_limit_chars(
    explicit: Option<docnav_protocol::PositiveInteger>,
    context: &ConfigContext,
) -> AppResult<ResolvedValue> {
    if let Some(limit_chars) = explicit {
        return Ok(ResolvedValue::explicit(json!(limit_chars.get())));
    }
    if let Some(limit_chars) = context.project_config.defaults.limit_chars {
        validate_positive_key("defaults.limit_chars", limit_chars)?;
        return Ok(ResolvedValue::project(json!(limit_chars)));
    }
    if let Some(limit_chars) = context.user_config.defaults.limit_chars {
        validate_positive_key("defaults.limit_chars", limit_chars)?;
        return Ok(ResolvedValue::user(json!(limit_chars)));
    }
    Ok(ResolvedValue::built_in(json!(DEFAULT_LIMIT_CHARS)))
}

pub fn resolve_output(
    explicit: Option<OutputMode>,
    context: &ConfigContext,
) -> AppResult<ResolvedValue> {
    if let Some(output) = explicit {
        return Ok(ResolvedValue::explicit(json!(output.as_str())));
    }
    if let Some(output) = &context.project_config.defaults.output {
        let output = parse_output(
            "defaults.output",
            output,
            &context.project.project_config_path,
        )?;
        return Ok(ResolvedValue::project(json!(output.as_str())));
    }
    if let Some(output) = &context.user_config.defaults.output {
        let output = parse_output("defaults.output", output, &context.project.user_config_path)?;
        return Ok(ResolvedValue::user(json!(output.as_str())));
    }
    Ok(ResolvedValue::built_in(json!(DEFAULT_OUTPUT)))
}

pub(super) fn supported_values_for_scope(
    config: &CoreConfig,
    source: ConfigSource,
) -> AppResult<Vec<Value>> {
    SUPPORTED_KEYS
        .iter()
        .map(|key| {
            scoped_key_value(key, config, source.clone()).map(|resolved| {
                json!({
                    "key": key,
                    "value": resolved.value,
                    "source": resolved.source,
                })
            })
        })
        .collect()
}

pub(super) fn effective_values(context: &ConfigContext) -> AppResult<Vec<Value>> {
    SUPPORTED_KEYS
        .iter()
        .map(|key| {
            effective_key_value(key, context).map(|resolved| {
                json!({
                    "key": key,
                    "value": resolved.value,
                    "source": resolved.source,
                })
            })
        })
        .collect()
}

pub(super) fn effective_key_value(key: &str, context: &ConfigContext) -> AppResult<ResolvedValue> {
    match key {
        "defaults.adapter" => Ok(resolve_adapter(None, context)),
        "defaults.limit_chars" => resolve_limit_chars(None, context),
        "defaults.output" => resolve_output(None, context),
        _ => Err(unknown_key(key)),
    }
}

pub(super) fn scoped_key_value(
    key: &str,
    config: &CoreConfig,
    source: ConfigSource,
) -> AppResult<ResolvedValue> {
    match key {
        "defaults.adapter" => Ok(config
            .defaults
            .adapter
            .as_ref()
            .map(|value| ResolvedValue::new(json!(value), source.clone()))
            .unwrap_or_else(ResolvedValue::unset)),
        "defaults.limit_chars" => Ok(config
            .defaults
            .limit_chars
            .map(|value| ResolvedValue::new(json!(value), source.clone()))
            .unwrap_or_else(ResolvedValue::unset)),
        "defaults.output" => Ok(config
            .defaults
            .output
            .as_ref()
            .map(|value| ResolvedValue::new(json!(value.as_str()), source))
            .unwrap_or_else(ResolvedValue::unset)),
        _ => Err(unknown_key(key)),
    }
}

pub(super) fn set_key(
    config: &mut CoreConfig,
    key: &str,
    value: &str,
    config_path: Option<&Path>,
) -> AppResult<()> {
    match key {
        "defaults.adapter" => {
            if value.is_empty() {
                return Err(AppError::invalid_request(
                    key,
                    "adapter id must not be empty",
                ));
            }
            config.defaults.adapter = Some(value.to_owned());
        }
        "defaults.limit_chars" => {
            let limit_chars = value.parse::<u32>().map_err(|_| {
                AppError::invalid_request(key, "defaults.limit_chars must be a positive integer")
            })?;
            validate_positive_key(key, limit_chars)?;
            config.defaults.limit_chars = Some(limit_chars);
        }
        "defaults.output" => {
            let output = match config_path {
                Some(path) => parse_output(key, value, path)?,
                None => value
                    .parse::<OutputMode>()
                    .map_err(|reason: String| AppError::invalid_request(key, reason))?,
            };
            config.defaults.output = Some(output.as_str().to_owned());
        }
        _ => return Err(unknown_key(key)),
    }
    Ok(())
}

pub(super) fn unset_key(config: &mut CoreConfig, key: &str) -> AppResult<()> {
    match key {
        "defaults.adapter" => config.defaults.adapter = None,
        "defaults.limit_chars" => config.defaults.limit_chars = None,
        "defaults.output" => config.defaults.output = None,
        _ => return Err(unknown_key(key)),
    }
    Ok(())
}

pub(super) fn config_value_to_json(key: &str, config: &CoreConfig) -> AppResult<Value> {
    Ok(match key {
        "defaults.adapter" => config
            .defaults
            .adapter
            .as_ref()
            .map(|value| json!(value))
            .unwrap_or(Value::Null),
        "defaults.limit_chars" => config
            .defaults
            .limit_chars
            .map(|value| json!(value))
            .unwrap_or(Value::Null),
        "defaults.output" => config
            .defaults
            .output
            .as_ref()
            .map(|value| json!(value.as_str()))
            .unwrap_or(Value::Null),
        _ => return Err(unknown_key(key)),
    })
}

pub(super) fn validate_output_key(
    field: &str,
    value: &Option<String>,
    path: &Path,
) -> AppResult<()> {
    if let Some(value) = value {
        parse_output(field, value, path)?;
    }
    Ok(())
}

fn parse_output(field: &str, value: &str, path: &Path) -> AppResult<OutputMode> {
    value.parse::<OutputMode>().map_err(|reason: String| {
        let path = path_to_slash(path);
        let accepted = OutputMode::ACCEPTED_VALUES.join(", ");
        let details = FieldReasonDetails {
            field: field.to_owned(),
            reason: format!(
                "{path} contains invalid {field}: received {value:?}; accepted values: {accepted}; {reason}"
            ),
            path: Some(path),
            received: Some(value.to_owned()),
            accepted: Some(
                OutputMode::ACCEPTED_VALUES
                    .iter()
                    .map(|value| (*value).to_owned())
                    .collect(),
            ),
        };
        AppError::new(protocol_error_record_draft_with_summary::<
            typed_codes::protocol::InvalidRequest,
        >(
            "Invalid protocol request.",
            details,
            DiagnosticSource::with_stage("docnav", "config"),
        ))
    })
}

pub(super) fn ensure_supported_key(key: &str) -> AppResult<()> {
    if SUPPORTED_KEYS.contains(&key) {
        Ok(())
    } else {
        Err(unknown_key(key))
    }
}

fn unknown_key(key: &str) -> AppError {
    AppError::invalid_request("key", format!("unsupported docnav config key {key:?}"))
}

pub(super) fn validate_positive_key(key: &str, value: u32) -> AppResult<()> {
    if value == 0 {
        Err(AppError::invalid_request(
            key,
            format!("{key} must be a positive integer"),
        ))
    } else {
        Ok(())
    }
}
