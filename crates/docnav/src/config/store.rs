use std::fs;
use std::io;
use std::path::Path;

use docnav_navigation::{validate_navigation_config_source_value, NavigationConfigSourceLevel};
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::project_context::{ConfigPathOrigin, ProjectContext, SelectedConfigPath};
use crate::project_paths::path_to_slash;
use crate::registry::AdapterRegistry;

use super::model::{ConfigContext, CoreConfig};

mod diagnostics;
use diagnostics::config_source_error;

pub(crate) fn load_context_for_project(project: ProjectContext) -> AppResult<ConfigContext> {
    let registry = AdapterRegistry::load(&project)?;
    let project_config = read_context_config(&project, &registry, ConfigFileSource::Project)?;
    let user_config = read_context_config(&project, &registry, ConfigFileSource::User)?;
    Ok(ConfigContext {
        project,
        project_config,
        user_config,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ConfigFileSource {
    Project,
    User,
}

impl ConfigFileSource {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::User => "user",
        }
    }

    pub(super) fn selected_path(self, project: &ProjectContext) -> &SelectedConfigPath {
        match self {
            Self::Project => &project.config_paths.project,
            Self::User => &project.config_paths.user,
        }
    }

    fn config_source_level(self) -> NavigationConfigSourceLevel {
        match self {
            Self::Project => NavigationConfigSourceLevel::Project,
            Self::User => NavigationConfigSourceLevel::User,
        }
    }
}

fn read_context_config(
    project: &ProjectContext,
    registry: &AdapterRegistry,
    source: ConfigFileSource,
) -> AppResult<CoreConfig> {
    read_selected_config(source.selected_path(project), registry, source)
}

fn read_config(
    selection: &SelectedConfigPath,
    registry: &AdapterRegistry,
    source: ConfigFileSource,
) -> AppResult<CoreConfig> {
    let path = &selection.path;
    let origin = selection.origin;
    let Some(value) = read_config_source_value(selection, source)? else {
        return Ok(CoreConfig::default());
    };
    validate_navigation_config_source_value(
        source.config_source_level(),
        navigation_config_path_origin(origin),
        path_string(path),
        value.clone(),
        registry,
    )
    .map_err(|error| AppError::new(error.into_diagnostic()))?;
    let config: CoreConfig = serde_json::from_value(value)
        .map_err(|_| config_source_error(path, source, origin, "invalid_config_value"))?;
    Ok(config)
}

pub(super) fn read_selected_config(
    selection: &SelectedConfigPath,
    registry: &AdapterRegistry,
    source: ConfigFileSource,
) -> AppResult<CoreConfig> {
    read_config(selection, registry, source)
}

fn read_config_source_value(
    selection: &SelectedConfigPath,
    source: ConfigFileSource,
) -> AppResult<Option<Value>> {
    if !config_source_file_exists(selection, source)? {
        return Ok(None);
    }
    let Some(content) = read_config_source_content(selection, source)? else {
        return Ok(None);
    };
    decode_config_source_object(selection, source, &content).map(Some)
}

fn config_source_file_exists(
    selection: &SelectedConfigPath,
    source: ConfigFileSource,
) -> AppResult<bool> {
    match fs::metadata(&selection.path) {
        Ok(metadata) if metadata.is_file() => Ok(true),
        Ok(_) => Err(config_source_error(
            &selection.path,
            source,
            selection.origin,
            "not_file",
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            handle_missing_config_source(selection, source)?;
            Ok(false)
        }
        Err(_) => Err(config_source_error(
            &selection.path,
            source,
            selection.origin,
            "unreadable",
        )),
    }
}

fn read_config_source_content(
    selection: &SelectedConfigPath,
    source: ConfigFileSource,
) -> AppResult<Option<String>> {
    match fs::read_to_string(&selection.path) {
        Ok(content) => Ok(Some(content)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            handle_missing_config_source(selection, source)?;
            Ok(None)
        }
        Err(_) => Err(config_source_error(
            &selection.path,
            source,
            selection.origin,
            "unreadable",
        )),
    }
}

fn handle_missing_config_source(
    selection: &SelectedConfigPath,
    source: ConfigFileSource,
) -> AppResult<()> {
    match selection.origin {
        ConfigPathOrigin::Default => Ok(()),
        ConfigPathOrigin::ExplicitCli => Err(config_source_error(
            &selection.path,
            source,
            selection.origin,
            "missing_explicit_cli",
        )),
    }
}

fn decode_config_source_object(
    selection: &SelectedConfigPath,
    source: ConfigFileSource,
    content: &str,
) -> AppResult<Value> {
    let value = serde_json::from_str::<Value>(content).map_err(|_| {
        config_source_error(&selection.path, source, selection.origin, "invalid_json")
    })?;
    if !value.is_object() {
        return Err(config_source_error(
            &selection.path,
            source,
            selection.origin,
            "non_object",
        ));
    }
    Ok(value)
}

fn navigation_config_path_origin(
    origin: ConfigPathOrigin,
) -> docnav_navigation::NavigationConfigSourceOrigin {
    match origin {
        ConfigPathOrigin::Default => docnav_navigation::NavigationConfigSourceOrigin::Default,
        ConfigPathOrigin::ExplicitCli => {
            docnav_navigation::NavigationConfigSourceOrigin::ExplicitCli
        }
    }
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
        .map_err(|_| AppError::internal("serialize-config-failed"))?;
    fs::write(path, format!("{content}\n")).map_err(|error| {
        AppError::invalid_request(
            "config",
            format!("failed to write {}: {error}", path_string(path)),
        )
    })
}

pub(super) fn path_string(path: &Path) -> String {
    path_to_slash(path)
}

#[cfg(test)]
mod tests;
