use cli_config_resolution::{ResolutionDiagnostic, ResolutionResult, SourceLocator};
use docnav_diagnostics::DiagnosticSource;
use docnav_protocol::Operation;
use docnav_typed_fields::{FieldDefSet, ProcessingId};
use serde_json::Value;

use crate::config_source::NavigationConfigSourceIssue;
use crate::error::ConfigFieldError;
use crate::{NavigationConfigSource, NavigationConfigSources, NavigationError};

use super::super::{
    catalog::DocumentParameterCatalog,
    native_options::{
        catalog_option_for_identity, native_option_validation_error, unsupported_option,
        UnsupportedOptionContext,
    },
    values::{resolution_reason_code, validation_error_for_identity},
    CONFIG_PROCESSING,
};
use super::key_registry::{ConfigKeyIssue, ConfigValuePath};
use super::ConfigValidationContext;

pub(super) fn config_key_issue_error(
    source: &NavigationConfigSource,
    root: &Value,
    context: &ConfigValidationContext<'_>,
    issue: ConfigKeyIssue,
) -> NavigationError {
    match issue {
        ConfigKeyIssue::ExpectedObject { path } => invalid_nested_object(source, &path.field()),
        ConfigKeyIssue::UnregisteredField { path } => {
            unregistered_config_key_error(source, root, context, &path)
        }
    }
}

fn unregistered_config_key_error(
    source: &NavigationConfigSource,
    root: &Value,
    context: &ConfigValidationContext<'_>,
    path: &ConfigValuePath,
) -> NavigationError {
    unknown_adapter_error(source, root, context, path)
        .or_else(|| selected_adapter_option_error(source, root, context, path))
        .or_else(|| unsupported_config_option_error(source, root, context, path))
        .unwrap_or_else(|| unknown_config_field_error(source, path))
}

fn unknown_adapter_error(
    source: &NavigationConfigSource,
    root: &Value,
    context: &ConfigValidationContext<'_>,
    path: &ConfigValuePath,
) -> Option<NavigationError> {
    let adapter_id = path.option_adapter_id()?;
    if context.catalog.is_known_adapter_id(adapter_id) {
        return None;
    }
    let field = path.field();
    let reason = if path.value(root).is_some_and(Value::is_object) {
        "unknown_adapter_id"
    } else {
        "unknown_config_field"
    };
    Some(NavigationError::config_invalid_field(
        ConfigFieldError::invalid(
            source,
            field,
            reason,
            "Use a registered adapter id under options.<adapter-id>.",
        ),
    ))
}

fn selected_adapter_option_error(
    source: &NavigationConfigSource,
    root: &Value,
    context: &ConfigValidationContext<'_>,
    path: &ConfigValuePath,
) -> Option<NavigationError> {
    let selected_adapter_id = context.selected_adapter_id?;
    if path.option_adapter_id()? != selected_adapter_id {
        return None;
    }
    unsupported_option_error(source, root, context, path, selected_adapter_id)
}

fn unsupported_config_option_error(
    source: &NavigationConfigSource,
    root: &Value,
    context: &ConfigValidationContext<'_>,
    path: &ConfigValuePath,
) -> Option<NavigationError> {
    let owner = path.option_adapter_id().unwrap_or("options");
    unsupported_option_error(source, root, context, path, owner)
}

fn unsupported_option_error(
    source: &NavigationConfigSource,
    root: &Value,
    context: &ConfigValidationContext<'_>,
    path: &ConfigValuePath,
    owner: &str,
) -> Option<NavigationError> {
    let key = path.option_key()?;
    Some(
        unsupported_option(
            UnsupportedOptionContext {
                source: source.level.as_str(),
                path_origin: Some(source.origin.as_str()),
                path: &source.path,
                owner,
                config_field: Some(path.field()),
                operation: context.operation,
                catalog: context.catalog,
            },
            key,
            path.value(root).cloned().unwrap_or(Value::Null),
        )
        .into(),
    )
}

