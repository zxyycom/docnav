mod catalog;
mod config;
mod fields;
mod input;
mod inspection;
mod native_options;
mod resolution;
mod values;

use cli_config_resolution::{FieldDefSet, ResolutionResult, Source};
use docnav_adapter_contracts::{
    FindInput, InfoInput, OutlineInput, ReadInput, StandardInputBinding, StandardOperationInput,
};
use docnav_protocol::{Operation, Options, PagedOperation, PositiveInteger};
use docnav_typed_fields::FieldStringEnum;

use crate::{
    NavigationCommand, NavigationConfigSource, NavigationConfigSources, NavigationContextDefaults,
    NavigationError, NavigationOutputMode, NavigationPaginationDefaults, NavigationResolvedValue,
};

pub use catalog::{
    DocumentParameterBinding, DocumentParameterCatalog, DocumentParameterCatalogBuildError,
    DocumentParameterEntry,
};
pub(crate) use fields::adapter_routing_fields;
pub use fields::{DocumentCliFieldAttribution, DocumentCliFieldSet, DocumentCliFieldSetBuildError};
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
    pub standard_input: StandardOperationInput,
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
    let cli_source = input::cli_source_for_fields(&command.cli_source, &fields)
        .map_err(|_| NavigationError::internal("adapter-intent-cli-source-invalid"))?;
    let resolution = resolve_with_fields(
        &fields,
        Some(input::direct_input(command)),
        Some(&cli_source),
        config_sources,
        "adapter-intent",
    )?;

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
    catalog: &DocumentParameterCatalog,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let operation_fields =
        fields::operation_fields(command.operation, selected_adapter_id, catalog)?;
    config::validate_navigation_sources(
        command,
        config_sources,
        selected_adapter_id,
        &operation_fields,
        catalog,
    )?;

    let resolution = resolve_command_with_fields(
        operation_fields.as_ref(),
        command,
        config_sources,
        "operation-input",
    )?;

    config::first_operation_resolution_error(
        &resolution,
        config_sources,
        selected_adapter_id,
        command.operation,
        catalog,
    )?;

    resolved_input_from_resolution(
        command.operation,
        selected_adapter_id,
        catalog,
        operation_fields.as_ref(),
        &resolution,
    )
}

pub fn resolve_context_defaults(
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

    let resolution = resolve_command_with_fields(
        operation_fields.as_ref(),
        command,
        config_sources,
        "context-defaults",
    )?;

    config::first_operation_resolution_error(
        &resolution,
        config_sources,
        selected_adapter_id,
        command.operation,
        catalog,
    )?;

    defaults_from_resolution(command.operation, operation_fields.as_ref(), &resolution)
}

pub(crate) fn validate_config_source_for_catalog(
    source: &NavigationConfigSource,
    catalog: &DocumentParameterCatalog,
) -> Result<(), NavigationError> {
    config::validate_config_source_for_catalog(source, catalog)
}

fn resolved_input_from_resolution(
    operation: Operation,
    selected_adapter_id: &str,
    catalog: &DocumentParameterCatalog,
    fields: &FieldDefSet,
    resolution: &ResolutionResult,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let output = required_output_binding(
        selected_adapter_id,
        catalog,
        fields,
        resolution,
        DocumentParameterBinding::OutputMode(operation),
    )?;

    match operation {
        Operation::Outline => {
            resolved_outline_input(selected_adapter_id, catalog, fields, resolution, output)
        }
        Operation::Read => {
            resolved_read_input(selected_adapter_id, catalog, fields, resolution, output)
        }
        Operation::Find => {
            resolved_find_input(selected_adapter_id, catalog, fields, resolution, output)
        }
        Operation::Info => resolved_info_input(fields, resolution, output),
    }
}

fn resolved_outline_input(
    selected_adapter_id: &str,
    catalog: &DocumentParameterCatalog,
    fields: &FieldDefSet,
    resolution: &ResolutionResult,
    output: NavigationOutputMode,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let document_path = values::required_string_value(fields, resolution, ids::PATH)?;
    let page = required_positive_binding(
        selected_adapter_id,
        catalog,
        fields,
        resolution,
        DocumentParameterBinding::StandardInput(StandardInputBinding::OutlinePage),
    )?;
    let raw_limit = required_positive_binding(
        selected_adapter_id,
        catalog,
        fields,
        resolution,
        DocumentParameterBinding::StandardInput(StandardInputBinding::OutlineLimit),
    )?;
    let pagination_enabled = required_bool_binding(
        selected_adapter_id,
        catalog,
        fields,
        resolution,
        DocumentParameterBinding::PaginationEnabled(PagedOperation::Outline),
    )?;
    let limit = effective_limit(raw_limit, pagination_enabled);
    let max_heading_binding = StandardInputBinding::OutlineMaxHeadingLevel;
    let max_heading_level = optional_adapter_integer_binding(
        selected_adapter_id,
        catalog,
        fields,
        resolution,
        max_heading_binding,
    )?;
    let options = protocol_options(max_heading_binding, max_heading_level);
    let standard_input = StandardOperationInput::Outline(OutlineInput {
        document_path: document_path.clone(),
        page,
        limit,
        max_heading_level,
    });

    Ok(ResolvedNavigationInput {
        document_path,
        ref_id: None,
        query: None,
        page: Some(page),
        limit: Some(limit),
        output,
        options,
        standard_input,
    })
}

