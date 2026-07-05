use std::path::Path;

use serde_json::Value;

use crate::error::AppResult;
use crate::project_context::ConfigPathOrigin;

use super::diagnostics::{
    invalid_config_array_error, invalid_config_object_error, unknown_config_field_error,
};
use super::ConfigFileSource;

pub(super) fn validate_outline_shape(
    value: &Value,
    path: &Path,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
) -> AppResult<()> {
    let Some(outline) = value.as_object() else {
        return Err(invalid_config_object_error(path, source, origin, "outline"));
    };

    for (key, child) in outline {
        match key.as_str() {
            "mode_rules" => {
                validate_array_shape(child, path, source, origin, "outline.mode_rules")?
            }
            "auto_full_read" => validate_auto_full_read_shape(child, path, source, origin)?,
            _ => {
                return Err(unknown_config_field_error(
                    path,
                    source,
                    origin,
                    &format!("outline.{key}"),
                    None,
                ));
            }
        }
    }

    Ok(())
}

fn validate_auto_full_read_shape(
    value: &Value,
    path: &Path,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
) -> AppResult<()> {
    let Some(auto_full_read) = value.as_object() else {
        return Err(invalid_config_object_error(
            path,
            source,
            origin,
            "outline.auto_full_read",
        ));
    };

    for (key, child) in auto_full_read {
        match key.as_str() {
            "thresholds" => validate_array_shape(
                child,
                path,
                source,
                origin,
                "outline.auto_full_read.thresholds",
            )?,
            _ => {
                return Err(unknown_config_field_error(
                    path,
                    source,
                    origin,
                    &format!("outline.auto_full_read.{key}"),
                    None,
                ));
            }
        }
    }

    Ok(())
}

fn validate_array_shape(
    value: &Value,
    path: &Path,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
    field: &str,
) -> AppResult<()> {
    if value.is_array() {
        Ok(())
    } else {
        Err(invalid_config_array_error(path, source, origin, field))
    }
}
