use std::collections::BTreeSet;

use docnav_adapter_contracts::AdapterOptionSpec;
use docnav_diagnostics::DiagnosticSource;
use docnav_parameter_resolution::{
    ids, ParameterResolution, ParameterResolutionHandoff, ParameterResolutionPipeline,
    ParameterSourceKind, ParameterValidationIssue,
};
use docnav_typed_fields::{FieldDefSet, ProcessingId};
use serde_json::Value;

use crate::error::ConfigFieldError;
use crate::{
    NavigationAdapterRegistry, NavigationCommand, NavigationConfigSource, NavigationConfigSources,
    NavigationError,
};

use super::{
    fields,
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
    selected_adapter_id: Option<&'a str>,
    fields: &'a FieldDefSet,
    selected_native_options: &'a [AdapterOptionSpec],
    known_adapter_ids: BTreeSet<String>,
    known_adapter_fields: &'a FieldDefSet,
}

pub(super) fn validate_navigation_sources(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selected_adapter_id: &str,
    fields: &FieldDefSet,
    selected_native_options: &[AdapterOptionSpec],
    registry: &(impl NavigationAdapterRegistry + ?Sized),
) -> Result<(), NavigationError> {
    first_config_source_error(config_sources)?;
    validate_explicit_native_options(command, selected_adapter_id, selected_native_options)?;
    let known_adapter_fields = fields::config_inspection_fields(registry)?;
    let context = ConfigValidationContext {
        selected_adapter_id: Some(selected_adapter_id),
        fields,
        selected_native_options,
        known_adapter_ids: known_adapter_ids(registry),
        known_adapter_fields: known_adapter_fields.as_ref(),
    };
    validate_config_sources(config_sources, &context)?;
    crate::outline_mode::validate_outline_config_sources(command, config_sources)
}

