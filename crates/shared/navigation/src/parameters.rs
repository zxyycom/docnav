mod config;
mod fields;
mod input;
mod native_options;
mod resolution;
mod values;

use docnav_adapter_contracts::{AdapterDefinition, AdapterOptionSpec, NativeOptionHandoff};
use docnav_protocol::{Operation, Options, PositiveInteger};
use docnav_typed_fields::{FieldStringEnum, ProcessingId};
use serde_json::{json, Value};

use crate::{
    NavigationAdapterRegistry, NavigationCommand, NavigationConfigSource, NavigationConfigSources,
    NavigationContextDefaults, NavigationError, NavigationOutputMode, NavigationPaginationDefaults,
    NavigationResolvedValue,
};

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

const DIRECT_PROCESSING: &str = "cli";
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
        adapter_id: values::optional_string_value(&resolution, ids::ADAPTER)?,
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
        operation_fields.as_ref(),
        selected_native_options,
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

    resolved_input_from_resolution(command.operation, &resolution, selected_native_options)
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
        operation_fields.as_ref(),
        selected_native_options,
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

    defaults_from_resolution(command.operation, &resolution)
}

pub(crate) fn validate_config_source_for_registry(
    source: &NavigationConfigSource,
    registry: &(impl NavigationAdapterRegistry + ?Sized),
) -> Result<(), NavigationError> {
    config::validate_config_source_for_registry(source, registry)
}

pub(crate) fn inspect_config_sources(
    config_sources: &NavigationConfigSources,
    registry: &(impl NavigationAdapterRegistry + ?Sized),
) -> Result<Value, NavigationError> {
    let field_set = fields::config_inspection_fields(registry)?;
    let fields = field_set.as_ref();
    let resolution = resolve_with_fields(fields, None, config_sources, "config-inspection")?;

    Ok(json!({
        "sources": [
            inspect_source(&config_sources.project, registry),
            inspect_source(&config_sources.user, registry),
        ],
        "config_source_projection": config_source_projection(fields),
        "parameter_facts": parameter_facts(&resolution),
        "parameter_diagnostics": parameter_diagnostics(&resolution),
    }))
}

fn resolved_input_from_resolution(
    operation: Operation,
    resolution: &resolution::ParameterResolution,
    selected_native_options: &[AdapterOptionSpec],
) -> Result<ResolvedNavigationInput, NavigationError> {
    let options = native_options::resolved_options(resolution, selected_native_options)?;
    let native_option_handoff =
        NativeOptionHandoff::from_options((!options.is_empty()).then_some(&options));
    Ok(ResolvedNavigationInput {
        document_path: values::required_string_value(resolution, ids::PATH)?,
        ref_id: values::optional_string_value(resolution, ids::REF)?,
        query: values::optional_string_value(resolution, ids::QUERY)?,
        page: values::optional_document_positive(operation, resolution, ids::PAGE)?,
        limit: values::optional_document_limit(operation, resolution)?,
        output: values::required_output_value(resolution)?,
        options: (!options.is_empty()).then_some(options),
        native_options: native_option_handoff,
    })
}

fn defaults_from_resolution(
    operation: Operation,
    resolution: &resolution::ParameterResolution,
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
    resolution: &resolution::ParameterResolution,
    identity: &str,
) -> Result<NavigationResolvedValue, NavigationError> {
    resolved_value(resolution, identity)
        .ok_or_else(|| NavigationError::internal("missing-resolved-navigation-parameter"))
}

fn resolved_value(
    resolution: &resolution::ParameterResolution,
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
    direct_input: Option<Value>,
    config_sources: &NavigationConfigSources,
    context: &str,
) -> Result<resolution::ParameterResolution, NavigationError> {
    resolution::resolve(fields, direct_input.as_ref(), config_sources)
        .map_err(|_| NavigationError::internal(resolution_pipeline_error_id(context)))
}

fn resolve_command_with_fields(
    fields: &docnav_typed_fields::FieldDefSet,
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selected_native_options: &[AdapterOptionSpec],
    context: &str,
) -> Result<resolution::ParameterResolution, NavigationError> {
    resolve_with_fields(
        fields,
        Some(input::direct_input(command, selected_native_options)),
        config_sources,
        context,
    )
}