fn resolved_read_input(
    selected_adapter_id: &str,
    catalog: &DocumentParameterCatalog,
    fields: &FieldDefSet,
    resolution: &ResolutionResult,
    output: NavigationOutputMode,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let document_path = values::required_string_value(fields, resolution, ids::PATH)?;
    let ref_id = values::required_string_value(fields, resolution, ids::REF)?;
    let page = required_positive_binding(
        selected_adapter_id,
        catalog,
        fields,
        resolution,
        DocumentParameterBinding::StandardInput(StandardInputBinding::ReadPage),
    )?;
    let raw_limit = required_positive_binding(
        selected_adapter_id,
        catalog,
        fields,
        resolution,
        DocumentParameterBinding::StandardInput(StandardInputBinding::ReadLimit),
    )?;
    let pagination_enabled = required_bool_binding(
        selected_adapter_id,
        catalog,
        fields,
        resolution,
        DocumentParameterBinding::PaginationEnabled(PagedOperation::Read),
    )?;
    let limit = effective_limit(raw_limit, pagination_enabled);
    let standard_input = StandardOperationInput::Read(ReadInput {
        document_path: document_path.clone(),
        ref_id: ref_id.clone(),
        page,
        limit,
    });

    Ok(ResolvedNavigationInput {
        document_path,
        ref_id: Some(ref_id),
        query: None,
        page: Some(page),
        limit: Some(limit),
        output,
        options: None,
        standard_input,
    })
}

fn resolved_find_input(
    selected_adapter_id: &str,
    catalog: &DocumentParameterCatalog,
    fields: &FieldDefSet,
    resolution: &ResolutionResult,
    output: NavigationOutputMode,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let document_path = values::required_string_value(fields, resolution, ids::PATH)?;
    let query = values::required_string_value(fields, resolution, ids::QUERY)?;
    let page = required_positive_binding(
        selected_adapter_id,
        catalog,
        fields,
        resolution,
        DocumentParameterBinding::StandardInput(StandardInputBinding::FindPage),
    )?;
    let raw_limit = required_positive_binding(
        selected_adapter_id,
        catalog,
        fields,
        resolution,
        DocumentParameterBinding::StandardInput(StandardInputBinding::FindLimit),
    )?;
    let pagination_enabled = required_bool_binding(
        selected_adapter_id,
        catalog,
        fields,
        resolution,
        DocumentParameterBinding::PaginationEnabled(PagedOperation::Find),
    )?;
    let limit = effective_limit(raw_limit, pagination_enabled);
    let max_heading_binding = StandardInputBinding::FindMaxHeadingLevel;
    let max_heading_level = optional_adapter_integer_binding(
        selected_adapter_id,
        catalog,
        fields,
        resolution,
        max_heading_binding,
    )?;
    let options = protocol_options(max_heading_binding, max_heading_level);
    let standard_input = StandardOperationInput::Find(FindInput {
        document_path: document_path.clone(),
        query: query.clone(),
        page,
        limit,
        max_heading_level,
    });

    Ok(ResolvedNavigationInput {
        document_path,
        ref_id: None,
        query: Some(query),
        page: Some(page),
        limit: Some(limit),
        output,
        options,
        standard_input,
    })
}

fn resolved_info_input(
    fields: &FieldDefSet,
    resolution: &ResolutionResult,
    output: NavigationOutputMode,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let document_path = values::required_string_value(fields, resolution, ids::PATH)?;
    let standard_input = StandardOperationInput::Info(InfoInput {
        document_path: document_path.clone(),
    });

    Ok(ResolvedNavigationInput {
        document_path,
        ref_id: None,
        query: None,
        page: None,
        limit: None,
        output,
        options: None,
        standard_input,
    })
}