pub(super) fn validate_config_source_for_registry(
    source: &NavigationConfigSource,
    registry: &(impl NavigationAdapterRegistry + ?Sized),
) -> Result<(), NavigationError> {
    first_config_source_error_for_source(source)?;
    let known_adapter_fields = fields::config_inspection_fields(registry)?;
    let context = ConfigValidationContext {
        selected_adapter_id: None,
        fields: known_adapter_fields.as_ref(),
        selected_native_options: &[],
        known_adapter_ids: known_adapter_ids(registry),
        known_adapter_fields: known_adapter_fields.as_ref(),
    };
    validate_config_source(source, &context)?;
    crate::outline_mode::validate_outline_config_source(source)
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

fn first_config_source_error_for_source(
    source: &NavigationConfigSource,
) -> Result<(), NavigationError> {
    if let Some(diagnostic) = source.loaded.diagnostics().first() {
        return Err(handoff_error(diagnostic));
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
                config_field: None,
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
    validate_selected_adapter_options(source, value, context)?;
    validate_config_source_values(source, context)?;
    Ok(())
}

fn config_key_registry(context: &ConfigValidationContext<'_>) -> ConfigKeyRegistry {
    ConfigKeyRegistry::from_field_set(context.fields, super::CONFIG_PROCESSING)
        .field_set(context.known_adapter_fields, super::CONFIG_PROCESSING)
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

fn validate_selected_adapter_options(
    source: &NavigationConfigSource,
    root: &Value,
    context: &ConfigValidationContext<'_>,
) -> Result<(), NavigationError> {
    let Some(selected_adapter_id) = context.selected_adapter_id else {
        return Ok(());
    };
    let Some(options) = root.get("options").and_then(Value::as_object) else {
        return Ok(());
    };
    let Some(selected_options) = options.get(selected_adapter_id).and_then(Value::as_object) else {
        return Ok(());
    };
    for (key, value) in selected_options {
        if context
            .selected_native_options
            .iter()
            .any(|spec| spec.key() == key)
        {
            continue;
        }
        return Err(unsupported_option(
            UnsupportedOptionContext {
                source: source.level,
                path_origin: Some(source.origin),
                path: &source.path,
                owner: selected_adapter_id,
                config_field: Some(format!("options.{selected_adapter_id}.{key}")),
                selected_native_options: context.selected_native_options,
            },
            key,
            value.clone(),
        )
        .into());
    }
    Ok(())
}

fn validate_config_source_values(
    source: &NavigationConfigSource,
    context: &ConfigValidationContext<'_>,
) -> Result<(), NavigationError> {
    let mut pipeline = ParameterResolutionPipeline::new(context.fields)
        .with_direct_input_processing_id(super::DIRECT_PROCESSING)
        .with_config_processing_id(super::CONFIG_PROCESSING)
        .with_passthrough_policy(docnav_parameter_resolution::EntryPassthroughPolicy::Discard);
    pipeline = match source.level {
        "project" => pipeline.with_loaded_project_config(source.loaded.clone()),
        "user" => pipeline.with_loaded_user_config(source.loaded.clone()),
        _ => pipeline,
    };
    let resolution = pipeline
        .resolve(None)
        .map_err(|_| NavigationError::internal("config-source-validation-resolution-failed"))?;

    for diagnostic in resolution.diagnostics() {
        let ParameterResolutionHandoff::Validation(diagnostic) = diagnostic else {
            continue;
        };
        if !validation_belongs_to_source(diagnostic, source) {
            continue;
        }
        if let Some(error) = config_validation_error_for_source(diagnostic, source, context.fields)
        {
            return Err(error);
        }
        return Err(validation_error_for_identity(diagnostic.identity.as_str()));
    }

    Ok(())
}

fn validation_belongs_to_source(
    diagnostic: &ParameterValidationIssue,
    source: &NavigationConfigSource,
) -> bool {
    matches!(
        (
            diagnostic.source.as_ref().map(|source| source.kind),
            source.level
        ),
        (Some(ParameterSourceKind::ProjectConfig), "project")
            | (Some(ParameterSourceKind::UserConfig), "user")
    )
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
            if let Some(adapter_id) = path.option_adapter_id() {
                if !context.known_adapter_ids.contains(adapter_id) {
                    let field = path.field();
                    let reason = if path.value(root).is_some_and(Value::is_object) {
                        "unknown_adapter_id"
                    } else {
                        "unknown_config_field"
                    };
                    return NavigationError::config_invalid_field(ConfigFieldError::invalid(
                        source,
                        field,
                        reason,
                        "Use a registered adapter id under options.<adapter-id>.",
                    ));
                }
            }
            if let (Some(selected_adapter_id), Some(adapter_id), Some(key)) = (
                context.selected_adapter_id,
                path.option_adapter_id(),
                path.option_key(),
            ) {
                if adapter_id == selected_adapter_id {
                    return unsupported_option(
                        UnsupportedOptionContext {
                            source: source.level,
                            path_origin: Some(source.origin),
                            path: &source.path,
                            owner: selected_adapter_id,
                            config_field: Some(path.field()),
                            selected_native_options: context.selected_native_options,
                        },
                        key,
                        path.value(root).cloned().unwrap_or(Value::Null),
                    )
                    .into();
                }
            }
            if let Some(key) = path.option_key() {
                return unsupported_option(
                    UnsupportedOptionContext {
                        source: source.level,
                        path_origin: Some(source.origin),
                        path: &source.path,
                        owner: path.option_adapter_id().unwrap_or("options"),
                        config_field: Some(path.field()),
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

fn known_adapter_ids(registry: &(impl NavigationAdapterRegistry + ?Sized)) -> BTreeSet<String> {
    registry
        .adapters()
        .into_iter()
        .map(|adapter| adapter.id().to_owned())
        .collect()
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
    let field = legacy_config_field_for_identity(diagnostic.identity.as_str())?;
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

fn config_validation_error_for_source(
    diagnostic: &ParameterValidationIssue,
    source: &NavigationConfigSource,
    fields: &FieldDefSet,
) -> Option<NavigationError> {
    let field = config_field_for_identity(diagnostic.identity.as_str(), fields)?;
    let reason_code = validation_reason_code(&diagnostic.failure.reason);
    Some(NavigationError::config_invalid_field(
        ConfigFieldError::invalid(
            source,
            field.clone(),
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

fn config_field_for_identity(identity: &str, fields: &FieldDefSet) -> Option<String> {
    fields
        .processing_metadata(&ProcessingId::from(super::CONFIG_PROCESSING))
        .into_iter()
        .find(|metadata| metadata.identity.as_str() == identity)
        .map(|metadata| metadata.path.segments().join("."))
}

fn legacy_config_field_for_identity(identity: &str) -> Option<&'static str> {
    match identity {
        ids::ADAPTER => Some("defaults.adapter"),
        ids::OUTPUT => Some("defaults.output"),
        ids::PAGINATION_ENABLED => Some("defaults.pagination.enabled"),
        ids::LIMIT => Some("defaults.pagination.limit"),
        _ => None,
    }
}
