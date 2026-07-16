mod config_source;
mod context;
mod error;
mod outline_mode;
mod parameters;
mod protocol;
mod routing;

use std::path::PathBuf;

use cli_config_resolution::Source;
use config_source::{load_config_source, LoadedNavigationConfigSource};
use docnav_protocol::{Operation, ProtocolResponse, RequestEnvelope};
use serde_json::Value;

pub use context::{select_navigation_context, NavigationContextSelection};
pub use error::NavigationError;
use outline_mode::{execute_unstructured_outline, resolve_outline_mode, OutlineMode};
use parameters::{
    resolve_adapter_intent, resolve_context_defaults, resolve_operation_input, AdapterIntent,
    ResolvedNavigationInput,
};
pub use parameters::{
    DocumentCliFieldAttribution, DocumentCliFieldOwner, DocumentCliFieldSet,
    DocumentCliFieldSetBuildError,
};
pub use protocol::{
    execute_operation, execute_protocol_request, protocol_request, NavigationInputError,
    OperationInput,
};
use routing::AdapterSelection;
pub use routing::{
    select_adapter, AdapterSelectionRequest, CandidateEvidence, NavigationAdapterRef,
    NavigationAdapterRegistry,
};

/// Builds the canonical CLI field set for a document operation from registered declarations.
///
/// Declaration conflicts retain their field and owner attribution in the returned build error.
pub fn document_cli_field_set(
    operation: Operation,
    registry: &(impl NavigationAdapterRegistry + ?Sized),
) -> Result<DocumentCliFieldSet, DocumentCliFieldSetBuildError> {
    parameters::registry_cli_fields(operation, registry)
}

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

/// Canonical identity used for invocation-local document CLI candidates.
pub const DOCUMENT_CLI_SOURCE_ID: &str = "explicit";
/// Source priority for invocation-local document CLI candidates.
pub const DOCUMENT_CLI_SOURCE_PRIORITY: i32 = 400;

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationCommand {
    pub operation: Operation,
    pub document_path: String,
    pub ref_id: Option<String>,
    pub query: Option<String>,
    pub cli_source: Source,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationConfigSourceOrigin {
    Default,
    ExplicitCli,
    Override,
}

impl NavigationConfigSourceOrigin {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::ExplicitCli => "explicit_cli",
            Self::Override => "override",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationConfigSourceLevel {
    Project,
    User,
}

impl NavigationConfigSourceLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::User => "user",
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

    pub fn explicit_cli(path: PathBuf) -> Self {
        Self::new(NavigationConfigSourceOrigin::ExplicitCli, path)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationConfigSourceDescriptors {
    pub project: NavigationConfigSourceDescriptor,
    pub user: NavigationConfigSourceDescriptor,
}

#[derive(Clone, Debug, PartialEq)]
struct NavigationConfigSource {
    pub level: NavigationConfigSourceLevel,
    pub origin: NavigationConfigSourceOrigin,
    pub path: String,
    pub loaded: LoadedNavigationConfigSource,
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
    pub trace: NavigationInvocationTrace,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationInvocationTrace {
    pub operation: Operation,
    pub selected_adapter_id: Option<String>,
    pub request_id: Option<String>,
    pub failure_layer: Option<NavigationFailureLayer>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationFailureLayer {
    Config,
    AdapterSelection,
    RequestConstruction,
    AdapterDispatch,
    ResultValidation,
}

impl NavigationFailureLayer {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::AdapterSelection => "adapter_selection",
            Self::RequestConstruction => "request_construction",
            Self::AdapterDispatch => "adapter_dispatch",
            Self::ResultValidation => "result_validation",
        }
    }
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

pub fn inspect_navigation_config_sources<R>(
    config_sources: NavigationConfigSourceDescriptors,
    registry: &R,
) -> Result<Value, NavigationError>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    parameters::inspect_config_sources(&load_navigation_config_sources(config_sources), registry)
}

pub fn validate_navigation_config_source_value<R>(
    level: NavigationConfigSourceLevel,
    origin: NavigationConfigSourceOrigin,
    path: impl Into<String>,
    value: Value,
    registry: &R,
) -> Result<(), NavigationError>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    let source = NavigationConfigSource {
        level,
        origin,
        path: path.into(),
        loaded: LoadedNavigationConfigSource::from_value(value),
    };
    parameters::validate_config_source_for_registry(&source, registry)
}

fn execute_loaded_navigation_command<R>(
    command: NavigationCommand,
    config_sources: NavigationConfigSources,
    registry: &R,
) -> Result<NavigationCommandOutcome, NavigationError>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    let mut trace = navigation_trace(command.operation);
    let adapter_intent = resolve_navigation_adapter_intent(&command, &config_sources, &mut trace)?;
    let selection = select_navigation_adapter(&command, &adapter_intent, registry, &mut trace)?;
    let resolved =
        resolve_navigation_input(&command, &config_sources, &selection, registry, &mut trace)?;
    let prepared = prepare_navigation_request(command.operation, resolved, &mut trace)?;
    let response = dispatch_navigation_request(&config_sources, &selection, &prepared, &mut trace)?;
    let response = validate_navigation_response(response, &mut trace)?;

    Ok(NavigationCommandOutcome {
        response,
        output: prepared.output,
        trace,
    })
}

struct PreparedNavigationRequest {
    request: RequestEnvelope,
    output: NavigationOutputMode,
    native_options: docnav_adapter_contracts::NativeOptionHandoff,
}

fn navigation_trace(operation: Operation) -> NavigationInvocationTrace {
    NavigationInvocationTrace {
        operation,
        selected_adapter_id: None,
        request_id: None,
        failure_layer: None,
    }
}

fn resolve_navigation_adapter_intent(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    trace: &mut NavigationInvocationTrace,
) -> Result<AdapterIntent, NavigationError> {
    resolve_adapter_intent(command, config_sources)
        .map_err(|error| error_with_trace(trace, NavigationFailureLayer::Config, error))
}

fn select_navigation_adapter<'a, R>(
    command: &'a NavigationCommand,
    adapter_intent: &'a AdapterIntent,
    registry: &'a R,
    trace: &mut NavigationInvocationTrace,
) -> Result<AdapterSelection<'a>, NavigationError>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    let selection = select_adapter(AdapterSelectionRequest {
        registry,
        document_path: &command.document_path,
        preselected_adapter_id: adapter_intent.adapter_id.as_deref(),
        preselected_adapter_source: adapter_intent.source,
    })
    .map_err(|error| error_with_trace(trace, NavigationFailureLayer::AdapterSelection, error))?;
    trace.selected_adapter_id = Some(selection.adapter.id().to_owned());
    Ok(selection)
}

