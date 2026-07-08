use std::path::Path;

use crate::error::{AppError, AppResult};
use crate::project_context::ConfigPathOrigin;
use crate::registry::AdapterRegistry;

use super::diagnostics::unknown_config_field_error;
use super::{path_string, ConfigFileSource};
use crate::config::keys::{
    validate_invocation_log_content_capture_root_key, validate_invocation_log_path_key,
    validate_output_key, validate_positive_key,
};
use crate::config::model::CoreConfig;

pub(super) fn validate_config_values(
    config: &CoreConfig,
    path: &Path,
    registry: &AdapterRegistry,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
) -> AppResult<()> {
    validate_defaults(config, path)?;
    validate_invocation_log(config)?;
    validate_registered_options(config, path, registry, source, origin)
}

fn validate_defaults(config: &CoreConfig, path: &Path) -> AppResult<()> {
    if let Some(adapter) = &config.defaults.adapter {
        validate_adapter_id(adapter, path)?;
    }
    if let Some(limit) = config.defaults.pagination.limit {
        validate_positive_key("defaults.pagination.limit", limit)?;
    }
    validate_output_key("defaults.output", &config.defaults.output, path)
}

fn validate_adapter_id(adapter: &str, path: &Path) -> AppResult<()> {
    if adapter.is_empty() {
        return Err(AppError::invalid_request(
            "defaults.adapter",
            format!("{} contains an empty adapter id", path_string(path)),
        ));
    }
    Ok(())
}

fn validate_invocation_log(config: &CoreConfig) -> AppResult<()> {
    if let Some(log_path) = &config.invocation_log.path {
        validate_invocation_log_path_key("invocation_log.path", log_path)?;
    }
    if let Some(root) = &config.invocation_log.content_capture.root {
        validate_invocation_log_content_capture_root_key(
            "invocation_log.content_capture.root",
            root,
        )?;
    }
    Ok(())
}

fn validate_registered_options(
    config: &CoreConfig,
    path: &Path,
    registry: &AdapterRegistry,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
) -> AppResult<()> {
    for key in config.options.keys() {
        let field = format!("options.{key}");
        if !registry.has_native_option_config_key(&field) {
            return Err(unknown_config_field_error(
                path, source, origin, &field, None,
            ));
        }
    }
    Ok(())
}
