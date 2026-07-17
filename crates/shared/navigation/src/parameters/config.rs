use cli_config_resolution::{
    ResolutionDiagnostic, ResolutionResult, SourceCandidate, SourceLocator,
};
use docnav_protocol::Operation;
use docnav_typed_fields::{FieldDefSet, ProcessingId};
use serde_json::Value;

use crate::config_source::LoadedNavigationConfigSource;
use crate::{
    NavigationCommand, NavigationConfigSource, NavigationConfigSourceLevel,
    NavigationConfigSourceOrigin, NavigationConfigSources, NavigationError,
};

use super::{
    catalog::DocumentParameterCatalog,
    fields,
    native_options::{
        catalog_option_for_identity, native_option_validation_error,
        selected_adapter_supports_option, unsupported_option, UnsupportedOptionContext,
    },
    resolution,
    values::validation_error_for_identity,
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
    operation: Option<Operation>,
    selected_fields: Option<&'a FieldDefSet>,
    catalog: &'a DocumentParameterCatalog,
    routing_fields: &'a FieldDefSet,
    invocation_log_fields: &'a FieldDefSet,
}

pub(super) fn validate_navigation_sources(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selected_adapter_id: &str,
    operation_fields: &fields::OperationFieldSet,
    catalog: &DocumentParameterCatalog,
) -> Result<(), NavigationError> {
    let selected_fields = operation_fields.as_ref();
    first_config_source_error(config_sources)?;
    validate_explicit_cli_candidates(command, selected_adapter_id, selected_fields, catalog)?;
    let routing_fields = fields::adapter_routing_fields()
        .map_err(|_| NavigationError::internal("config-routing-fields-build-failed"))?;
    let invocation_log_fields = fields::invocation_log_fields()
        .map_err(|_| NavigationError::internal("config-invocation-log-fields-build-failed"))?;
    let context = ConfigValidationContext {
        selected_adapter_id: Some(selected_adapter_id),
        operation: Some(command.operation),
        selected_fields: Some(selected_fields),
        catalog,
        routing_fields: &routing_fields,
        invocation_log_fields: &invocation_log_fields,
    };
    validate_config_sources(config_sources, &context)?;
    crate::outline_mode::validate_outline_config_sources(command, config_sources)
}

pub(super) fn validate_config_source_for_catalog(
    source: &NavigationConfigSource,
    catalog: &DocumentParameterCatalog,
) -> Result<(), NavigationError> {
    first_config_source_error_for_source(source)?;
    let routing_fields = fields::adapter_routing_fields()
        .map_err(|_| NavigationError::internal("config-routing-fields-build-failed"))?;
    let invocation_log_fields = fields::invocation_log_fields()
        .map_err(|_| NavigationError::internal("config-invocation-log-fields-build-failed"))?;
    let context = ConfigValidationContext {
        selected_adapter_id: None,
        operation: None,
        selected_fields: None,
        catalog,
        routing_fields: &routing_fields,
        invocation_log_fields: &invocation_log_fields,
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
    config_sources: &NavigationConfigSources,
    selected_adapter_id: &str,
    operation: Operation,
    catalog: &DocumentParameterCatalog,
) -> Result<(), NavigationError> {
    if let Some(diagnostic) = resolution.diagnostics().first() {
        if let Some((field, entry)) = catalog_option_for_identity(
            catalog,
            selected_adapter_id,
            operation,
            diagnostic.field.as_str(),
        ) {
            return Err(
                native_option_validation_error(config_sources, field, entry, diagnostic).into(),
            );
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
    catalog: &DocumentParameterCatalog,
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
                operation: Some(command.operation),
                catalog,
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
    validate_selected_adapter_option_values(source, value, context)?;
    if let Some(selected_fields) = context.selected_fields {
        validate_config_source_values(source, selected_fields)?;
    }
    for fields in [
        context.routing_fields,
        context.catalog.fields(),
        context.invocation_log_fields,
    ] {
        validate_config_source_values(source, fields)?;
    }
    Ok(())
}

fn config_key_registry(context: &ConfigValidationContext<'_>) -> ConfigKeyRegistry {
    let processing_id =
        ProcessingId::new(super::CONFIG_PROCESSING).expect("config processing id is valid");
    let registry = ConfigKeyRegistry::from_field_set(context.routing_fields, &processing_id)
        .field_set(context.catalog.fields(), &processing_id)
        .field_set(context.invocation_log_fields, &processing_id)
        .leaf_path(["defaults", "adapter"])
        .container_path(["options"])
        .array_item_leaf_path(["outline", "mode_rules"], "path")
        .array_item_leaf_path(["outline", "mode_rules"], "mode")
        .array_item_leaf_path(["outline", "auto_full_read", "thresholds"], "adapter")
        .array_item_leaf_path(["outline", "auto_full_read", "thresholds"], "unit")
        .array_item_leaf_path(["outline", "auto_full_read", "thresholds"], "value");
    match context.selected_fields {
        Some(selected_fields) => registry.field_set(selected_fields, &processing_id),
        None => registry,
    }
}

fn validate_selected_adapter_option_values(
    source: &NavigationConfigSource,
    root: &Value,
    context: &ConfigValidationContext<'_>,
) -> Result<(), NavigationError> {
    let Some(selected_adapter_id) = context.selected_adapter_id else {
        return Ok(());
    };
    let Some(operation) = context.operation else {
        return Ok(());
    };
    let Some(options) = root.get("options").and_then(Value::as_object) else {
        return Ok(());
    };
    let Some(selected_options) = options.get(selected_adapter_id).and_then(Value::as_object) else {
        return Ok(());
    };
    for (key, value) in selected_options {
        if selected_adapter_supports_option(context.catalog, selected_adapter_id, operation, key) {
            continue;
        }
        return Err(unsupported_option(
            UnsupportedOptionContext {
                source: source.level.as_str(),
                path_origin: Some(source.origin.as_str()),
                path: &source.path,
                owner: selected_adapter_id,
                config_field: Some(format!("options.{selected_adapter_id}.{key}")),
                operation: Some(operation),
                catalog: context.catalog,
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
    fields: &FieldDefSet,
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
    let resolution = resolution::resolve(fields, None, None, &config_sources)?;

    for diagnostic in resolution.diagnostics() {
        if !validation_belongs_to_source(diagnostic, source) {
            continue;
        }
        if let Some(error) = config_validation_error_for_source(diagnostic, source, fields) {
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
