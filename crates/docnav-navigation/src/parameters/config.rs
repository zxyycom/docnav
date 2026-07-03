use docnav_adapter_contracts::NativeOptionSpec;
use docnav_diagnostics::DiagnosticSource;
use docnav_parameter_resolution::{ParameterResolution, ParameterResolutionHandoff};
use serde_json::Value;

use crate::{NavigationCommand, NavigationConfigSource, NavigationConfigSources, NavigationError};

use super::{
    input::native_option_cli_value,
    native_options::{
        native_option_validation_error, spec_for_identity, unsupported_option,
        UnsupportedOptionContext,
    },
    values::validation_error_for_identity,
};

pub(super) fn validate_navigation_sources(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selected_adapter_id: &str,
    selected_native_options: &[NativeOptionSpec],
) -> Result<(), NavigationError> {
    first_config_source_error(config_sources)?;
    validate_explicit_native_options(command, selected_adapter_id, selected_native_options)?;
    validate_config_sources(config_sources, selected_adapter_id, selected_native_options)
}

pub(super) fn first_resolution_error(
    resolution: &ParameterResolution,
) -> Result<(), NavigationError> {
    if let Some(diagnostic) = resolution.diagnostics().first() {
        return Err(handoff_error(diagnostic));
    }
    Ok(())
}

pub(super) fn first_operation_resolution_error(
    resolution: &ParameterResolution,
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selected_native_options: &[NativeOptionSpec],
) -> Result<(), NavigationError> {
    if let Some(diagnostic) = resolution.diagnostics().first() {
        if let ParameterResolutionHandoff::Validation(diagnostic) = diagnostic {
            if let Some(spec) =
                spec_for_identity(selected_native_options, diagnostic.identity.as_str())
            {
                return Err(native_option_validation_error(
                    command,
                    config_sources,
                    spec,
                    diagnostic.source.as_ref().map(|source| source.kind),
                    &diagnostic.failure.reason,
                )
                .into());
            }
        }
        return Err(handoff_error(diagnostic));
    }
    Ok(())
}

fn first_config_source_error(
    config_sources: &NavigationConfigSources,
) -> Result<(), NavigationError> {
    for source in [&config_sources.project, &config_sources.user] {
        if let Some(diagnostic) = source.loaded.diagnostics().first() {
            return Err(handoff_error(diagnostic));
        }
    }
    Ok(())
}

fn validate_config_sources(
    config_sources: &NavigationConfigSources,
    selected_adapter_id: &str,
    selected_native_options: &[NativeOptionSpec],
) -> Result<(), NavigationError> {
    validate_config_source(
        &config_sources.project,
        selected_adapter_id,
        selected_native_options,
    )?;
    validate_config_source(
        &config_sources.user,
        selected_adapter_id,
        selected_native_options,
    )
}

fn validate_explicit_native_options(
    command: &NavigationCommand,
    selected_adapter_id: &str,
    selected_native_options: &[NativeOptionSpec],
) -> Result<(), NavigationError> {
    for option in &command.native_options {
        if selected_native_options
            .iter()
            .any(|spec| spec.cli_flag == Some(option.flag.as_str()))
        {
            continue;
        }
        let key = option.flag.strip_prefix("--").unwrap_or(&option.flag);
        return Err(unsupported_option(
            UnsupportedOptionContext {
                source: "explicit",
                path: "command",
                owner: selected_adapter_id,
                selected_native_options,
            },
            key,
            native_option_cli_value(&option.value),
        )
        .into());
    }
    Ok(())
}

fn validate_config_source(
    source: &NavigationConfigSource,
    selected_adapter_id: &str,
    selected_native_options: &[NativeOptionSpec],
) -> Result<(), NavigationError> {
    let Some(value) = source.loaded.value() else {
        return Ok(());
    };
    let Some(root) = value.as_object() else {
        return Ok(());
    };
    for (key, child) in root {
        match key.as_str() {
            "defaults" => validate_defaults_shape(source, child)?,
            "options" => {
                validate_options_shape(source, child, selected_adapter_id, selected_native_options)?
            }
            _ => {
                return Err(NavigationError::config_unknown_field(
                    source.level,
                    &source.path,
                    key,
                    None,
                ));
            }
        }
    }
    Ok(())
}

fn validate_defaults_shape(
    source: &NavigationConfigSource,
    value: &Value,
) -> Result<(), NavigationError> {
    let Some(defaults) = value.as_object() else {
        return Err(invalid_nested_object(source, "defaults"));
    };
    for (key, child) in defaults {
        match key.as_str() {
            "adapter" | "output" => {}
            "pagination" => validate_pagination_shape(source, child)?,
            "limit" => {
                return Err(NavigationError::config_unknown_field(
                    source.level,
                    &source.path,
                    "defaults.limit",
                    Some("defaults.pagination.limit"),
                ));
            }
            _ => {
                return Err(NavigationError::config_unknown_field(
                    source.level,
                    &source.path,
                    format!("defaults.{key}"),
                    None,
                ));
            }
        }
    }
    Ok(())
}

fn validate_pagination_shape(
    source: &NavigationConfigSource,
    value: &Value,
) -> Result<(), NavigationError> {
    let Some(pagination) = value.as_object() else {
        return Err(invalid_nested_object(source, "defaults.pagination"));
    };
    for key in pagination.keys() {
        match key.as_str() {
            "enabled" | "limit" => {}
            _ => {
                return Err(NavigationError::config_unknown_field(
                    source.level,
                    &source.path,
                    format!("defaults.pagination.{key}"),
                    None,
                ));
            }
        }
    }
    Ok(())
}

fn validate_options_shape(
    source: &NavigationConfigSource,
    value: &Value,
    selected_adapter_id: &str,
    selected_native_options: &[NativeOptionSpec],
) -> Result<(), NavigationError> {
    let Some(options) = value.as_object() else {
        return Err(invalid_nested_object(source, "options"));
    };
    for (key, value) in options {
        if selected_native_options.iter().any(|spec| spec.key == key) {
            continue;
        }
        return Err(unsupported_option(
            UnsupportedOptionContext {
                source: source.level,
                path: &source.path,
                owner: selected_adapter_id,
                selected_native_options,
            },
            key,
            value.clone(),
        )
        .into());
    }
    Ok(())
}

fn invalid_nested_object(source: &NavigationConfigSource, field: &str) -> NavigationError {
    NavigationError::invalid_request(
        field,
        format!(
            "{} config field {field} in {} must be an object",
            source.level, source.path
        ),
    )
}

fn handoff_error(diagnostic: &ParameterResolutionHandoff) -> NavigationError {
    match diagnostic {
        ParameterResolutionHandoff::Validation(diagnostic) => {
            validation_error_for_identity(diagnostic.identity.as_str())
        }
        ParameterResolutionHandoff::ConfigSource(issue) => NavigationError::new(
            issue.to_record_draft(DiagnosticSource::with_stage("docnav", "config")),
        ),
    }
}
