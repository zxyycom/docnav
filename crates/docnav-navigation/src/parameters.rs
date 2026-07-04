mod config;
mod fields;
mod input;
mod native_options;
mod values;

use docnav_adapter_contracts::{Adapter, AdapterOptionSpec};
use docnav_parameter_resolution::{
    ids, EntryPassthroughPolicy, ParameterResolution, ParameterResolutionPipeline,
};
use docnav_protocol::{Operation, Options, PositiveInteger};
use docnav_typed_fields::FieldStringEnum;

use crate::{
    NavigationCommand, NavigationConfigSources, NavigationContextDefaults, NavigationError,
    NavigationOutputMode, NavigationPaginationDefaults, NavigationResolvedValue,
};

const DIRECT_PROCESSING: &str = "cli";
const CONFIG_PROCESSING: &str = "config";
const DEFAULT_LIMIT: i64 = 6000;
const DEFAULT_PAGE: i64 = 1;
const DEFAULT_PAGINATION_ENABLED: bool = true;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterIntent {
    pub adapter_id: Option<String>,
    pub source: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedNavigationInput {
    pub document_path: String,
    pub ref_id: Option<String>,
    pub query: Option<String>,
    pub page: Option<PositiveInteger>,
    pub limit: Option<PositiveInteger>,
    pub output: NavigationOutputMode,
    pub options: Option<Options>,
}

impl FieldStringEnum for NavigationOutputMode {
    fn variants() -> &'static [Self] {
        &[Self::ReadableView, Self::ReadableJson, Self::ProtocolJson]
    }

    fn as_str(&self) -> &'static str {
        NavigationOutputMode::as_str(*self)
    }
}

pub fn resolve_adapter_intent(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
) -> Result<AdapterIntent, NavigationError> {
    let fields = fields::adapter_intent_fields()?;
    let resolution = resolve_with_fields(&fields, command, config_sources, &[], "adapter-intent")?;

    config::first_resolution_error(&resolution)?;
    Ok(AdapterIntent {
        adapter_id: values::optional_string_value(&resolution, ids::ADAPTER)?,
        source: values::resolved_source_label(&resolution, ids::ADAPTER).unwrap_or("built_in"),
    })
}

pub fn resolve_operation_input(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selected_adapter_id: &str,
    selected_adapter: &dyn Adapter,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let operation_fields = fields::operation_fields(command.operation, selected_adapter)?;
    let selected_native_options = operation_fields.adapter_options();
    config::validate_navigation_sources(
        command,
        config_sources,
        selected_adapter_id,
        operation_fields.as_ref(),
        selected_native_options,
    )?;

    let resolution = resolve_with_fields(
        operation_fields.as_ref(),
        command,
        config_sources,
        selected_native_options,
        "operation-input",
    )?;

    config::first_operation_resolution_error(
        &resolution,
        command,
        config_sources,
        selected_native_options,
    )?;

    resolved_input_from_resolution(command.operation, &resolution, selected_native_options)
}

pub fn resolve_context_defaults(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selected_adapter_id: &str,
    selected_adapter: &dyn Adapter,
) -> Result<NavigationContextDefaults, NavigationError> {
    let operation_fields = fields::operation_fields(command.operation, selected_adapter)?;
    let selected_native_options = operation_fields.adapter_options();
    config::validate_navigation_sources(
        command,
        config_sources,
        selected_adapter_id,
        operation_fields.as_ref(),
        selected_native_options,
    )?;

    let resolution = resolve_with_fields(
        operation_fields.as_ref(),
        command,
        config_sources,
        selected_native_options,
        "context-defaults",
    )?;

    config::first_operation_resolution_error(
        &resolution,
        command,
        config_sources,
        selected_native_options,
    )?;

    defaults_from_resolution(command.operation, &resolution)
}

fn resolved_input_from_resolution(
    operation: Operation,
    resolution: &ParameterResolution,
    selected_native_options: &[AdapterOptionSpec],
) -> Result<ResolvedNavigationInput, NavigationError> {
    let options = native_options::resolved_options(resolution, selected_native_options)?;
    Ok(ResolvedNavigationInput {
        document_path: values::required_string_value(resolution, ids::PATH)?,
        ref_id: values::optional_string_value(resolution, ids::REF)?,
        query: values::optional_string_value(resolution, ids::QUERY)?,
        page: values::optional_document_positive(operation, resolution, ids::PAGE)?,
        limit: values::optional_document_limit(operation, resolution)?,
        output: values::required_output_value(resolution)?,
        options: (!options.is_empty()).then_some(options),
    })
}

fn defaults_from_resolution(
    operation: Operation,
    resolution: &ParameterResolution,
) -> Result<NavigationContextDefaults, NavigationError> {
    Ok(NavigationContextDefaults {
        adapter: resolved_value(resolution, ids::ADAPTER)
            .unwrap_or_else(|| NavigationResolvedValue::new(serde_json::Value::Null, "unset")),
        pagination: if values::uses_document_window(operation) {
            Some(NavigationPaginationDefaults {
                enabled: required_resolved_value(resolution, ids::PAGINATION_ENABLED)?,
                limit: required_resolved_value(resolution, ids::LIMIT)?,
            })
        } else {
            None
        },
        output: required_resolved_value(resolution, ids::OUTPUT)?,
        page: if values::uses_document_window(operation) {
            Some(required_resolved_value(resolution, ids::PAGE)?)
        } else {
            None
        },
    })
}

fn required_resolved_value(
    resolution: &ParameterResolution,
    identity: &str,
) -> Result<NavigationResolvedValue, NavigationError> {
    resolved_value(resolution, identity).ok_or_else(|| {
        NavigationError::internal(format!("missing-resolved-navigation-parameter:{identity}"))
    })
}

fn resolved_value(
    resolution: &ParameterResolution,
    identity: &str,
) -> Option<NavigationResolvedValue> {
    let value = resolution.value(&values::identity_key(identity).ok()?)?;
    Some(NavigationResolvedValue::new(
        values::typed_value_to_json(&value.value),
        values::source_label(value.source.kind),
    ))
}

fn resolve_with_fields(
    fields: &docnav_typed_fields::FieldDefSet,
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selected_native_options: &[AdapterOptionSpec],
    context: &str,
) -> Result<ParameterResolution, NavigationError> {
    ParameterResolutionPipeline::new(fields)
        .with_direct_input_processing_id(DIRECT_PROCESSING)
        .with_config_processing_id(CONFIG_PROCESSING)
        .with_loaded_project_config(config_sources.project.loaded.clone())
        .with_loaded_user_config(config_sources.user.loaded.clone())
        .with_passthrough_policy(EntryPassthroughPolicy::Discard)
        .resolve(input::direct_input(command, selected_native_options))
        .map_err(|error| NavigationError::internal(format!("{context}:{error}")))
}
