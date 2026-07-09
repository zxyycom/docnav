use std::fs;
use std::path::Path;

use docnav_navigation::validate_navigation_config_source_value;
use docnav_parameter_resolution::{
    load_parameter_config_source, ConfigPathOrigin as ParameterConfigPathOrigin, ConfigSourceLevel,
    ParameterConfigSourceDescriptor,
};
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

    fn config_source_level(self) -> ConfigSourceLevel {
        match self {
            Self::Project => ConfigSourceLevel::Project,
            Self::User => ConfigSourceLevel::User,
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
    let descriptor = ParameterConfigSourceDescriptor::new(
        source.config_source_level(),
        parameter_config_path_origin(selection.origin),
        selection.path.clone(),
    );
    let loaded = load_parameter_config_source(&descriptor);
    if let Some(issue) = loaded
        .diagnostics()
        .first()
        .and_then(|diagnostic| diagnostic.as_config_source())
    {
        return Err(config_source_error(
            &selection.path,
            source,
            selection.origin,
            &issue.reason_code,
        ));
    }
    Ok(loaded.value().cloned())
}

fn parameter_config_path_origin(origin: ConfigPathOrigin) -> ParameterConfigPathOrigin {
    match origin {
        ConfigPathOrigin::Default => ParameterConfigPathOrigin::Default,
        ConfigPathOrigin::ExplicitCli => ParameterConfigPathOrigin::ExplicitCli,
    }
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
