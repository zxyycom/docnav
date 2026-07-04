mod context;
mod error;
mod outline_mode;
mod parameters;
mod protocol;
mod routing;

use std::path::PathBuf;

use docnav_parameter_resolution::{
    load_parameter_config_source, ConfigPathOrigin, ConfigSourceLevel, LoadedParameterConfigSource,
    ParameterConfigSourceDescriptor,
};
use docnav_protocol::{Operation, PositiveInteger, ProtocolResponse, RequestEnvelope};
use serde_json::Value;

pub use context::{select_navigation_context, NavigationContextSelection};
pub use error::NavigationError;
use outline_mode::{execute_unstructured_outline, resolve_outline_mode, OutlineMode};
use parameters::{resolve_adapter_intent, resolve_context_defaults, resolve_operation_input};
pub use protocol::{
    execute_operation, execute_protocol_request, protocol_request, NavigationInputError,
    OperationInput,
};
pub use routing::{
    select_adapter, AdapterSelectionRequest, CandidateEvidence, NavigationAdapterRef,
    NavigationAdapterRegistry,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationOutputMode {
    ReadableView,
    ReadableJson,
    ProtocolJson,
}

impl NavigationOutputMode {
    pub const ACCEPTED_VALUES: &'static [&'static str] =
        &["readable-view", "readable-json", "protocol-json"];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadableView => "readable-view",
            Self::ReadableJson => "readable-json",
            Self::ProtocolJson => "protocol-json",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "readable-view" => Ok(Self::ReadableView),
            "readable-json" => Ok(Self::ReadableJson),
            "protocol-json" => Ok(Self::ProtocolJson),
            _ => Err(format!(
                "invalid output value {value:?}, accepted values: {}",
                Self::ACCEPTED_VALUES.join(", ")
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationCommand {
    pub operation: Operation,
    pub document_path: String,
    pub ref_id: Option<String>,
    pub query: Option<String>,
    pub page: Option<PositiveInteger>,
    pub pagination_enabled: Option<bool>,
    pub limit: Option<PositiveInteger>,
    pub output: Option<NavigationOutputMode>,
    pub adapter: Option<String>,
    pub native_options: Vec<NavigationNativeOptionInput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationNativeOptionInput {
    pub flag: String,
    pub value: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationConfigSourceOrigin {
    Default,
    Override,
}

impl NavigationConfigSourceOrigin {
    const fn to_parameter_origin(self) -> ConfigPathOrigin {
        match self {
            Self::Default => ConfigPathOrigin::Default,
            Self::Override => ConfigPathOrigin::Override,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationConfigSourceDescriptor {
    pub origin: NavigationConfigSourceOrigin,
    pub path: PathBuf,
}

impl NavigationConfigSourceDescriptor {
    pub fn new(origin: NavigationConfigSourceOrigin, path: PathBuf) -> Self {
        Self { origin, path }
    }

    pub fn default(path: PathBuf) -> Self {
        Self::new(NavigationConfigSourceOrigin::Default, path)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationConfigSourceDescriptors {
    pub project: NavigationConfigSourceDescriptor,
    pub user: NavigationConfigSourceDescriptor,
}

#[derive(Clone, Debug, PartialEq)]
struct NavigationConfigSource {
    pub level: &'static str,
    pub path: String,
    pub loaded: LoadedParameterConfigSource,
}

#[derive(Clone, Debug, PartialEq)]
struct NavigationConfigSources {
    pub project: NavigationConfigSource,
    pub user: NavigationConfigSource,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationCommandOutcome {
    pub response: ProtocolResponse,
    pub output: NavigationOutputMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationContextOutcome {
    pub selection: NavigationContextSelection,
    pub defaults: NavigationContextDefaults,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationContextDefaults {
    pub adapter: NavigationResolvedValue,
    pub pagination: Option<NavigationPaginationDefaults>,
    pub output: NavigationResolvedValue,
    pub page: Option<NavigationResolvedValue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationPaginationDefaults {
    pub enabled: NavigationResolvedValue,
    pub limit: NavigationResolvedValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationResolvedValue {
    pub value: Value,
    pub source: String,
}

impl NavigationResolvedValue {
    pub fn new(value: Value, source: impl Into<String>) -> Self {
        Self {
            value,
            source: source.into(),
        }
    }
}

pub fn execute_navigation_command<R>(
    command: NavigationCommand,
    config_sources: NavigationConfigSourceDescriptors,
    registry: &R,
) -> Result<NavigationCommandOutcome, NavigationError>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    execute_loaded_navigation_command(
        command,
        load_navigation_config_sources(config_sources),
        registry,
    )
}

fn execute_loaded_navigation_command<R>(
    command: NavigationCommand,
    config_sources: NavigationConfigSources,
    registry: &R,
) -> Result<NavigationCommandOutcome, NavigationError>
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
    let resolved = resolve_operation_input(
        &command,
        &config_sources,
        selection.adapter.id,
        selection.adapter.adapter,
    )?;
    let request = protocol_request(OperationInput {
        operation: command.operation,
        document_path: resolved.document_path,
        ref_id: resolved.ref_id,
        query: resolved.query,
        page: resolved.page,
        limit: resolved.limit,
        options: resolved.options,
    })
    .map_err(|error| NavigationError::invalid_request(error.field(), error.reason()))?;

    let response = execute_navigation_request(&config_sources, selection.adapter, &request)?;

    Ok(NavigationCommandOutcome {
        response,
        output: resolved.output,
    })
}

fn execute_navigation_request(
    config_sources: &NavigationConfigSources,
    adapter: NavigationAdapterRef<'_>,
    request: &RequestEnvelope,
) -> Result<ProtocolResponse, NavigationError> {
    if request.operation == Operation::Outline {
        if let OutlineMode::UnstructuredFull(unstructured) =
            resolve_outline_mode(config_sources, adapter.id, adapter.adapter, request)?
        {
            return Ok(execute_unstructured_outline(
                adapter.adapter,
                request,
                unstructured,
            ));
        }
    }

    Ok(execute_protocol_request(adapter.adapter, request))
}

pub fn resolve_navigation_context<R>(
    command: NavigationCommand,
    config_sources: NavigationConfigSourceDescriptors,
    registry: &R,
) -> Result<NavigationContextOutcome, NavigationError>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    resolve_loaded_navigation_context(
        command,
        load_navigation_config_sources(config_sources),
        registry,
    )
}

fn resolve_loaded_navigation_context<R>(
    command: NavigationCommand,
    config_sources: NavigationConfigSources,
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
    let defaults = resolve_context_defaults(
        &command,
        &config_sources,
        selection.adapter.id,
        selection.adapter.adapter,
    )?;
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
            "project",
            ConfigSourceLevel::Project,
            descriptors.project,
        ),
        user: load_navigation_config_source("user", ConfigSourceLevel::User, descriptors.user),
    }
}

fn load_navigation_config_source(
    level: &'static str,
    source_level: ConfigSourceLevel,
    descriptor: NavigationConfigSourceDescriptor,
) -> NavigationConfigSource {
    let parameter_descriptor = ParameterConfigSourceDescriptor::new(
        source_level,
        descriptor.origin.to_parameter_origin(),
        descriptor.path.clone(),
    );
    NavigationConfigSource {
        level,
        path: descriptor.path.display().to_string(),
        loaded: load_parameter_config_source(&parameter_descriptor),
    }
}

#[cfg(test)]
mod tests;
