use docnav_adapter_contracts::AdapterOptionSpec;
use docnav_diagnostics::DiagnosticSource;
use docnav_parameter_resolution::{
    ids, ParameterResolution, ParameterResolutionHandoff, ParameterSourceKind,
    ParameterValidationIssue,
};
use docnav_typed_fields::FieldDefSet;
use serde_json::Value;

use crate::error::ConfigFieldError;
use crate::{NavigationCommand, NavigationConfigSource, NavigationConfigSources, NavigationError};

use super::{
    input::native_option_cli_value,
    native_options::{
        native_option_validation_error, spec_for_identity, unsupported_option,
        UnsupportedOptionContext,
    },
    values::{validation_error_for_identity, validation_reason_code},
};

mod key_registry;

use key_registry::{ConfigKeyIssue, ConfigKeyRegistry};

struct ConfigValidationContext<'a> {
    selected_adapter_id: &'a str,
    fields: &'a FieldDefSet,
    selected_native_options: &'a [AdapterOptionSpec],
}

pub(super) fn validate_navigation_sources(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selected_adapter_id: &str,
    fields: &FieldDefSet,
    selected_native_options: &[AdapterOptionSpec],
) -> Result<(), NavigationError> {
    first_config_source_error(config_sources)?;
    validate_explicit_native_options(command, selected_adapter_id, selected_native_options)?;
    let context = ConfigValidationContext {
        selected_adapter_id,
        fields,
        selected_native_options,
    };
    validate_config_sources(config_sources, &context)?;
    crate::outline_mode::validate_outline_config_sources(command, config_sources)
}

pub(super) fn first_resolution_error(
    resolution: &ParameterResolution,
    config_sources: &NavigationConfigSources,
) -> Result<(), NavigationError> {
    if let Some(diagnostic) = resolution.diagnostics().first() {
        return Err(handoff_error_with_config_sources(
            diagnostic,
            config_sources,
        ));
    }
    Ok(())
}

pub(super) fn first_operation_resolution_error(
    resolution: &ParameterResolution,
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selected_native_options: &[AdapterOptionSpec],
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
        return Err(handoff_error_with_config_sources(
            diagnostic,
            config_sources,
        ));
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
    context: &ConfigValidationContext<'_>,
) -> Result<(), NavigationError> {
    validate_config_source(&config_sources.project, context)?;
    validate_config_source(&config_sources.user, context)
}

fn validate_explicit_native_options(
    command: &NavigationCommand,
    selected_adapter_id: &str,
    selected_native_options: &[AdapterOptionSpec],
) -> Result<(), NavigationError> {
    for option in &command.native_options {
        if selected_native_options
            .iter()
            .any(|spec| spec.cli_flag() == Some(option.flag.as_str()))
        {
            continue;
        }
        let key = option.flag.strip_prefix("--").unwrap_or(&option.flag);
        return Err(unsupported_option(
            UnsupportedOptionContext {
                source: "explicit",
                path_origin: None,
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
    context: &ConfigValidationContext<'_>,
) -> Result<(), NavigationError> {
    let Some(value) = source.loaded.value() else {
        return Ok(());
    };
    let issue = config_key_registry(context).first_issue(value);
    if let Some(issue) = issue {
        return Err(config_key_issue_error(source, value, context, issue));
    }
    Ok(())
}

fn config_key_registry(context: &ConfigValidationContext<'_>) -> ConfigKeyRegistry {
    ConfigKeyRegistry::from_field_set(context.fields, super::CONFIG_PROCESSING)
        .leaf_path(["defaults", "adapter"])
        .leaf_path(["defaults", "output"])
        .leaf_path(["defaults", "pagination", "enabled"])
        .leaf_path(["defaults", "pagination", "limit"])
        .leaf_path(["invocation_log", "enabled"])
        .leaf_path(["invocation_log", "path"])
        .leaf_path(["invocation_log", "content_capture", "enabled"])
        .leaf_path(["invocation_log", "content_capture", "root"])
        .container_path(["options"])
        .array_item_leaf_path(["outline", "mode_rules"], "path")
        .array_item_leaf_path(["outline", "mode_rules"], "mode")
        .array_item_leaf_path(["outline", "auto_full_read", "thresholds"], "adapter")
        .array_item_leaf_path(["outline", "auto_full_read", "thresholds"], "unit")
        .array_item_leaf_path(["outline", "auto_full_read", "thresholds"], "value")
}

fn config_key_issue_error(
    source: &NavigationConfigSource,
    root: &Value,
    context: &ConfigValidationContext<'_>,
    issue: ConfigKeyIssue,
) -> NavigationError {
    match issue {
        ConfigKeyIssue::ExpectedObject { path } => {
            let field = path.field();
            invalid_nested_object(source, &field)
        }
        ConfigKeyIssue::UnregisteredField { path } => {
            if let Some(key) = path.option_key() {
                return unsupported_option(
                    UnsupportedOptionContext {
                        source: source.level,
                        path_origin: Some(source.origin),
                        path: &source.path,
                        owner: context.selected_adapter_id,
                        selected_native_options: context.selected_native_options,
                    },
                    key,
                    path.value(root).cloned().unwrap_or(Value::Null),
                )
                .into();
            }
            let field = path.field();
            NavigationError::config_unknown_field(
                source.level,
                source.origin,
                &source.path,
                field.as_str(),
                accepted_config_field(field.as_str()),
            )
        }
    }
}

fn accepted_config_field(field: &str) -> Option<&'static str> {
    match field {
        "defaults.limit" => Some("defaults.pagination.limit"),
        _ => None,
    }
}

fn invalid_nested_object(source: &NavigationConfigSource, field: &str) -> NavigationError {
    NavigationError::config_invalid_object(source.level, source.origin, &source.path, field)
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

fn handoff_error_with_config_sources(
    diagnostic: &ParameterResolutionHandoff,
    config_sources: &NavigationConfigSources,
) -> NavigationError {
    match diagnostic {
        ParameterResolutionHandoff::Validation(diagnostic) => {
            config_validation_error(diagnostic, config_sources)
                .unwrap_or_else(|| validation_error_for_identity(diagnostic.identity.as_str()))
        }
        ParameterResolutionHandoff::ConfigSource(_) => handoff_error(diagnostic),
    }
}

fn config_validation_error(
    diagnostic: &ParameterValidationIssue,
    config_sources: &NavigationConfigSources,
) -> Option<NavigationError> {
    let source = config_source_for_validation(diagnostic.source.as_ref()?.kind, config_sources)?;
    let field = config_field_for_identity(diagnostic.identity.as_str())?;
    let reason_code = validation_reason_code(&diagnostic.failure.reason);
    Some(NavigationError::config_invalid_field(
        ConfigFieldError::invalid(
            source,
            field,
            reason_code,
            format!("Use a valid value for config field {field}."),
        ),
    ))
}

fn config_source_for_validation(
    source: ParameterSourceKind,
    config_sources: &NavigationConfigSources,
) -> Option<&NavigationConfigSource> {
    match source {
        ParameterSourceKind::ProjectConfig => Some(&config_sources.project),
        ParameterSourceKind::UserConfig => Some(&config_sources.user),
        _ => None,
    }
}

fn config_field_for_identity(identity: &str) -> Option<&'static str> {
    match identity {
        ids::ADAPTER => Some("defaults.adapter"),
        ids::OUTPUT => Some("defaults.output"),
        ids::PAGINATION_ENABLED => Some("defaults.pagination.enabled"),
        ids::LIMIT => Some("defaults.pagination.limit"),
        _ => None,
    }
}
