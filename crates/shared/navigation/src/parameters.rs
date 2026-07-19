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
    AutoReadMode, NavigationCommand, NavigationConfigSource, NavigationConfigSources,
    NavigationContextDefaults, NavigationError, NavigationOutputMode, NavigationPaginationDefaults,
    NavigationResolvedValue,
};

pub use catalog::{
    DocumentParameterBinding, DocumentParameterCatalog, DocumentParameterCatalogBuildError,
    DocumentParameterEntry,
};
pub(crate) use fields::adapter_routing_fields;
pub(crate) use inspection::inspect_config_sources;

pub(super) mod ids {
    pub(super) const ADAPTER: &str = "docnav.defaults.adapter";
    pub(super) const AUTO_READ: &str = "docnav.defaults.auto_read";
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
    pub auto_read: Option<AutoReadMode>,
    pub options: Option<Options>,
    pub standard_input: StandardOperationInput,
}

impl FieldStringEnum for NavigationOutputMode {
    fn variants() -> &'static [Self] {
        &[Self::ReadableView, Self::ProtocolJson]
    }

    fn as_str(&self) -> &'static str {
        NavigationOutputMode::as_str(*self)
    }
}

impl FieldStringEnum for AutoReadMode {
    fn variants() -> &'static [Self] {
        &[Self::Disabled, Self::UniqueRef]
    }

    fn as_str(&self) -> &'static str {
        AutoReadMode::as_str(*self)
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

    let resolution =
        resolve_command_with_fields(operation_fields.as_ref(), command, config_sources)?;

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

pub(crate) fn validate_config_source_for_catalog(
    source: &NavigationConfigSource,
    catalog: &DocumentParameterCatalog,
) -> Result<(), NavigationError> {
    config::validate_config_source_for_catalog(source, catalog)
}

struct InputResolutionContext<'a> {
    selected_adapter_id: &'a str,
    catalog: &'a DocumentParameterCatalog,
    fields: &'a FieldDefSet,
    resolution: &'a ResolutionResult,
}

impl InputResolutionContext<'_> {
    fn required_string(&self, identity: &str) -> Result<String, NavigationError> {
        values::required_string_value(self.fields, self.resolution, identity)
    }

    fn required_positive_binding(
        &self,
        binding: DocumentParameterBinding,
    ) -> Result<PositiveInteger, NavigationError> {
        let identity = required_binding_identity(self.selected_adapter_id, self.catalog, binding)?;
        values::required_positive_value(self.fields, self.resolution, identity)
    }

    fn required_bool_binding(
        &self,
        binding: DocumentParameterBinding,
    ) -> Result<bool, NavigationError> {
        let identity = required_binding_identity(self.selected_adapter_id, self.catalog, binding)?;
        values::required_bool_value(self.fields, self.resolution, identity)
    }

    fn required_output_binding(
        &self,
        binding: DocumentParameterBinding,
    ) -> Result<NavigationOutputMode, NavigationError> {
        let identity = required_binding_identity(self.selected_adapter_id, self.catalog, binding)?;
        values::required_output_value_for_identity(self.fields, self.resolution, identity)
    }

    fn required_auto_read_binding(
        &self,
        binding: DocumentParameterBinding,
    ) -> Result<AutoReadMode, NavigationError> {
        let identity = required_binding_identity(self.selected_adapter_id, self.catalog, binding)?;
        let value = self.required_string(identity)?;
        AutoReadMode::parse(&value).map_err(|_| values::validation_error_for_identity(identity))
    }

    fn optional_adapter_integer_binding(
        &self,
        binding: StandardInputBinding,
    ) -> Result<Option<i64>, NavigationError> {
        let Some(identity) =
            selected_adapter_binding_identity(self.selected_adapter_id, self.catalog, binding)
        else {
            return Ok(None);
        };
        values::optional_integer_value(self.fields, self.resolution, identity)
    }
}

