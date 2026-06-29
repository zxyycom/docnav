use docnav_protocol::Operation;
use docnav_standard_parameters::{
    configurable_limit_field, configurable_output_field, document_path_field, find_query_field,
    page_field as standard_page_field, read_ref_field, EntryPassthroughPolicy,
    StandardParameterConfigSourceDescriptor, StandardParameterPipeline,
    StandardParameterResolution,
};
use docnav_typed_fields::{FieldDefBuilder, FieldDefs, FieldIdentity, JsonValue, ProcessingBuild};
use serde_json::json;

use crate::standard_parameters::native_options_passthrough;

use super::super::output::DirectOutputMode;

pub(super) const DIRECT_PROCESSING: &str = "direct";
pub(super) const CONFIG_PROCESSING: &str = "config";

pub(super) use docnav_standard_parameters::ids::{
    LIMIT as ID_LIMIT, OUTPUT as ID_OUTPUT, PAGE as ID_PAGE, PATH as ID_PATH, QUERY as ID_QUERY,
    REF as ID_REF,
};

pub(super) const DEFAULT_LIMIT_TEXT: &str = "6000";
pub(super) const DEFAULT_OUTPUT_TEXT: &str = "readable-view";
pub(super) const DEFAULT_PAGE_TEXT: &str = "1";
pub(super) const DEFAULT_PROTOCOL_OUTPUT_TEXT: &str = "protocol-json";

const DEFAULT_PAGE: i64 = 1;

impl docnav_typed_fields::FieldStringEnum for DirectOutputMode {
    fn variants() -> &'static [Self] {
        &[Self::ReadableView, Self::ReadableJson, Self::ProtocolJson]
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::ReadableView => "readable-view",
            Self::ReadableJson => "readable-json",
            Self::ProtocolJson => DEFAULT_PROTOCOL_OUTPUT_TEXT,
        }
    }
}

// FieldDefs consumes these fields as metadata; runtime code uses the generated definition set.
#[allow(dead_code)]
#[derive(Debug, FieldDefs)]
struct DirectOutlineStandardParameters {
    #[field(document_path_field(DIRECT_PROCESSING))]
    path: String,
    #[field(group)]
    content_window: DirectContentWindowParameters,
    #[field(direct_cli_output_field())]
    output: DirectOutputMode,
}

#[allow(dead_code)]
#[derive(Debug, FieldDefs)]
struct DirectReadStandardParameters {
    #[field(document_path_field(DIRECT_PROCESSING))]
    path: String,
    #[field(read_ref_field(DIRECT_PROCESSING))]
    ref_id: String,
    #[field(group)]
    content_window: DirectContentWindowParameters,
    #[field(direct_cli_output_field())]
    output: DirectOutputMode,
}

#[allow(dead_code)]
#[derive(Debug, FieldDefs)]
struct DirectFindStandardParameters {
    #[field(document_path_field(DIRECT_PROCESSING))]
    path: String,
    #[field(find_query_field(DIRECT_PROCESSING))]
    query: String,
    #[field(group)]
    content_window: DirectContentWindowParameters,
    #[field(direct_cli_output_field())]
    output: DirectOutputMode,
}

#[allow(dead_code)]
#[derive(Debug, FieldDefs)]
struct DirectInfoStandardParameters {
    #[field(document_path_field(DIRECT_PROCESSING))]
    path: String,
    #[field(direct_cli_output_field())]
    output: DirectOutputMode,
}

#[allow(dead_code)]
#[derive(Debug, FieldDefs)]
struct DirectContentWindowParameters {
    #[field(direct_cli_page_field())]
    page: i64,
    #[field(direct_cli_limit_field())]
    limit: i64,
}

fn direct_cli_page_field() -> FieldDefBuilder<i64> {
    standard_page_field(DIRECT_PROCESSING).default_static(DEFAULT_PAGE)
}

fn direct_cli_limit_field() -> FieldDefBuilder<i64> {
    configurable_limit_field(DIRECT_PROCESSING, CONFIG_PROCESSING)
}

fn direct_cli_output_field() -> FieldDefBuilder<DirectOutputMode> {
    configurable_output_field::<DirectOutputMode>(DIRECT_PROCESSING, CONFIG_PROCESSING)
        .default_static(DirectOutputMode::ReadableView)
}

pub(super) fn resolve_operation_parameters(
    operation: Operation,
    direct_input: JsonValue,
    project_config: StandardParameterConfigSourceDescriptor,
    user_config: StandardParameterConfigSourceDescriptor,
    default_limit: u32,
) -> Result<StandardParameterResolution, String> {
    match operation {
        Operation::Outline => {
            resolve_direct_standard_parameters::<DirectOutlineStandardParameters>(
                direct_input,
                project_config,
                user_config,
                default_limit,
            )
        }
        Operation::Read => resolve_direct_standard_parameters::<DirectReadStandardParameters>(
            direct_input,
            project_config,
            user_config,
            default_limit,
        ),
        Operation::Find => resolve_direct_standard_parameters::<DirectFindStandardParameters>(
            direct_input,
            project_config,
            user_config,
            default_limit,
        ),
        Operation::Info => resolve_direct_standard_parameters::<DirectInfoStandardParameters>(
            direct_input,
            project_config,
            user_config,
            default_limit,
        ),
    }
}

fn resolve_direct_standard_parameters<P>(
    direct_input: JsonValue,
    project_config: StandardParameterConfigSourceDescriptor,
    user_config: StandardParameterConfigSourceDescriptor,
    default_limit: u32,
) -> Result<StandardParameterResolution, String>
where
    P: FieldDefs,
    P::DefinitionSet: AsRef<docnav_typed_fields::FieldDefSet>,
{
    let fields = P::field_defs().map_err(field_defs_error)?;
    resolve_with_fields(
        &fields,
        direct_input,
        project_config,
        user_config,
        default_limit,
    )
}

fn resolve_with_fields<D>(
    fields: &D,
    direct_input: JsonValue,
    project_config: StandardParameterConfigSourceDescriptor,
    user_config: StandardParameterConfigSourceDescriptor,
    default_limit: u32,
) -> Result<StandardParameterResolution, String>
where
    D: AsRef<docnav_typed_fields::FieldDefSet> + ?Sized,
{
    StandardParameterPipeline::new(fields)
        .with_direct_input_processing_id(DIRECT_PROCESSING)
        .with_config_processing_id(CONFIG_PROCESSING)
        .with_config_source_descriptor(project_config)
        .with_config_source_descriptor(user_config)
        .with_dynamic_default(identity(ID_LIMIT)?, json!(default_limit))
        .with_direct_input_passthrough_processing(native_options_processing(DIRECT_PROCESSING)?)
        .with_config_passthrough_processing(native_options_processing(CONFIG_PROCESSING)?)
        .with_passthrough_policy(EntryPassthroughPolicy::Delegate)
        .resolve(direct_input)
        .map_err(|error| format!("standard parameter resolution failed: {error}"))
}

fn native_options_processing(
    processing_id: &'static str,
) -> Result<ProcessingBuild<'static, JsonValue, JsonValue>, String> {
    ProcessingBuild::new(processing_id, native_options_passthrough)
        .map_err(|error| format!("invalid passthrough processing id: {error}"))
}

fn field_defs_error(error: docnav_typed_fields::FieldDefSetBuildError) -> String {
    format!("direct CLI standard parameter definitions failed: {error}")
}

fn identity(value: &str) -> Result<FieldIdentity, String> {
    FieldIdentity::new(value)
        .map_err(|error| format!("invalid standard parameter identity: {error}"))
}
