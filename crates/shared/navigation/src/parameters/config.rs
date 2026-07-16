use std::collections::BTreeSet;

use cli_config_resolution::{
    ResolutionDiagnostic, ResolutionResult, SourceCandidate, SourceLocator,
};
use docnav_adapter_contracts::AdapterOptionSpec;
use docnav_typed_fields::{FieldDefSet, ProcessingId};
use serde_json::Value;

use crate::config_source::LoadedNavigationConfigSource;
use crate::{
    NavigationAdapterRegistry, NavigationCommand, NavigationConfigSource,
    NavigationConfigSourceLevel, NavigationConfigSourceOrigin, NavigationConfigSources,
    NavigationError,
};

use super::{
    fields,
    native_options::{
        native_option_validation_error, spec_for_identity, unsupported_option,
        UnsupportedOptionContext,
    },
    resolution,
    values::{diagnostic_source_label, resolution_reason_code, validation_error_for_identity},
};

mod errors;
mod key_registry;

use errors::{
    config_key_issue_error, config_source_error, config_validation_error_for_source,
    resolution_error_with_config_sources,
};
use key_registry::ConfigKeyRegistry;

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
    operation_fields: &fields::OperationFieldSet,
    registry: &(impl NavigationAdapterRegistry + ?Sized),
) -> Result<(), NavigationError> {
    let fields = operation_fields.as_ref();
    let selected_native_options = operation_fields.adapter_options();
    first_config_source_error(config_sources)?;
    validate_explicit_cli_candidates(
        command,
        selected_adapter_id,
        fields,
        selected_native_options,
    )?;
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
    resolution: &ResolutionResult,
    config_sources: &NavigationConfigSources,
) -> Result<(), NavigationError> {
    first_config_source_error(config_sources)?;
    if let Some(diagnostic) = resolution.diagnostics().first() {
        return Err(resolution_error_with_config_sources(
            diagnostic,
            config_sources,
        ));
    }
    Ok(())
}

pub(super) fn first_operation_resolution_error(
    resolution: &ResolutionResult,
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selected_native_options: &[AdapterOptionSpec],
) -> Result<(), NavigationError> {
    if let Some(diagnostic) = resolution.diagnostics().first() {
        if let Some(spec) = spec_for_identity(selected_native_options, diagnostic.field.as_str()) {
            return Err(native_option_validation_error(
                command,
                config_sources,
                spec,
                diagnostic_source_label(diagnostic),
                resolution_reason_code(diagnostic),
            )
            .into());
        }
        return Err(resolution_error_with_config_sources(
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
        if let Some(issue) = source.loaded.diagnostics().first() {
            return Err(config_source_error(issue));
        }
    }
    Ok(())
}

fn first_config_source_error_for_source(
    source: &NavigationConfigSource,
) -> Result<(), NavigationError> {
    if let Some(issue) = source.loaded.diagnostics().first() {
        return Err(config_source_error(issue));
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

fn validate_explicit_cli_candidates(
    command: &NavigationCommand,
    selected_adapter_id: &str,
    fields: &FieldDefSet,
    selected_native_options: &[AdapterOptionSpec],
) -> Result<(), NavigationError> {
    for candidate in command.cli_source.candidates() {
        if fields.field(candidate.field()).is_some() {
            continue;
        }
        let key = cli_candidate_key(candidate);
        return Err(unsupported_option(
            UnsupportedOptionContext {
                source: "explicit",
                path_origin: None,
                path: "command",
                owner: selected_adapter_id,
                config_field: None,
                selected_native_options,
            },
            &key,
            candidate.input().raw().clone(),
        )
        .into());
    }
    Ok(())
}

fn cli_candidate_key(candidate: &SourceCandidate) -> String {
    match candidate.locator() {
        SourceLocator::CliFlag(flag) => flag.strip_prefix("--").unwrap_or(flag).to_owned(),
        _ => candidate.field().as_str().to_owned(),
    }
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
    let processing_id =
        ProcessingId::new(super::CONFIG_PROCESSING).expect("config processing id is valid");
    ConfigKeyRegistry::from_field_set(context.fields, &processing_id)
        .field_set(context.known_adapter_fields, &processing_id)
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
                source: source.level.as_str(),
                path_origin: Some(source.origin.as_str()),
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
    let config_sources = match source.level {
        NavigationConfigSourceLevel::Project => NavigationConfigSources {
            project: source.clone(),
            user: empty_config_source(NavigationConfigSourceLevel::User),
        },
        NavigationConfigSourceLevel::User => NavigationConfigSources {
            project: empty_config_source(NavigationConfigSourceLevel::Project),
            user: source.clone(),
        },
    };
    let resolution = resolution::resolve(context.fields, None, None, &config_sources)?;

    for diagnostic in resolution.diagnostics() {
        if !validation_belongs_to_source(diagnostic, source) {
            continue;
        }
        if let Some(error) = config_validation_error_for_source(diagnostic, source, context.fields)
        {
            return Err(error);
        }
        return Err(validation_error_for_identity(diagnostic.field.as_str()));
    }

    Ok(())
}

fn validation_belongs_to_source(
    diagnostic: &ResolutionDiagnostic,
    source: &NavigationConfigSource,
) -> bool {
    diagnostic
        .source_id
        .as_ref()
        .is_some_and(|source_id| source_id.as_str() == source.level.as_str())
}

fn empty_config_source(level: NavigationConfigSourceLevel) -> NavigationConfigSource {
    NavigationConfigSource {
        level,
        origin: NavigationConfigSourceOrigin::Default,
        path: String::new(),
        loaded: LoadedNavigationConfigSource::default(),
    }
}

fn known_adapter_ids(registry: &(impl NavigationAdapterRegistry + ?Sized)) -> BTreeSet<String> {
    registry
        .adapters()
        .into_iter()
        .map(|adapter| adapter.id().to_owned())
        .collect()
}
