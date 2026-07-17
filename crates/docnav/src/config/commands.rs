use std::fs;

use docnav_navigation::inspect_navigation_config_sources;
use serde_json::json;

use crate::cli::{ConfigCommand, ConfigInspect, ConfigPathArgs};
use crate::error::{AppError, AppResult};
use crate::output::CommandOutcome;
use crate::parameter_catalog::document_parameter_catalog;
use crate::project_context::ProjectContext;

use super::model::CoreConfig;
use super::store::{path_string, write_config};

pub fn execute(command: ConfigCommand) -> AppResult<CommandOutcome> {
    match command {
        ConfigCommand::Inspect(command) => config_inspect(command),
    }
}

pub fn init_project(config_paths: ConfigPathArgs) -> AppResult<CommandOutcome> {
    let target =
        ProjectContext::discover_project_config_target(config_paths.project_config.as_deref())?;
    let config_path = target.config_path.path;
    let config_dir = config_path.parent().ok_or_else(|| {
        AppError::invalid_request("project_config", "project config path has no parent")
    })?;
    fs::create_dir_all(config_dir)
        .map_err(|error| AppError::invalid_request("project_config", error.to_string()))?;
    let created = if config_path.exists() {
        if !config_path.is_file() {
            return Err(AppError::invalid_request(
                "project_config",
                format!("{} is not a file", path_string(&config_path)),
            ));
        }
        false
    } else {
        write_config(&config_path, &CoreConfig::default())?;
        true
    };

    Ok(CommandOutcome::json(json!({
        "ok": true,
        "created": created,
        "project_root": path_string(&target.project_root),
        "config_path": path_string(&config_path),
    })))
}

fn config_inspect(command: ConfigInspect) -> AppResult<CommandOutcome> {
    let project = ProjectContext::discover_with_cli_config_paths(
        command.config_paths.project_config.as_deref(),
        command.config_paths.user_config.as_deref(),
    )?;
    let catalog = document_parameter_catalog().map_err(|error| {
        AppError::internal(format!(
            "document-parameter-catalog-build-failed:config-inspect:{error}"
        ))
    })?;
    let inspection =
        inspect_navigation_config_sources(project.navigation_config_source_descriptors(), &catalog)
            .map_err(|error| AppError::new(error.into_diagnostic()))?;

    Ok(CommandOutcome::json(json!({
        "project_root": path_string(&project.project_root),
        "inspection": inspection,
    })))
}

#[cfg(test)]
mod tests;
