use std::path::Path;

use crate::cli::OutputMode;
use crate::error::{AppError, AppResult};
use crate::registry::AdapterRegistry;

use super::super::model::CoreConfig;
use super::registered_native_option_key;
use super::validation::{
    parse_native_option_value, parse_output, parse_pagination_enabled, unknown_key,
    validate_positive_key,
};

pub(in crate::config) fn set_key(
    config: &mut CoreConfig,
    key: &str,
    value: &str,
    config_path: Option<&Path>,
    registry: &AdapterRegistry,
) -> AppResult<()> {
    match key {
        "defaults.adapter" => set_adapter(config, key, value),
        "defaults.pagination.enabled" => set_pagination_enabled(config, key, value),
        "defaults.pagination.limit" => set_pagination_limit(config, key, value),
        "defaults.output" => set_output(config, key, value, config_path),
        _ if key.starts_with("options.") => set_native_option(config, key, value, registry),
        _ => Err(unknown_key(key)),
    }
}

pub(in crate::config) fn unset_key(
    config: &mut CoreConfig,
    key: &str,
    registry: &AdapterRegistry,
) -> AppResult<()> {
    match key {
        "defaults.adapter" => config.defaults.adapter = None,
        "defaults.pagination.enabled" => config.defaults.pagination.enabled = None,
        "defaults.pagination.limit" => config.defaults.pagination.limit = None,
        "defaults.output" => config.defaults.output = None,
        _ => {
            let option_key = registered_native_option_key(key, registry)?;
            config.options.remove(option_key);
        }
    }
    Ok(())
}

fn set_adapter(config: &mut CoreConfig, key: &str, value: &str) -> AppResult<()> {
    if value.is_empty() {
        return Err(AppError::invalid_request(
            key,
            "adapter id must not be empty",
        ));
    }
    config.defaults.adapter = Some(value.to_owned());
    Ok(())
}

fn set_pagination_enabled(config: &mut CoreConfig, key: &str, value: &str) -> AppResult<()> {
    config.defaults.pagination.enabled = Some(parse_pagination_enabled(key, value)?);
    Ok(())
}

fn set_pagination_limit(config: &mut CoreConfig, key: &str, value: &str) -> AppResult<()> {
    let limit = value.parse::<u32>().map_err(|_| {
        AppError::invalid_request(key, "defaults.pagination.limit must be a positive integer")
    })?;
    validate_positive_key(key, limit)?;
    config.defaults.pagination.limit = Some(limit);
    Ok(())
}

fn set_output(
    config: &mut CoreConfig,
    key: &str,
    value: &str,
    config_path: Option<&Path>,
) -> AppResult<()> {
    let output = match config_path {
        Some(path) => parse_output(key, value, path)?,
        None => value
            .parse::<OutputMode>()
            .map_err(|reason: String| AppError::invalid_request(key, reason))?,
    };
    config.defaults.output = Some(output.as_str().to_owned());
    Ok(())
}

fn set_native_option(
    config: &mut CoreConfig,
    key: &str,
    value: &str,
    registry: &AdapterRegistry,
) -> AppResult<()> {
    let value = parse_native_option_value(registry, key, value)?;
    let option_key = registered_native_option_key(key, registry)?;
    config.options.insert(option_key, value);
    Ok(())
}
