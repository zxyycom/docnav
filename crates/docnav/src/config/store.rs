use std::fs;
use std::path::{Path, PathBuf};

use crate::context::ProjectContext;
use crate::error::{AppError, AppResult};
use crate::project::path_to_slash;

use super::keys::{validate_output_key, validate_positive_key};
use super::model::{ConfigContext, CoreConfig};

pub fn load_context() -> AppResult<ConfigContext> {
    let project = ProjectContext::discover()?;
    let project_config = read_config(&project.project_config_path)?;
    let user_config = read_config(&project.user_config_path)?;
    Ok(ConfigContext {
        project,
        project_config,
        user_config,
    })
}

pub(super) fn read_config(path: &Path) -> AppResult<CoreConfig> {
    if !path.exists() {
        return Ok(CoreConfig::default());
    }
    let content = fs::read_to_string(path).map_err(|error| {
        AppError::invalid_request(
            "config",
            format!("failed to read {}: {error}", path_string(path)),
        )
    })?;
    let config: CoreConfig = serde_json::from_str(&content).map_err(|error| {
        AppError::invalid_request(
            "config",
            format!("failed to parse {}: {error}", path_string(path)),
        )
    })?;
    validate_config(&config, path)?;
    Ok(config)
}

pub(super) fn write_config(path: &Path, config: &CoreConfig) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::invalid_request(
                "config",
                format!("failed to create {}: {error}", path_string(parent)),
            )
        })?;
    }
    let content = serde_json::to_string_pretty(config)
        .map_err(|error| AppError::internal(format!("serialize-config:{error}")))?;
    fs::write(path, format!("{content}\n")).map_err(|error| {
        AppError::invalid_request(
            "config",
            format!("failed to write {}: {error}", path_string(path)),
        )
    })
}

fn validate_config(config: &CoreConfig, path: &Path) -> AppResult<()> {
    if let Some(adapter) = &config.defaults.adapter {
        if adapter.is_empty() {
            return Err(AppError::invalid_request(
                "defaults.adapter",
                format!("{} contains an empty adapter id", path_string(path)),
            ));
        }
    }
    if let Some(limit_chars) = config.defaults.limit_chars {
        validate_positive_key("defaults.limit_chars", limit_chars)?;
    }
    validate_output_key("defaults.output", &config.defaults.output, path)?;
    Ok(())
}

pub(super) fn target_config_path(context: &ConfigContext, user: bool) -> PathBuf {
    if user {
        context.project.user_config_path.clone()
    } else {
        context.project.project_config_path.clone()
    }
}

pub(super) fn path_string(path: &Path) -> String {
    path_to_slash(path)
}
