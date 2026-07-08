use serde_json::{json, Value};

use crate::error::AppResult;
use crate::registry::AdapterRegistry;

use super::super::model::{
    ConfigContext, ConfigSource, CoreConfig, ResolvedValue, SUPPORTED_CORE_KEYS,
};
use super::registered_native_option_key;
use super::resolve::effective_key_value;
use super::validation::unknown_key;

pub(in crate::config) fn supported_values_for_scope(
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

pub(in crate::config) fn effective_values(
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

pub(in crate::config) fn scoped_key_value(
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
        "invocation_log.enabled" => Ok(config
            .invocation_log
            .enabled
            .map(|value| ResolvedValue::new(json!(value), source.clone()))
            .unwrap_or_else(ResolvedValue::unset)),
        "invocation_log.path" => Ok(config
            .invocation_log
            .path
            .as_ref()
            .map(|value| ResolvedValue::new(json!(value), source.clone()))
            .unwrap_or_else(ResolvedValue::unset)),
        "invocation_log.content_capture.enabled" => Ok(config
            .invocation_log
            .content_capture
            .enabled
            .map(|value| ResolvedValue::new(json!(value), source.clone()))
            .unwrap_or_else(ResolvedValue::unset)),
        "invocation_log.content_capture.root" => Ok(config
            .invocation_log
            .content_capture
            .root
            .as_ref()
            .map(|value| ResolvedValue::new(json!(value), source))
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

pub(in crate::config) fn config_value_to_json(
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
        "invocation_log.enabled" => config
            .invocation_log
            .enabled
            .map(|value| json!(value))
            .unwrap_or(Value::Null),
        "invocation_log.path" => config
            .invocation_log
            .path
            .as_ref()
            .map(|value| json!(value))
            .unwrap_or(Value::Null),
        "invocation_log.content_capture.enabled" => config
            .invocation_log
            .content_capture
            .enabled
            .map(|value| json!(value))
            .unwrap_or(Value::Null),
        "invocation_log.content_capture.root" => config
            .invocation_log
            .content_capture
            .root
            .as_ref()
            .map(|value| json!(value))
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

pub(in crate::config) fn ensure_supported_key(
    key: &str,
    registry: &AdapterRegistry,
) -> AppResult<()> {
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