fn unknown_config_field_error(
    source: &NavigationConfigSource,
    path: &ConfigValuePath,
) -> NavigationError {
    let field = path.field();
    NavigationError::config_unknown_field(
        source.level.as_str(),
        source.origin.as_str(),
        &source.path,
        field.as_str(),
        accepted_config_field(field.as_str()),
    )
}

fn accepted_config_field(field: &str) -> Option<&'static str> {
    match field {
        "defaults.limit" => Some("defaults.pagination.limit"),
        _ => None,
    }
}

fn invalid_nested_object(source: &NavigationConfigSource, field: &str) -> NavigationError {
    NavigationError::config_invalid_object(
        source.level.as_str(),
        source.origin.as_str(),
        &source.path,
        field,
    )
}

pub(super) fn config_source_error(issue: &NavigationConfigSourceIssue) -> NavigationError {
    NavigationError::new(issue.to_record_draft(DiagnosticSource::with_stage("docnav", "config")))
}

pub(in crate::parameters) fn first_resolution_error(
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

pub(in crate::parameters) fn first_operation_resolution_error(
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

pub(super) fn first_config_source_error(
    config_sources: &NavigationConfigSources,
) -> Result<(), NavigationError> {
    for source in [&config_sources.project, &config_sources.user] {
        if let Some(issue) = source.loaded.diagnostics().first() {
            return Err(config_source_error(issue));
        }
    }
    Ok(())
}

pub(super) fn first_config_source_error_for_source(
    source: &NavigationConfigSource,
) -> Result<(), NavigationError> {
    if let Some(issue) = source.loaded.diagnostics().first() {
        return Err(config_source_error(issue));
    }
    Ok(())
}

pub(super) fn resolution_error_with_config_sources(
    diagnostic: &ResolutionDiagnostic,
    config_sources: &NavigationConfigSources,
) -> NavigationError {
    config_validation_error(diagnostic, config_sources)
        .unwrap_or_else(|| validation_error_for_identity(diagnostic.field.as_str()))
}

fn config_validation_error(
    diagnostic: &ResolutionDiagnostic,
    config_sources: &NavigationConfigSources,
) -> Option<NavigationError> {
    let source = config_source_for_validation(diagnostic.source_id.as_ref()?, config_sources)?;
    let field = config_field_from_diagnostic(diagnostic)?;
    Some(invalid_config_value_error(diagnostic, source, field))
}

pub(super) fn config_validation_error_for_source(
    diagnostic: &ResolutionDiagnostic,
    source: &NavigationConfigSource,
    fields: &FieldDefSet,
) -> Option<NavigationError> {
    let field = config_field_from_diagnostic(diagnostic)
        .or_else(|| config_field_for_identity(diagnostic.field.as_str(), fields))?;
    Some(invalid_config_value_error(diagnostic, source, field))
}

fn invalid_config_value_error(
    diagnostic: &ResolutionDiagnostic,
    source: &NavigationConfigSource,
    field: String,
) -> NavigationError {
    let reason_code = resolution_reason_code(diagnostic);
    NavigationError::config_invalid_field(ConfigFieldError::invalid(
        source,
        field.clone(),
        reason_code,
        format!("Use a valid value for config field {field}."),
    ))
}

fn config_source_for_validation<'a>(
    source_id: &cli_config_resolution::SourceId,
    config_sources: &'a NavigationConfigSources,
) -> Option<&'a NavigationConfigSource> {
    [&config_sources.project, &config_sources.user]
        .into_iter()
        .find(|source| source.level.as_str() == source_id.as_str())
}

fn config_field_from_diagnostic(diagnostic: &ResolutionDiagnostic) -> Option<String> {
    match diagnostic.locator.as_ref()? {
        SourceLocator::ConfigPath(path) => Some(path.segments().join(".")),
        _ => None,
    }
}

fn config_field_for_identity(identity: &str, fields: &FieldDefSet) -> Option<String> {
    fields
        .processing_metadata(
            &ProcessingId::new(CONFIG_PROCESSING).expect("config processing id is valid"),
        )
        .into_iter()
        .find(|metadata| metadata.identity().as_str() == identity)
        .map(|metadata| metadata.path.segments().join("."))
}