fn inspect_source(
    source: &NavigationConfigSource,
    registry: &(impl NavigationAdapterRegistry + ?Sized),
) -> Value {
    let diagnostics = source_diagnostics(source, registry);
    let load_state = load_state(source);
    json!({
        "scope": source.level,
        "path": source.path,
        "origin": source.origin,
        "exists": std::path::Path::new(&source.path).exists(),
        "load_state": load_state,
        "summary": source.loaded.value().map(source_summary).unwrap_or_else(|| {
            json!({
                "top_level_fields": [],
                "field_count": 0
            })
        }),
        "diagnostics": diagnostics,
    })
}

fn source_diagnostics(
    source: &NavigationConfigSource,
    registry: &(impl NavigationAdapterRegistry + ?Sized),
) -> Vec<Value> {
    let mut diagnostics = source
        .loaded
        .diagnostics()
        .iter()
        .map(config_source_issue_json)
        .collect::<Vec<_>>();
    if diagnostics.is_empty() {
        if let Err(error) = validate_config_source_for_registry(source, registry) {
            diagnostics.push(diagnostic_json(error.diagnostic()));
        }
    }
    diagnostics
}

fn load_state(source: &NavigationConfigSource) -> String {
    if source.loaded.value().is_some() {
        return "loaded".to_owned();
    }
    source
        .loaded
        .diagnostics()
        .first()
        .map_or_else(|| "missing".to_owned(), |issue| issue.reason_code.clone())
}

fn source_summary(value: &Value) -> Value {
    let fields = value
        .as_object()
        .map(|object| object.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    json!({
        "top_level_fields": fields,
        "field_count": flatten_fields(value).len(),
    })
}

fn flatten_fields(value: &Value) -> Vec<String> {
    let mut fields = Vec::new();
    flatten_value(value, String::new(), &mut fields);
    fields
}

fn flatten_value(value: &Value, path: String, fields: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                flatten_value(child, child_path, fields);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                flatten_value(child, format!("{path}[{index}]"), fields);
            }
        }
        _ if !path.is_empty() => fields.push(path),
        _ => {}
    }
}

fn config_source_projection(fields: &docnav_typed_fields::FieldDefSet) -> Vec<Value> {
    fields
        .processing_metadata(&ProcessingId::from(CONFIG_PROCESSING))
        .into_iter()
        .map(|metadata| {
            json!({
                "identity": metadata.identity.as_str(),
                "path": metadata.path.segments().join("."),
                "value_kind": format!("{:?}", metadata.value_kind),
                "has_default": !matches!(metadata.default, docnav_typed_fields::DefaultMetadata::None),
            })
        })
        .collect()
}

fn parameter_facts(resolution: &resolution::ParameterResolution) -> Vec<Value> {
    resolution
        .values()
        .iter()
        .map(|(identity, resolved)| {
            json!({
                "identity": identity.as_str(),
                "source": values::source_label(resolved.source.kind),
                "value": values::typed_value_to_json(&resolved.value),
            })
        })
        .collect()
}

fn parameter_diagnostics(resolution: &resolution::ParameterResolution) -> Vec<Value> {
    resolution
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            diagnostic_json(&diagnostic.to_record_draft(
                docnav_diagnostics::DiagnosticSource::with_stage(
                    "docnav-navigation",
                    "config-inspect",
                ),
            ))
        })
        .collect()
}

fn config_source_issue_json(
    diagnostic: &crate::config_source::NavigationConfigSourceIssue,
) -> Value {
    json!({
        "source_level": diagnostic.source_level,
        "path_origin": diagnostic.path_origin,
        "path": diagnostic.path,
        "field": diagnostic.field,
        "reason_code": diagnostic.reason_code,
    })
}

fn diagnostic_json(diagnostic: &docnav_diagnostics::DiagnosticRecordDraft) -> Value {
    diagnostic.details().to_value()
}

fn resolution_pipeline_error_id(context: &str) -> &'static str {
    match context {
        "adapter-intent" => "adapter-intent-resolution-failed",
        "operation-input" => "operation-input-resolution-failed",
        "context-defaults" => "context-defaults-resolution-failed",
        _ => "parameter-resolution-failed",
    }
}
