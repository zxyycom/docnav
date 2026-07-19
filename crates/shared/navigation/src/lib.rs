//! Source resolution, multi-adapter routing, and strategy dispatch for Docnav.
//!
//! Registries expose adapter definitions directly. Navigation probes and selects among all
//! registered adapters, constructs closed operation input, and dispatches through the selected
//! strategy while preserving adapter-owned capabilities and full-read hooks.

mod auto_read;
mod config_source;
mod context;
mod error;
mod execution;
mod model;
mod outline_mode;
mod parameters;
mod protocol;
mod routing;

use config_source::{load_config_source, LoadedNavigationConfigSource};
use serde_json::Value;

pub use context::{select_navigation_context, NavigationContextSelection};
pub use error::NavigationError;
#[cfg(test)]
use execution::{execute_loaded_navigation_command, validate_navigation_response};
pub use model::{
    AutoReadMode, NavigationCommand, NavigationCommandOutcome, NavigationConfigSourceDescriptor,
    NavigationConfigSourceDescriptors, NavigationConfigSourceLevel, NavigationConfigSourceOrigin,
    NavigationContextDefaults, NavigationContextOutcome, NavigationFailureLayer,
    NavigationInvocationTrace, NavigationOutputMode, NavigationPaginationDefaults,
    NavigationResolvedValue, DOCUMENT_CLI_SOURCE_ID, DOCUMENT_CLI_SOURCE_PRIORITY,
};
use model::{NavigationConfigSource, NavigationConfigSources};
#[cfg(test)]
use parameters::resolve_operation_input;
use parameters::{resolve_adapter_intent, resolve_context_defaults};
pub use parameters::{
    DocumentParameterBinding, DocumentParameterCatalog, DocumentParameterCatalogBuildError,
    DocumentParameterEntry,
};
pub use protocol::{
    execute_operation, execute_protocol_request, protocol_request, NavigationInputError,
    OperationInput,
};
pub use routing::{
    select_adapter, AdapterSelectionRequest, CandidateEvidence, NavigationAdapterRegistry,
};

/// Builds the adapter-routing CLI projection independently from product parameters.
pub fn document_adapter_routing_fields(
) -> Result<docnav_typed_fields::FieldDefSet, docnav_typed_fields::FieldDefSetBuildError> {
    parameters::adapter_routing_fields()
}

pub fn execute_navigation_command<R>(
    command: NavigationCommand,
    config_sources: NavigationConfigSourceDescriptors,
    catalog: &DocumentParameterCatalog,
    registry: &R,
) -> Result<NavigationCommandOutcome, NavigationError>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    execution::execute_loaded_navigation_command(
        command,
        load_navigation_config_sources(config_sources),
        catalog,
        registry,
    )
}

pub fn inspect_navigation_config_sources(
    config_sources: NavigationConfigSourceDescriptors,
    catalog: &DocumentParameterCatalog,
) -> Result<Value, NavigationError> {
    parameters::inspect_config_sources(&load_navigation_config_sources(config_sources), catalog)
}

pub fn validate_navigation_config_source_value(
    level: NavigationConfigSourceLevel,
    origin: NavigationConfigSourceOrigin,
    path: impl Into<String>,
    value: Value,
    catalog: &DocumentParameterCatalog,
) -> Result<(), NavigationError> {
    let source = NavigationConfigSource {
        level,
        origin,
        path: path.into(),
        loaded: LoadedNavigationConfigSource::from_value(value),
    };
    parameters::validate_config_source_for_catalog(&source, catalog)
}

pub fn resolve_navigation_context<R>(
    command: NavigationCommand,
    config_sources: NavigationConfigSourceDescriptors,
    catalog: &DocumentParameterCatalog,
    registry: &R,
) -> Result<NavigationContextOutcome, NavigationError>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    resolve_loaded_navigation_context(
        command,
        load_navigation_config_sources(config_sources),
        catalog,
        registry,
    )
}

fn resolve_loaded_navigation_context<R>(
    command: NavigationCommand,
    config_sources: NavigationConfigSources,
    catalog: &DocumentParameterCatalog,
    registry: &R,
) -> Result<NavigationContextOutcome, NavigationError>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    let adapter_intent = resolve_adapter_intent(&command, &config_sources)?;
    let selection = select_adapter(AdapterSelectionRequest {
        registry,
        document_path: &command.document_path,
        preselected_adapter_id: adapter_intent.adapter_id.as_deref(),
        preselected_adapter_source: adapter_intent.source,
    })?;
    let defaults =
        resolve_context_defaults(&command, &config_sources, selection.adapter.id(), catalog)?;
    let selection = NavigationContextSelection::from_selection(
        &selection,
        adapter_intent.adapter_id.as_deref(),
        adapter_intent.source,
    );

    Ok(NavigationContextOutcome {
        selection,
        defaults,
    })
}

fn load_navigation_config_sources(
    descriptors: NavigationConfigSourceDescriptors,
) -> NavigationConfigSources {
    NavigationConfigSources {
        project: load_navigation_config_source(
            NavigationConfigSourceLevel::Project,
            descriptors.project,
        ),
        user: load_navigation_config_source(NavigationConfigSourceLevel::User, descriptors.user),
    }
}

fn load_navigation_config_source(
    level: NavigationConfigSourceLevel,
    descriptor: NavigationConfigSourceDescriptor,
) -> NavigationConfigSource {
    NavigationConfigSource {
        level,
        origin: descriptor.origin,
        path: descriptor.path.display().to_string(),
        loaded: load_config_source(level, descriptor.origin, &descriptor.path),
    }
}

#[cfg(test)]
mod tests;