fn required_positive_binding(
    selected_adapter_id: &str,
    catalog: &DocumentParameterCatalog,
    fields: &FieldDefSet,
    resolution: &ResolutionResult,
    binding: DocumentParameterBinding,
) -> Result<PositiveInteger, NavigationError> {
    let identity = required_binding_identity(selected_adapter_id, catalog, binding)?;
    values::required_positive_value(fields, resolution, identity)
}

fn required_bool_binding(
    selected_adapter_id: &str,
    catalog: &DocumentParameterCatalog,
    fields: &FieldDefSet,
    resolution: &ResolutionResult,
    binding: DocumentParameterBinding,
) -> Result<bool, NavigationError> {
    let identity = required_binding_identity(selected_adapter_id, catalog, binding)?;
    values::required_bool_value(fields, resolution, identity)
}

fn required_output_binding(
    selected_adapter_id: &str,
    catalog: &DocumentParameterCatalog,
    fields: &FieldDefSet,
    resolution: &ResolutionResult,
    binding: DocumentParameterBinding,
) -> Result<NavigationOutputMode, NavigationError> {
    let identity = required_binding_identity(selected_adapter_id, catalog, binding)?;
    values::required_output_value_for_identity(fields, resolution, identity)
}

fn optional_adapter_integer_binding(
    selected_adapter_id: &str,
    catalog: &DocumentParameterCatalog,
    fields: &FieldDefSet,
    resolution: &ResolutionResult,
    binding: StandardInputBinding,
) -> Result<Option<i64>, NavigationError> {
    let Some(identity) = selected_adapter_binding_identity(selected_adapter_id, catalog, binding)
    else {
        return Ok(None);
    };
    values::optional_integer_value(fields, resolution, identity)
}

fn protocol_options(binding: StandardInputBinding, value: Option<i64>) -> Option<Options> {
    let key = protocol_option_key(binding)?;
    value.map(|value| Options::from_iter([(key.to_owned(), value.into())]))
}

fn protocol_option_key(binding: StandardInputBinding) -> Option<&'static str> {
    match binding {
        StandardInputBinding::OutlineMaxHeadingLevel
        | StandardInputBinding::FindMaxHeadingLevel => Some("max_heading_level"),
        StandardInputBinding::OutlinePage
        | StandardInputBinding::OutlineLimit
        | StandardInputBinding::ReadPage
        | StandardInputBinding::ReadLimit
        | StandardInputBinding::FindPage
        | StandardInputBinding::FindLimit => None,
    }
}

fn required_binding_identity<'a>(
    selected_adapter_id: &'a str,
    catalog: &'a DocumentParameterCatalog,
    binding: DocumentParameterBinding,
) -> Result<&'a str, NavigationError> {
    selected_binding_identity(selected_adapter_id, catalog, binding)
        .ok_or_else(|| NavigationError::internal("missing-parameter-binding"))
}

fn selected_binding_identity<'a>(
    selected_adapter_id: &'a str,
    catalog: &'a DocumentParameterCatalog,
    binding: DocumentParameterBinding,
) -> Option<&'a str> {
    catalog
        .selected_operation_parameters(selected_adapter_id, binding.operation())
        .find_map(|(_, entry, candidate)| (candidate == binding).then(|| entry.identity().as_str()))
}

fn selected_adapter_binding_identity<'a>(
    selected_adapter_id: &'a str,
    catalog: &'a DocumentParameterCatalog,
    binding: StandardInputBinding,
) -> Option<&'a str> {
    catalog
        .selected_operation_parameters(selected_adapter_id, binding.operation())
        .find_map(|(_, entry, candidate)| {
            (entry.adapter_id() == Some(selected_adapter_id)
                && candidate == DocumentParameterBinding::StandardInput(binding))
            .then(|| entry.identity().as_str())
        })
}

fn effective_limit(limit: PositiveInteger, pagination_enabled: bool) -> PositiveInteger {
    if pagination_enabled {
        limit
    } else {
        values::max_pagination_limit()
    }
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
    cli_source: Option<&Source>,
    config_sources: &NavigationConfigSources,
    context: &str,
) -> Result<ResolutionResult, NavigationError> {
    resolution::resolve(fields, direct_input.as_ref(), cli_source, config_sources)
        .map_err(|_| NavigationError::internal(resolution_pipeline_error_id(context)))
}

fn resolve_command_with_fields(
    fields: &docnav_typed_fields::FieldDefSet,
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    context: &str,
) -> Result<ResolutionResult, NavigationError> {
    resolve_with_fields(
        fields,
        Some(input::direct_input(command)),
        Some(&command.cli_source),
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