fn resolve_navigation_input<R>(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selection: &AdapterSelection<'_>,
    registry: &R,
    trace: &mut NavigationInvocationTrace,
) -> Result<ResolvedNavigationInput, NavigationError>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    resolve_operation_input(
        command,
        config_sources,
        selection.adapter.id(),
        &selection.adapter.definition,
        registry,
    )
    .map_err(|error| error_with_trace(trace, NavigationFailureLayer::RequestConstruction, error))
}

fn prepare_navigation_request(
    operation: Operation,
    resolved: ResolvedNavigationInput,
    trace: &mut NavigationInvocationTrace,
) -> Result<PreparedNavigationRequest, NavigationError> {
    let request = protocol_request(OperationInput {
        operation,
        document_path: resolved.document_path,
        ref_id: resolved.ref_id,
        query: resolved.query,
        page: resolved.page,
        limit: resolved.limit,
        options: resolved.options,
    })
    .map_err(|error| input_error_with_trace(trace, error))?;
    trace.request_id = Some(request.request_id.clone());

    Ok(PreparedNavigationRequest {
        request,
        output: resolved.output,
        native_options: resolved.native_options,
    })
}

fn dispatch_navigation_request(
    config_sources: &NavigationConfigSources,
    selection: &AdapterSelection<'_>,
    prepared: &PreparedNavigationRequest,
    trace: &mut NavigationInvocationTrace,
) -> Result<ProtocolResponse, NavigationError> {
    let response = execute_navigation_request(
        config_sources,
        &selection.adapter,
        &prepared.request,
        &prepared.native_options,
    )
    .map_err(|error| error_with_trace(trace, NavigationFailureLayer::AdapterDispatch, error))?;
    if matches!(response, ProtocolResponse::Failure(_)) {
        trace.failure_layer = Some(NavigationFailureLayer::AdapterDispatch);
    }
    Ok(response)
}

fn input_error_with_trace(
    trace: &mut NavigationInvocationTrace,
    error: NavigationInputError,
) -> NavigationError {
    trace.failure_layer = Some(NavigationFailureLayer::RequestConstruction);
    NavigationError::invalid_request(error.field(), error.reason()).with_invocation_trace(trace)
}

fn error_with_trace(
    trace: &mut NavigationInvocationTrace,
    layer: NavigationFailureLayer,
    error: NavigationError,
) -> NavigationError {
    trace.failure_layer = Some(layer);
    error.with_invocation_trace(trace)
}

fn validate_navigation_response(
    response: ProtocolResponse,
    trace: &mut NavigationInvocationTrace,
) -> Result<ProtocolResponse, NavigationError> {
    response.validate().map_err(|error| {
        trace.failure_layer = Some(NavigationFailureLayer::ResultValidation);
        NavigationError::protocol_response_invalid(error.to_string()).with_invocation_trace(trace)
    })?;
    Ok(response)
}

fn execute_navigation_request(
    config_sources: &NavigationConfigSources,
    adapter: &NavigationAdapterRef<'_>,
    request: &RequestEnvelope,
    native_options: &docnav_adapter_contracts::NativeOptionHandoff,
) -> Result<ProtocolResponse, NavigationError> {
    if request.operation == Operation::Outline {
        if let OutlineMode::UnstructuredFull(unstructured) =
            resolve_outline_mode(config_sources, adapter.id(), &adapter.definition, request)?
        {
            return Ok(execute_unstructured_outline(
                &adapter.definition,
                request,
                unstructured,
            ));
        }
    }

    Ok(execute_protocol_request(
        &adapter.definition,
        request,
        native_options,
    ))
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
        selection.adapter.id(),
        &selection.adapter.definition,
        registry,
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
