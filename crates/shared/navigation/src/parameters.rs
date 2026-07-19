mod catalog;
mod config;
mod defaults;
mod fields;
mod input;
mod inspection;
mod native_options;
mod projection;
mod resolution;
mod values;

use cli_config_resolution::{FieldDefSet, ResolutionResult, Source};
use docnav_adapter_contracts::{StandardInputBinding, StandardOperationInput};
use docnav_protocol::{Options, PositiveInteger};
use docnav_typed_fields::FieldStringEnum;

use crate::{
    AutoReadMode, NavigationCommand, NavigationConfigSource, NavigationConfigSources,
    NavigationError, NavigationOutputMode,
};

pub use catalog::{
    DocumentParameterBinding, DocumentParameterCatalog, DocumentParameterCatalogBuildError,
    DocumentParameterEntry,
};
pub(crate) use defaults::resolve_context_defaults;
pub(crate) use fields::adapter_routing_fields;
pub(crate) use inspection::inspect_config_sources;
pub(crate) use projection::resolve_operation_input;

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
