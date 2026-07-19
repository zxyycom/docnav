use cli_config_resolution::{FieldDefSet, ResolutionResult};
use docnav_protocol::Operation;

use super::{config, fields, ids, resolve_command_with_fields, values, DocumentParameterCatalog};
use crate::{
    NavigationCommand, NavigationConfigSources, NavigationContextDefaults, NavigationError,
    NavigationPaginationDefaults, NavigationResolvedValue,
};

pub(crate) fn resolve_context_defaults(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selected_adapter_id: &str,
    catalog: &DocumentParameterCatalog,
) -> Result<NavigationContextDefaults, NavigationError> {
    let operation_fields =
        fields::operation_fields(command.operation, selected_adapter_id, catalog)?;
    config::validate_navigation_sources(
        command,
        config_sources,
        selected_adapter_id,
        &operation_fields,
        catalog,
    )?;

    let resolution =
        resolve_command_with_fields(operation_fields.as_ref(), command, config_sources)?;

    config::first_operation_resolution_error(
        &resolution,
        config_sources,
        selected_adapter_id,
        command.operation,
        catalog,
    )?;

    defaults_from_resolution(command.operation, operation_fields.as_ref(), &resolution)
}

fn defaults_from_resolution(
    operation: Operation,
    fields: &FieldDefSet,
    resolution: &ResolutionResult,
) -> Result<NavigationContextDefaults, NavigationError> {
    Ok(NavigationContextDefaults {
        adapter: resolved_value(fields, resolution, ids::ADAPTER)
            .unwrap_or_else(|| NavigationResolvedValue::new(serde_json::Value::Null, "unset")),
        pagination: if values::uses_document_window(operation) {
            Some(NavigationPaginationDefaults {
                enabled: required_resolved_value(fields, resolution, ids::PAGINATION_ENABLED)?,
                limit: required_resolved_value(fields, resolution, ids::LIMIT)?,
            })
        } else {
            None
        },
        output: required_resolved_value(fields, resolution, ids::OUTPUT)?,
        page: if values::uses_document_window(operation) {
            Some(required_resolved_value(fields, resolution, ids::PAGE)?)
        } else {
            None
        },
    })
}

fn required_resolved_value(
    fields: &FieldDefSet,
    resolution: &ResolutionResult,
    identity: &str,
) -> Result<NavigationResolvedValue, NavigationError> {
    resolved_value(fields, resolution, identity)
        .ok_or_else(|| NavigationError::internal("missing-resolved-navigation-parameter"))
}

fn resolved_value(
    fields: &FieldDefSet,
    resolution: &ResolutionResult,
    identity: &str,
) -> Option<NavigationResolvedValue> {
    let identity = values::identity_key(identity).ok()?;
    let value = resolution.fields().get(&identity)?;
    let typed = values::projected_field_value(fields, &identity, value)?;
    Some(NavigationResolvedValue::new(
        values::typed_value_to_json(typed),
        values::field_source_label(value).unwrap_or("built_in"),
    ))
}
