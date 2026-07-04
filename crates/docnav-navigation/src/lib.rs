mod context;
mod error;
mod parameters;
mod routing;

use std::fmt;
use std::path::PathBuf;

use docnav_adapter_contracts::{Adapter, AdapterResult};
use docnav_parameter_resolution::{
    load_parameter_config_source, ConfigPathOrigin, ConfigSourceLevel, LoadedParameterConfigSource,
    ParameterConfigSourceDescriptor,
};
use docnav_protocol::{
    generate_request_id, Document, FindArguments, InfoArguments, Operation, OperationArguments,
    OperationResult, Options, OutlineArguments, PositiveInteger, ProtocolResponse, ReadArguments,
    RequestEnvelope, PROTOCOL_VERSION,
};
use serde_json::Value;

pub use context::{select_navigation_context, NavigationContextSelection};
pub use error::NavigationError;
use parameters::{resolve_adapter_intent, resolve_context_defaults, resolve_operation_input};
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationInput {
    pub operation: Operation,
    pub document_path: String,
    pub ref_id: Option<String>,
    pub query: Option<String>,
    pub page: Option<PositiveInteger>,
    pub limit: Option<PositiveInteger>,
    pub options: Option<Options>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationInputError {
    field: &'static str,
    operation: Operation,
    argument: &'static str,
}

impl NavigationInputError {
    pub const fn field(&self) -> &'static str {
        self.field
    }

    pub fn reason(&self) -> String {
        format!("{} requires {}", self.operation, self.argument)
    }
}

impl fmt::Display for NavigationInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.reason())
    }
}

impl std::error::Error for NavigationInputError {}

// @case WB-NAVIGATION-DISPATCH-001
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
    let response = execute_protocol_request(selection.adapter.adapter, &request);

    Ok(NavigationCommandOutcome {
        response,
        output: resolved.output,
    })
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

pub fn protocol_request(input: OperationInput) -> Result<RequestEnvelope, NavigationInputError> {
    let arguments = operation_arguments(&input)?;

    Ok(RequestEnvelope {
        protocol_version: PROTOCOL_VERSION.to_owned(),
        request_id: generate_request_id(),
        operation: input.operation,
        document: Document {
            path: input.document_path,
        },
        arguments,
    })
}

pub fn execute_protocol_request(
    adapter: &dyn Adapter,
    request: &RequestEnvelope,
) -> ProtocolResponse {
    match execute_operation(adapter, request) {
        Ok(result) => ProtocolResponse::success(
            request.protocol_version.clone(),
            request.request_id.clone(),
            result,
        ),
        Err(error) => ProtocolResponse::failure_for_request(request, error.protocol_error()),
    }
}

pub fn execute_operation(
    adapter: &dyn Adapter,
    request: &RequestEnvelope,
) -> AdapterResult<OperationResult> {
    match (&request.operation, &request.arguments) {
        (Operation::Outline, OperationArguments::Outline(arguments)) => adapter
            .outline(request, arguments)
            .map(OperationResult::Outline),
        (Operation::Read, OperationArguments::Read(arguments)) => {
            adapter.read(request, arguments).map(OperationResult::Read)
        }
        (Operation::Find, OperationArguments::Find(arguments)) => {
            adapter.find(request, arguments).map(OperationResult::Find)
        }
        (Operation::Info, OperationArguments::Info(arguments)) => {
            adapter.info(request, arguments).map(OperationResult::Info)
        }
        _ => Err(docnav_adapter_contracts::AdapterError::invalid_request(
            "arguments",
            format!("arguments do not match operation {}", request.operation),
        )),
    }
}

fn operation_arguments(input: &OperationInput) -> Result<OperationArguments, NavigationInputError> {
    match input.operation {
        Operation::Outline => Ok(OperationArguments::Outline(OutlineArguments {
            limit: required_limit(input, "limit")?,
            page: required_page(input, "page")?,
            options: input.options.clone(),
        })),
        Operation::Read => Ok(OperationArguments::Read(ReadArguments {
            ref_id: required_ref_id(input)?,
            limit: required_limit(input, "limit")?,
            page: required_page(input, "page")?,
            options: input.options.clone(),
        })),
        Operation::Find => Ok(OperationArguments::Find(FindArguments {
            query: required_query(input)?,
            limit: required_limit(input, "limit")?,
            page: required_page(input, "page")?,
            options: input.options.clone(),
        })),
        Operation::Info => Ok(OperationArguments::Info(InfoArguments {
            options: input.options.clone(),
        })),
    }
}

fn required_limit(
    input: &OperationInput,
    argument: &'static str,
) -> Result<PositiveInteger, NavigationInputError> {
    input
        .limit
        .ok_or_else(|| missing_argument(input, "limit", argument))
}

fn required_page(
    input: &OperationInput,
    argument: &'static str,
) -> Result<PositiveInteger, NavigationInputError> {
    input
        .page
        .ok_or_else(|| missing_argument(input, "page", argument))
}

fn required_ref_id(input: &OperationInput) -> Result<String, NavigationInputError> {
    input
        .ref_id
        .clone()
        .ok_or_else(|| missing_argument(input, "ref", "ref"))
}

fn required_query(input: &OperationInput) -> Result<String, NavigationInputError> {
    input
        .query
        .clone()
        .ok_or_else(|| missing_argument(input, "query", "query"))
}

fn missing_argument(
    input: &OperationInput,
    field: &'static str,
    argument: &'static str,
) -> NavigationInputError {
    NavigationInputError {
        field,
        operation: input.operation,
        argument,
    }
}

#[cfg(test)]
mod tests;
