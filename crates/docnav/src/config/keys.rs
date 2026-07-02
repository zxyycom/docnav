use serde_json::{json, Value};

use crate::cli::OutputMode;
use crate::error::AppResult;
use crate::registry::AdapterRegistry;

use super::model::{
    ConfigContext, ConfigSource, CoreConfig, ResolvedValue, DEFAULT_LIMIT, DEFAULT_OUTPUT,
    DEFAULT_PAGINATION_ENABLED, SUPPORTED_CORE_KEYS,
};

mod update;
mod validation;

pub(super) use self::update::{set_key, unset_key};
use self::validation::{parse_output, unknown_key};
pub(super) use self::validation::{
    validate_native_option_key_for_registry, validate_output_key, validate_positive_key,
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

pub fn resolve_pagination_enabled(context: &ConfigContext) -> AppResult<ResolvedValue> {
    if let Some(enabled) = context.project_config.defaults.pagination.enabled {
        return Ok(ResolvedValue::project(json!(enabled)));
    }
    if let Some(enabled) = context.user_config.defaults.pagination.enabled {
        return Ok(ResolvedValue::user(json!(enabled)));
    }
    Ok(ResolvedValue::built_in(json!(DEFAULT_PAGINATION_ENABLED)))
}

pub fn resolve_limit(
    explicit: Option<docnav_protocol::PositiveInteger>,
    context: &ConfigContext,
) -> AppResult<ResolvedValue> {
    if let Some(limit) = explicit {
        return Ok(ResolvedValue::explicit(json!(limit.get())));
    }
    if let Some(limit) = context.project_config.defaults.pagination.limit {
        validate_positive_key("defaults.pagination.limit", limit)?;
        return Ok(ResolvedValue::project(json!(limit)));
    }
    if let Some(limit) = context.user_config.defaults.pagination.limit {
        validate_positive_key("defaults.pagination.limit", limit)?;
        return Ok(ResolvedValue::user(json!(limit)));
    }
    Ok(ResolvedValue::built_in(json!(DEFAULT_LIMIT)))
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
    registry: &AdapterRegistry,
) -> AppResult<Vec<Value>> {
    supported_keys(registry)
        .into_iter()
        .map(|key| {
            scoped_key_value(&key, config, source.clone(), registry).map(|resolved| {
                json!({
                    "key": key,
                    "value": resolved.value,
                    "source": resolved.source,
                })
            })
        })
        .collect()
}

pub(super) fn effective_values(
    context: &ConfigContext,
    registry: &AdapterRegistry,
) -> AppResult<Vec<Value>> {
    supported_keys(registry)
        .into_iter()
        .map(|key| {
            effective_key_value(&key, context, registry).map(|resolved| {
                json!({
                    "key": key,
                    "value": resolved.value,
                    "source": resolved.source,
                })
            })
        })
        .collect()
}

pub(super) fn effective_key_value(
    key: &str,
    context: &ConfigContext,
    registry: &AdapterRegistry,
) -> AppResult<ResolvedValue> {
    match key {
        "defaults.adapter" => Ok(resolve_adapter(None, context)),
        "defaults.pagination.enabled" => resolve_pagination_enabled(context),
        "defaults.pagination.limit" => resolve_limit(None, context),
        "defaults.output" => resolve_output(None, context),
        _ => {
            let option_key = registered_native_option_key(key, registry)?;
            Ok(resolve_native_option(option_key, context))
        }
    }
}

pub(super) fn scoped_key_value(
    key: &str,
    config: &CoreConfig,
    source: ConfigSource,
    registry: &AdapterRegistry,
) -> AppResult<ResolvedValue> {
    match key {
        "defaults.adapter" => Ok(config
            .defaults
            .adapter
            .as_ref()
            .map(|value| ResolvedValue::new(json!(value), source.clone()))
            .unwrap_or_else(ResolvedValue::unset)),
        "defaults.pagination.enabled" => Ok(config
            .defaults
            .pagination
            .enabled
            .map(|value| ResolvedValue::new(json!(value), source.clone()))
            .unwrap_or_else(ResolvedValue::unset)),
        "defaults.pagination.limit" => Ok(config
            .defaults
            .pagination
            .limit
            .map(|value| ResolvedValue::new(json!(value), source.clone()))
            .unwrap_or_else(ResolvedValue::unset)),
        "defaults.output" => Ok(config
            .defaults
            .output
            .as_ref()
            .map(|value| ResolvedValue::new(json!(value.as_str()), source))
            .unwrap_or_else(ResolvedValue::unset)),
        _ => {
            let option_key = registered_native_option_key(key, registry)?;
            Ok(config
                .options
                .value_for_key(option_key)
                .map(|value| ResolvedValue::new(value.clone(), source))
                .unwrap_or_else(ResolvedValue::unset))
        }
    }
}

pub(super) fn config_value_to_json(
    key: &str,
    config: &CoreConfig,
    registry: &AdapterRegistry,
) -> AppResult<Value> {
    Ok(match key {
        "defaults.adapter" => config
            .defaults
            .adapter
            .as_ref()
            .map(|value| json!(value))
            .unwrap_or(Value::Null),
        "defaults.pagination.enabled" => config
            .defaults
            .pagination
            .enabled
            .map(|value| json!(value))
            .unwrap_or(Value::Null),
        "defaults.pagination.limit" => config
            .defaults
            .pagination
            .limit
            .map(|value| json!(value))
            .unwrap_or(Value::Null),
        "defaults.output" => config
            .defaults
            .output
            .as_ref()
            .map(|value| json!(value.as_str()))
            .unwrap_or(Value::Null),
        _ => {
            let option_key = registered_native_option_key(key, registry)?;
            config
                .options
                .value_for_key(option_key)
                .cloned()
                .unwrap_or(Value::Null)
        }
    })
}

pub(super) fn ensure_supported_key(key: &str, registry: &AdapterRegistry) -> AppResult<()> {
    if SUPPORTED_CORE_KEYS.contains(&key) || registry.has_native_option_config_key(key) {
        Ok(())
    } else {
        Err(unknown_key(key))
    }
}

fn supported_keys(registry: &AdapterRegistry) -> Vec<String> {
    SUPPORTED_CORE_KEYS
        .iter()
        .map(|key| (*key).to_owned())
        .chain(registry.native_option_config_keys())
        .collect()
}

fn resolve_native_option(key: &str, context: &ConfigContext) -> ResolvedValue {
    if let Some(value) = context.project_config.options.value_for_key(key) {
        return ResolvedValue::project(value.clone());
    }
    if let Some(value) = context.user_config.options.value_for_key(key) {
        return ResolvedValue::user(value.clone());
    }
    ResolvedValue::unset()
}

pub(super) fn registered_native_option_key<'a>(
    config_key: &'a str,
    registry: &AdapterRegistry,
) -> AppResult<&'a str> {
    validate_native_option_key_for_registry(registry, config_key)?;
    config_key
        .strip_prefix("options.")
        .ok_or_else(|| unknown_key(config_key))
}
