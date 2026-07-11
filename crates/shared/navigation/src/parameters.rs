mod config;
mod fields;
mod input;
mod inspection;
mod native_options;
mod resolution;
mod values;

use cli_config_resolution::{FieldDefSet, ResolutionResult};
use docnav_adapter_contracts::{AdapterDefinition, AdapterOptionSpec, NativeOptionHandoff};
use docnav_protocol::{Operation, Options, PositiveInteger};
use docnav_typed_fields::FieldStringEnum;

use crate::{
    NavigationAdapterRegistry, NavigationCommand, NavigationConfigSource, NavigationConfigSources,
    NavigationContextDefaults, NavigationError, NavigationOutputMode, NavigationPaginationDefaults,
    NavigationResolvedValue,
};

pub(crate) use inspection::inspect_config_sources;

pub(super) mod ids {
    pub(super) const ADAPTER: &str = "docnav.defaults.adapter";
    pub(super) const INVOCATION_LOG_CONTENT_CAPTURE_ENABLED: &str =
        "docnav.invocation_log.content_capture.enabled";
    pub(super) const INVOCATION_LOG_CONTENT_CAPTURE_ROOT: &str =
        "docnav.invocation_log.content_capture.root";
    pub(super) const INVOCATION_LOG_ENABLED: &str = "docnav.invocation_log.enabled";
    pub(super) const INVOCATION_LOG_PATH: &str = "docnav.invocation_log.path";
    pub(super) const LIMIT: &str = "docnav.defaults.pagination.limit";
    pub(super) const OUTPUT: &str = "docnav.defaults.output";
    pub(super) const PAGE: &str = "docnav.document.page";
    pub(super) const PAGINATION_ENABLED: &str = "docnav.defaults.pagination.enabled";
    pub(super) const PATH: &str = "docnav.document.path";
    pub(super) const QUERY: &str = "docnav.document.query";
    pub(super) const REF: &str = "docnav.document.ref";
}

const DIRECT_PROCESSING: &str = "direct";
const CONFIG_PROCESSING: &str = "config";
const DEFAULT_LIMIT: i64 = 6000;
const DEFAULT_PAGE: i64 = 1;
const DEFAULT_PAGINATION_ENABLED: bool = true;
const MAX_PAGINATION_LIMIT: u32 = u32::MAX;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterIntent {
    pub adapter_id: Option<String>,
    pub source: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedNavigationInput {
    pub document_path: String,
    pub ref_id: Option<String>,
    pub query: Option<String>,
    pub page: Option<PositiveInteger>,
    pub limit: Option<PositiveInteger>,
    pub output: NavigationOutputMode,
    pub options: Option<Options>,
    pub native_options: NativeOptionHandoff,
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
    let resolution =
        resolve_command_with_fields(&fields, command, config_sources, &[], "adapter-intent")?;

    config::first_resolution_error(&resolution, config_sources)?;
    Ok(AdapterIntent {
        adapter_id: values::optional_string_value(&fields, &resolution, ids::ADAPTER)?,
        source: values::resolved_source_label(&resolution, ids::ADAPTER).unwrap_or("built_in"),
    })
}

pub fn resolve_operation_input(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selected_adapter_id: &str,
    selected_adapter: &AdapterDefinition<'_>,
    registry: &(impl NavigationAdapterRegistry + ?Sized),
) -> Result<ResolvedNavigationInput, NavigationError> {
    let operation_fields = fields::operation_fields(command.operation, selected_adapter)?;
    let selected_native_options = operation_fields.adapter_options();
    config::validate_navigation_sources(
        command,
        config_sources,
        selected_adapter_id,
        &operation_fields,
        registry,
    )?;

    let resolution = resolve_command_with_fields(
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

    resolved_input_from_resolution(
        command.operation,
        operation_fields.as_ref(),
        &resolution,
        selected_native_options,
    )
}

pub fn resolve_context_defaults(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selected_adapter_id: &str,
    selected_adapter: &AdapterDefinition<'_>,
    registry: &(impl NavigationAdapterRegistry + ?Sized),
) -> Result<NavigationContextDefaults, NavigationError> {
    let operation_fields = fields::operation_fields(command.operation, selected_adapter)?;
    let selected_native_options = operation_fields.adapter_options();
    config::validate_navigation_sources(
        command,
        config_sources,
        selected_adapter_id,
        &operation_fields,
        registry,
    )?;

    let resolution = resolve_command_with_fields(
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

    defaults_from_resolution(command.operation, operation_fields.as_ref(), &resolution)
}

pub(crate) fn validate_config_source_for_registry(
    source: &NavigationConfigSource,
    registry: &(impl NavigationAdapterRegistry + ?Sized),
) -> Result<(), NavigationError> {
    config::validate_config_source_for_registry(source, registry)
}

fn resolved_input_from_resolution(
    operation: Operation,
    fields: &FieldDefSet,
    resolution: &ResolutionResult,
    selected_native_options: &[AdapterOptionSpec],
) -> Result<ResolvedNavigationInput, NavigationError> {
    let options = native_options::resolved_options(fields, resolution, selected_native_options)?;
    let native_option_handoff =
        NativeOptionHandoff::from_options((!options.is_empty()).then_some(&options));
    Ok(ResolvedNavigationInput {
        document_path: values::required_string_value(fields, resolution, ids::PATH)?,
        ref_id: values::optional_string_value(fields, resolution, ids::REF)?,
        query: values::optional_string_value(fields, resolution, ids::QUERY)?,
        page: values::optional_document_positive(operation, fields, resolution, ids::PAGE)?,
        limit: values::optional_document_limit(operation, fields, resolution)?,
        output: values::required_output_value(fields, resolution)?,
        options: (!options.is_empty()).then_some(options),
        native_options: native_option_handoff,
    })
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

fn resolve_with_fields(
    fields: &docnav_typed_fields::FieldDefSet,
    direct_input: Option<input::DirectInput>,
    config_sources: &NavigationConfigSources,
    context: &str,
) -> Result<ResolutionResult, NavigationError> {
    resolution::resolve(fields, direct_input.as_ref(), config_sources)
        .map_err(|_| NavigationError::internal(resolution_pipeline_error_id(context)))
}

fn resolve_command_with_fields(
    fields: &docnav_typed_fields::FieldDefSet,
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selected_native_options: &[AdapterOptionSpec],
    context: &str,
) -> Result<ResolutionResult, NavigationError> {
    resolve_with_fields(
        fields,
        Some(input::direct_input(command, selected_native_options)),
        config_sources,
        context,
    )
}

fn resolution_pipeline_error_id(context: &str) -> &'static str {
    match context {
        "adapter-intent" => "adapter-intent-resolution-failed",
        "operation-input" => "operation-input-resolution-failed",
        "context-defaults" => "context-defaults-resolution-failed",
        _ => "parameter-resolution-failed",
    }
}
