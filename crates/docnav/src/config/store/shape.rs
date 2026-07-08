use std::path::Path;

use serde_json::Value;

use crate::error::AppResult;
use crate::project_context::ConfigPathOrigin;
use crate::registry::AdapterRegistry;

use super::diagnostics::{
    config_source_error, invalid_config_object_error, unknown_config_field_error,
};
use super::outline::validate_outline_shape;
use super::ConfigFileSource;

pub(super) fn validate_config_shape(
    value: &Value,
    path: &Path,
    registry: &AdapterRegistry,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
) -> AppResult<()> {
    let Some(root) = value.as_object() else {
        return Err(config_source_error(path, source, origin, "non_object"));
    };

    for (key, child) in root {
        match key.as_str() {
            "defaults" => validate_defaults_shape(child, path, source, origin)?,
            "outline" => validate_outline_shape(child, path, source, origin)?,
            "invocation_log" => validate_invocation_log_shape(child, path, source, origin)?,
            "options" => validate_options_shape(child, path, registry, source, origin)?,
            _ => return Err(unknown_config_field_error(path, source, origin, key, None)),
        }
    }

    Ok(())
}

fn validate_defaults_shape(
    value: &Value,
    path: &Path,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
) -> AppResult<()> {
    let Some(defaults) = value.as_object() else {
        return Err(invalid_config_object_error(
            path, source, origin, "defaults",
        ));
    };

    for (key, child) in defaults {
        match key.as_str() {
            "adapter" | "output" => {}
            "pagination" => validate_pagination_shape(child, path, source, origin)?,
            "limit" => {
                return Err(unknown_config_field_error(
                    path,
                    source,
                    origin,
                    "defaults.limit",
                    Some("defaults.pagination.limit"),
                ));
            }
            _ => {
                return Err(unknown_config_field_error(
                    path,
                    source,
                    origin,
                    &format!("defaults.{key}"),
                    None,
                ));
            }
        }
    }

    Ok(())
}

fn validate_pagination_shape(
    value: &Value,
    path: &Path,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
) -> AppResult<()> {
    let Some(pagination) = value.as_object() else {
        return Err(invalid_config_object_error(
            path,
            source,
            origin,
            "defaults.pagination",
        ));
    };

    for key in pagination.keys() {
        match key.as_str() {
            "enabled" | "limit" => {}
            _ => {
                return Err(unknown_config_field_error(
                    path,
                    source,
                    origin,
                    &format!("defaults.pagination.{key}"),
                    None,
                ));
            }
        }
    }

    Ok(())
}

fn validate_invocation_log_shape(
    value: &Value,
    path: &Path,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
) -> AppResult<()> {
    let Some(invocation_log) = value.as_object() else {
        return Err(invalid_config_object_error(
            path,
            source,
            origin,
            "invocation_log",
        ));
    };

    for (key, child) in invocation_log {
        match key.as_str() {
            "enabled" | "path" => {}
            "content_capture" => {
                validate_invocation_log_content_capture_shape(child, path, source, origin)?
            }
            _ => {
                return Err(unknown_config_field_error(
                    path,
                    source,
                    origin,
                    &format!("invocation_log.{key}"),
                    None,
                ));
            }
        }
    }

    Ok(())
}

fn validate_invocation_log_content_capture_shape(
    value: &Value,
    path: &Path,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
) -> AppResult<()> {
    let Some(content_capture) = value.as_object() else {
        return Err(invalid_config_object_error(
            path,
            source,
            origin,
            "invocation_log.content_capture",
        ));
    };

    for key in content_capture.keys() {
        match key.as_str() {
            "enabled" | "root" => {}
            _ => {
                return Err(unknown_config_field_error(
                    path,
                    source,
                    origin,
                    &format!("invocation_log.content_capture.{key}"),
                    None,
                ));
            }
        }
    }

    Ok(())
}

fn validate_options_shape(
    value: &Value,
    path: &Path,
    registry: &AdapterRegistry,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
) -> AppResult<()> {
    let Some(options) = value.as_object() else {
        return Err(invalid_config_object_error(path, source, origin, "options"));
    };

    for key in options.keys() {
        let field = format!("options.{key}");
        if !registry.has_native_option_config_key(&field) {
            return Err(unknown_config_field_error(
                path, source, origin, &field, None,
            ));
        }
    }

    Ok(())
}