fn resolved_input_from_resolution(
    operation: Operation,
    selected_adapter_id: &str,
    catalog: &DocumentParameterCatalog,
    fields: &FieldDefSet,
    resolution: &ResolutionResult,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let context = InputResolutionContext {
        selected_adapter_id,
        catalog,
        fields,
        resolution,
    };
    let output =
        context.required_output_binding(DocumentParameterBinding::OutputMode(operation))?;

    match operation {
        Operation::Outline => resolved_outline_input(
            &context,
            output,
            context.required_auto_read_binding(DocumentParameterBinding::AutoReadMode(
                Operation::Outline,
            ))?,
        ),
        Operation::Read => resolved_read_input(&context, output),
        Operation::Find => resolved_find_input(
            &context,
            output,
            context.required_auto_read_binding(DocumentParameterBinding::AutoReadMode(
                Operation::Find,
            ))?,
        ),
        Operation::Info => resolved_info_input(&context, output),
    }
}

fn resolved_outline_input(
    context: &InputResolutionContext<'_>,
    output: NavigationOutputMode,
    auto_read: AutoReadMode,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let document_path = context.required_string(ids::PATH)?;
    let page = context.required_positive_binding(DocumentParameterBinding::StandardInput(
        StandardInputBinding::OutlinePage,
    ))?;
    let raw_limit = context.required_positive_binding(DocumentParameterBinding::StandardInput(
        StandardInputBinding::OutlineLimit,
    ))?;
    let pagination_enabled = context.required_bool_binding(
        DocumentParameterBinding::PaginationEnabled(PagedOperation::Outline),
    )?;
    let limit = effective_limit(raw_limit, pagination_enabled);
    let max_heading_binding = StandardInputBinding::OutlineMaxHeadingLevel;
    let max_heading_level = context.optional_adapter_integer_binding(max_heading_binding)?;
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
        auto_read: Some(auto_read),
        options,
        standard_input,
    })
}

fn resolved_read_input(
    context: &InputResolutionContext<'_>,
    output: NavigationOutputMode,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let document_path = context.required_string(ids::PATH)?;
    let ref_id = context.required_string(ids::REF)?;
    let page = context.required_positive_binding(DocumentParameterBinding::StandardInput(
        StandardInputBinding::ReadPage,
    ))?;
    let raw_limit = context.required_positive_binding(DocumentParameterBinding::StandardInput(
        StandardInputBinding::ReadLimit,
    ))?;
    let pagination_enabled = context.required_bool_binding(
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
        auto_read: None,
        options: None,
        standard_input,
    })
}

fn resolved_find_input(
    context: &InputResolutionContext<'_>,
    output: NavigationOutputMode,
    auto_read: AutoReadMode,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let document_path = context.required_string(ids::PATH)?;
    let query = context.required_string(ids::QUERY)?;
    let page = context.required_positive_binding(DocumentParameterBinding::StandardInput(
        StandardInputBinding::FindPage,
    ))?;
    let raw_limit = context.required_positive_binding(DocumentParameterBinding::StandardInput(
        StandardInputBinding::FindLimit,
    ))?;
    let pagination_enabled = context.required_bool_binding(
        DocumentParameterBinding::PaginationEnabled(PagedOperation::Find),
    )?;
    let limit = effective_limit(raw_limit, pagination_enabled);
    let max_heading_binding = StandardInputBinding::FindMaxHeadingLevel;
    let max_heading_level = context.optional_adapter_integer_binding(max_heading_binding)?;
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
        auto_read: Some(auto_read),
        options,
        standard_input,
    })
}

fn resolved_info_input(
    context: &InputResolutionContext<'_>,
    output: NavigationOutputMode,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let document_path = context.required_string(ids::PATH)?;
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
        auto_read: None,
        options: None,
        standard_input,
    })
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
) -> Result<ResolutionResult, NavigationError> {
    resolution::resolve(fields, direct_input.as_ref(), cli_source, config_sources)
}

fn resolve_command_with_fields(
    fields: &docnav_typed_fields::FieldDefSet,
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
) -> Result<ResolutionResult, NavigationError> {
    resolve_with_fields(
        fields,
        Some(input::direct_input(command)),
        Some(&command.cli_source),
        config_sources,
    )
}

#[cfg(test)]
mod tests;
