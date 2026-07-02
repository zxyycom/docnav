use std::fs;
use std::path::{Path, PathBuf};

use docnav_diagnostics::{
    typed_codes, AdapterConfigSourceDetails, DiagnosticSource, FieldReasonDetails,
};
use docnav_protocol::protocol_error_record_draft_with_summary;
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::project_context::ProjectContext;
use crate::project_paths::path_to_slash;
use crate::registry::AdapterRegistry;

use super::keys::{validate_output_key, validate_positive_key};
use super::model::{ConfigContext, CoreConfig};

pub fn load_context() -> AppResult<ConfigContext> {
    let project = ProjectContext::discover()?;
    let registry = AdapterRegistry::load(&project)?;
    let project_config = read_config(
        &project.project_config_path,
        &registry,
        ConfigFileSource::Project,
    )?;
    let user_config = read_config(&project.user_config_path, &registry, ConfigFileSource::User)?;
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
    const fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::User => "user",
        }
    }
}

pub(super) fn read_config(
    path: &Path,
    registry: &AdapterRegistry,
    source: ConfigFileSource,
) -> AppResult<CoreConfig> {
    if !path.exists() {
        return Ok(CoreConfig::default());
    }
    let content =
        fs::read_to_string(path).map_err(|_| config_source_error(path, source, "unreadable"))?;
    let value: Value = serde_json::from_str(&content)
        .map_err(|_| config_source_error(path, source, "invalid_json"))?;
    validate_config_shape(&value, path, registry, source)?;
    let config: CoreConfig = serde_json::from_value(value)
        .map_err(|_| config_source_error(path, source, "invalid_config_value"))?;
    validate_config(&config, path, registry, source)?;
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

fn validate_config(
    config: &CoreConfig,
    path: &Path,
    registry: &AdapterRegistry,
    source: ConfigFileSource,
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
    for key in config.options.keys() {
        let field = format!("options.{key}");
        if !registry.has_native_option_config_key(&field) {
            return Err(unknown_config_field_error(path, source, &field, None));
        }
    }
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

fn validate_config_shape(
    value: &Value,
    path: &Path,
    registry: &AdapterRegistry,
    source: ConfigFileSource,
) -> AppResult<()> {
    let Some(root) = value.as_object() else {
        return Err(config_source_error(path, source, "non_object"));
    };

    for (key, child) in root {
        match key.as_str() {
            "defaults" => validate_defaults_shape(child, path, source)?,
            "options" => validate_options_shape(child, path, registry, source)?,
            _ => return Err(unknown_config_field_error(path, source, key, None)),
        }
    }

    Ok(())
}

fn validate_defaults_shape(value: &Value, path: &Path, source: ConfigFileSource) -> AppResult<()> {
    let Some(defaults) = value.as_object() else {
        return Ok(());
    };

    for (key, child) in defaults {
        match key.as_str() {
            "adapter" | "output" => {}
            "pagination" => validate_pagination_shape(child, path, source)?,
            "limit" => {
                return Err(unknown_config_field_error(
                    path,
                    source,
                    "defaults.limit",
                    Some("defaults.pagination.limit"),
                ));
            }
            _ => {
                return Err(unknown_config_field_error(
                    path,
                    source,
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
) -> AppResult<()> {
    let Some(pagination) = value.as_object() else {
        return Ok(());
    };

    for key in pagination.keys() {
        match key.as_str() {
            "enabled" | "limit" => {}
            _ => {
                return Err(unknown_config_field_error(
                    path,
                    source,
                    &format!("defaults.pagination.{key}"),
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
) -> AppResult<()> {
    let Some(options) = value.as_object() else {
        return Ok(());
    };

    for key in options.keys() {
        let field = format!("options.{key}");
        if !registry.has_native_option_config_key(&field) {
            return Err(unknown_config_field_error(path, source, &field, None));
        }
    }

    Ok(())
}

fn config_source_error(path: &Path, source: ConfigFileSource, reason_code: &str) -> AppError {
    config_error(
        path,
        source,
        ConfigErrorSpec {
            field: "config",
            reason_code,
            accepted: None,
            summary: "Config file is invalid.",
            guidance: Some("Fix the config file so it is a readable JSON object.".to_owned()),
        },
    )
}

fn unknown_config_field_error(
    path: &Path,
    source: ConfigFileSource,
    field: &str,
    accepted: Option<&str>,
) -> AppError {
    config_error(
        path,
        source,
        ConfigErrorSpec {
            field,
            reason_code: "unknown_config_field",
            accepted,
            summary: "Config file contains an unknown field.",
            guidance: Some(match accepted {
                Some(accepted) => format!("Rename {field} to {accepted}."),
                None => format!("Remove unsupported config field {field}."),
            }),
        },
    )
}

struct ConfigErrorSpec<'a> {
    field: &'a str,
    reason_code: &'a str,
    accepted: Option<&'a str>,
    summary: &'a str,
    guidance: Option<String>,
}

fn config_error(path: &Path, source: ConfigFileSource, spec: ConfigErrorSpec<'_>) -> AppError {
    let path = path_string(path);
    let mut details = FieldReasonDetails::new(spec.field, spec.reason_code);
    details.path = Some(path.clone());
    details.received = Some(if spec.field == "config" {
        path.clone()
    } else {
        spec.field.to_owned()
    });
    details.accepted = spec.accepted.map(|value| vec![value.to_owned()]);
    let mut issue =
        AdapterConfigSourceDetails::new(source.as_str(), "default", &path, spec.reason_code);
    if spec.field != "config" {
        issue = issue.with_field(spec.field);
    }
    details.config_issues = Some(vec![issue]);

    let mut draft = protocol_error_record_draft_with_summary::<typed_codes::protocol::InvalidRequest>(
        spec.summary,
        details,
        DiagnosticSource::with_stage("docnav", "config"),
    );
    if let Some(guidance) = spec.guidance {
        draft = draft.with_guidance([guidance]);
    }
    AppError::new(draft)
}

#[cfg(test)]
mod tests;
