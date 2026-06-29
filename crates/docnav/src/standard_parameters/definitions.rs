use docnav_protocol::Operation;
use docnav_standard_parameters::{
    adapter_selection_field, configurable_limit_field, configurable_output_field,
    document_path_field, find_query_field, page_field as standard_page_field, read_ref_field,
    EntryPassthroughPolicy, LoadedStandardParameterConfigSource, StandardParameterPipeline,
    StandardParameterResolution,
};
use docnav_typed_fields::{FieldDefBuilder, FieldDefs, JsonValue};
use serde_json::{json, Map, Value};

use crate::cli::{DocumentCommand, OutputMode};
use crate::config::ConfigContext;
use crate::error::{AppError, AppResult};

const DIRECT_PROCESSING: &str = "direct";
const CONFIG_PROCESSING: &str = "config";

const DEFAULT_LIMIT: i64 = 6000;
const DEFAULT_PAGE: i64 = 1;

// FieldDefs consumes these fields as metadata; runtime code uses the generated definition set.
#[allow(dead_code)]
#[derive(Debug, FieldDefs)]
struct CoreOutlineStandardParameters {
    #[field(document_path_field(DIRECT_PROCESSING))]
    path: String,
    #[field(adapter_selection_field(DIRECT_PROCESSING, CONFIG_PROCESSING))]
    adapter: Option<String>,
    #[field(group)]
    content_window: CoreContentWindowParameters,
    #[field(core_output_field())]
    output: OutputMode,
}

#[allow(dead_code)]
#[derive(Debug, FieldDefs)]
struct CoreReadStandardParameters {
    #[field(document_path_field(DIRECT_PROCESSING))]
    path: String,
    #[field(read_ref_field(DIRECT_PROCESSING))]
    ref_id: String,
    #[field(adapter_selection_field(DIRECT_PROCESSING, CONFIG_PROCESSING))]
    adapter: Option<String>,
    #[field(group)]
    content_window: CoreContentWindowParameters,
    #[field(core_output_field())]
    output: OutputMode,
}

#[allow(dead_code)]
#[derive(Debug, FieldDefs)]
struct CoreFindStandardParameters {
    #[field(document_path_field(DIRECT_PROCESSING))]
    path: String,
    #[field(find_query_field(DIRECT_PROCESSING))]
    query: String,
    #[field(adapter_selection_field(DIRECT_PROCESSING, CONFIG_PROCESSING))]
    adapter: Option<String>,
    #[field(group)]
    content_window: CoreContentWindowParameters,
    #[field(core_output_field())]
    output: OutputMode,
}

#[allow(dead_code)]
#[derive(Debug, FieldDefs)]
struct CoreInfoStandardParameters {
    #[field(document_path_field(DIRECT_PROCESSING))]
    path: String,
    #[field(adapter_selection_field(DIRECT_PROCESSING, CONFIG_PROCESSING))]
    adapter: Option<String>,
    #[field(core_output_field())]
    output: OutputMode,
}

#[allow(dead_code)]
#[derive(Debug, FieldDefs)]
struct CoreContentWindowParameters {
    #[field(core_page_field())]
    page: i64,
    #[field(core_limit_field())]
    limit: i64,
}

fn core_page_field() -> FieldDefBuilder<i64> {
    standard_page_field(DIRECT_PROCESSING).default_static(DEFAULT_PAGE)
}

fn core_limit_field() -> FieldDefBuilder<i64> {
    configurable_limit_field(DIRECT_PROCESSING, CONFIG_PROCESSING).default_static(DEFAULT_LIMIT)
}

fn core_output_field() -> FieldDefBuilder<OutputMode> {
    configurable_output_field::<OutputMode>(DIRECT_PROCESSING, CONFIG_PROCESSING)
        .default_static(OutputMode::ReadableView)
}

pub(super) fn resolve_for_operation(
    command: &DocumentCommand,
    context: &ConfigContext,
) -> AppResult<StandardParameterResolution> {
    match command.operation {
        Operation::Outline => {
            resolve_core_standard_parameters::<CoreOutlineStandardParameters>(command, context)
        }
        Operation::Read => {
            resolve_core_standard_parameters::<CoreReadStandardParameters>(command, context)
        }
        Operation::Find => {
            resolve_core_standard_parameters::<CoreFindStandardParameters>(command, context)
        }
        Operation::Info => {
            resolve_core_standard_parameters::<CoreInfoStandardParameters>(command, context)
        }
    }
}

fn resolve_core_standard_parameters<P>(
    command: &DocumentCommand,
    context: &ConfigContext,
) -> AppResult<StandardParameterResolution>
where
    P: FieldDefs,
    P::DefinitionSet: AsRef<docnav_typed_fields::FieldDefSet>,
{
    let fields = P::field_defs().map_err(field_defs_error)?;
    resolve_with_fields(&fields, command, context)
}

fn resolve_with_fields<D>(
    fields: &D,
    command: &DocumentCommand,
    context: &ConfigContext,
) -> AppResult<StandardParameterResolution>
where
    D: AsRef<docnav_typed_fields::FieldDefSet> + ?Sized,
{
    StandardParameterPipeline::new(fields)
        .with_direct_input_processing_id(DIRECT_PROCESSING)
        .with_config_processing_id(CONFIG_PROCESSING)
        .with_loaded_project_config(LoadedStandardParameterConfigSource::from_value(
            config_value(&context.project_config)?,
        ))
        .with_loaded_user_config(LoadedStandardParameterConfigSource::from_value(
            config_value(&context.user_config)?,
        ))
        .with_passthrough_policy(EntryPassthroughPolicy::Discard)
        .resolve(direct_input(command))
        .map_err(|error| AppError::internal(format!("core-standard-parameters:{error}")))
}

fn direct_input(command: &DocumentCommand) -> JsonValue {
    let mut input = Map::new();
    input.insert("path".to_owned(), json!(command.path));
    if let Some(adapter) = &command.adapter {
        input.insert("adapter".to_owned(), json!(adapter));
    }
    if let Some(ref_id) = &command.ref_id {
        input.insert("ref".to_owned(), json!(ref_id));
    }
    if let Some(query) = &command.query {
        input.insert("query".to_owned(), json!(query));
    }
    if let Some(page) = command.page {
        input.insert("page".to_owned(), json!(page.get()));
    }
    if let Some(limit) = command.limit {
        input.insert("limit".to_owned(), json!(limit.get()));
    }
    if let Some(output) = command.output {
        input.insert("output".to_owned(), json!(output.as_str()));
    }
    Value::Object(input)
}

fn config_value<T: serde::Serialize>(config: &T) -> AppResult<JsonValue> {
    serde_json::to_value(config)
        .map_err(|error| AppError::internal(format!("serialize-core-config:{error}")))
}

fn field_defs_error(error: docnav_typed_fields::FieldDefSetBuildError) -> AppError {
    AppError::internal(format!("core-standard-parameter-defs:{error}"))
}
