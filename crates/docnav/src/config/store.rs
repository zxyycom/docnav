use std::fs;
use std::path::Path;

use docnav_parameter_resolution::{
    load_parameter_config_source, ConfigPathOrigin as ParameterConfigPathOrigin, ConfigSourceLevel,
    ParameterConfigSourceDescriptor,
};
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::project_context::{ConfigPathOrigin, ProjectContext, SelectedConfigPath};
use crate::project_paths::path_to_slash;
use crate::registry::AdapterRegistry;

use super::keys::{
    validate_invocation_log_content_capture_root_key, validate_invocation_log_path_key,
    validate_output_key, validate_positive_key,
};
use super::model::{ConfigContext, CoreConfig};

mod diagnostics;
mod outline;

use diagnostics::{config_source_error, invalid_config_object_error, unknown_config_field_error};
use outline::validate_outline_shape;

pub fn load_context(
    project_config: Option<&str>,
    user_config: Option<&str>,
) -> AppResult<ConfigContext> {
    let project = ProjectContext::discover_with_cli_config_paths(project_config, user_config)?;
    load_context_for_project(project)
}

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
    validate_config_shape(&value, path, registry, source, origin)?;
    let config: CoreConfig = serde_json::from_value(value)
        .map_err(|_| config_source_error(path, source, origin, "invalid_config_value"))?;
    validate_config(&config, path, registry, source, origin)?;
    Ok(config)
}

pub(super) fn read_selected_config(
    selection: &SelectedConfigPath,
    registry: &AdapterRegistry,
    source: ConfigFileSource,
) -> AppResult<CoreConfig> {
    read_config(selection, registry, source)
}

pub(super) fn read_config_for_update(
    selection: &SelectedConfigPath,
    registry: &AdapterRegistry,
    source: ConfigFileSource,
) -> AppResult<CoreConfig> {
    if selection.path.exists() {
        read_selected_config(selection, registry, source)
    } else {
        Ok(CoreConfig::default())
    }
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

fn validate_config(
    config: &CoreConfig,
    path: &Path,
    registry: &AdapterRegistry,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
) -> AppResult<()> {
    if let Some(adapter) = &config.defaults.adapter {
        if adapter.is_empty() {
            return Err(AppError::invalid_request(
                "defaults.adapter",
                format!("{} contains an empty adapter id", path_string(path)),
            ));
        }
    }
    if let Some(limit) = config.defaults.pagination.limit {
        validate_positive_key("defaults.pagination.limit", limit)?;
    }
    validate_output_key("defaults.output", &config.defaults.output, path)?;
    if let Some(log_path) = &config.invocation_log.path {
        validate_invocation_log_path_key("invocation_log.path", log_path)?;
    }
    if let Some(root) = &config.invocation_log.content_capture.root {
        validate_invocation_log_content_capture_root_key(
            "invocation_log.content_capture.root",
            root,
        )?;
    }
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

pub(super) fn path_string(path: &Path) -> String {
    path_to_slash(path)
}

fn validate_config_shape(
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

#[cfg(test)]
mod tests;
