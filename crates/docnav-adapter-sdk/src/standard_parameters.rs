use docnav_diagnostics::{typed_codes, DiagnosticSource, FieldReasonDetails};
use docnav_protocol::protocol_error_record_draft_with_summary;
use docnav_protocol::{
    FindArguments, Operation, OperationArguments, OutlineArguments, PositiveInteger,
    RawRequestEnvelope, ReadArguments, RequestEnvelope,
};
use docnav_standard_parameters::{
    find_query_field, ids, limit_field as standard_limit_field, page_field as standard_page_field,
    read_ref_field, EntryPassthroughPolicy, PassthroughValue, StandardParameterHandoff,
    StandardParameterPipeline, StandardParameterResolution, StandardParameterSourceKind,
    StandardParameterValidationIssue,
};
use docnav_typed_fields::{FieldDefs, FieldIdentity, JsonValue, ProcessingBuild, TypedValue};
use serde_json::{json, Map, Value};

use crate::AdapterError;

const DIRECT_PROCESSING: &str = "direct";
const CONFIG_PROCESSING: &str = "config";
const DEFAULT_PAGE: i64 = 1;

use ids::{LIMIT as ID_LIMIT, PAGE as ID_PAGE, QUERY as ID_QUERY, REF as ID_REF};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InvokeStandardParameterConfig {
    pub(crate) default_limit: u32,
}

// FieldDefs consumes these fields as metadata; runtime code uses the generated definition set.
#[allow(dead_code)]
#[derive(Debug, FieldDefs)]
struct InvokeOutlineStandardArguments {
    #[field(group)]
    content_window: InvokeContentWindowArguments,
}

#[allow(dead_code)]
#[derive(Debug, FieldDefs)]
struct InvokeReadStandardArguments {
    #[field(read_ref_field(DIRECT_PROCESSING))]
    ref_id: String,
    #[field(group)]
    content_window: InvokeContentWindowArguments,
}

#[allow(dead_code)]
#[derive(Debug, FieldDefs)]
struct InvokeFindStandardArguments {
    #[field(find_query_field(DIRECT_PROCESSING))]
    query: String,
    #[field(group)]
    content_window: InvokeContentWindowArguments,
}

#[allow(dead_code)]
#[derive(Debug, FieldDefs)]
struct InvokeContentWindowArguments {
    #[field(invoke_page_field())]
    page: i64,
    #[field(invoke_limit_field())]
    limit: i64,
}

fn invoke_page_field() -> docnav_typed_fields::FieldDefBuilder<i64> {
    standard_page_field(DIRECT_PROCESSING).default_static(DEFAULT_PAGE)
}

fn invoke_limit_field() -> docnav_typed_fields::FieldDefBuilder<i64> {
    standard_limit_field(DIRECT_PROCESSING)
}

pub(crate) fn standardize_invoke_request(
    request: &RawRequestEnvelope,
    config: InvokeStandardParameterConfig,
) -> Result<RequestEnvelope, AdapterError> {
    let arguments = match request.operation {
        Operation::Outline => OperationArguments::Outline(standardize_outline(
            &request.arguments,
            config.default_limit,
        )?),
        Operation::Read => {
            OperationArguments::Read(standardize_read(&request.arguments, config.default_limit)?)
        }
        Operation::Find => {
            OperationArguments::Find(standardize_find(&request.arguments, config.default_limit)?)
        }
        Operation::Info => OperationArguments::Info(standardize_info(&request.arguments)?),
    };

    Ok(RequestEnvelope {
        protocol_version: request.protocol_version.clone(),
        request_id: request.request_id.clone(),
        operation: request.operation,
        document: request.document.clone(),
        arguments,
    })
}

fn standardize_outline(
    arguments: &JsonValue,
    default_limit: u32,
) -> Result<OutlineArguments, AdapterError> {
    let resolution = resolve_invoke_standard_arguments::<InvokeOutlineStandardArguments>(
        arguments,
        default_limit,
    )?;

    Ok(OutlineArguments {
        limit: required_positive_value(&resolution, ID_LIMIT)?,
        page: required_positive_value(&resolution, ID_PAGE)?,
        options: options_from_resolution(&resolution),
    })
}

fn standardize_read(
    arguments: &JsonValue,
    default_limit: u32,
) -> Result<ReadArguments, AdapterError> {
    let resolution =
        resolve_invoke_standard_arguments::<InvokeReadStandardArguments>(arguments, default_limit)?;

    Ok(ReadArguments {
        ref_id: required_string_value(&resolution, ID_REF)?,
        limit: required_positive_value(&resolution, ID_LIMIT)?,
        page: required_positive_value(&resolution, ID_PAGE)?,
        options: options_from_resolution(&resolution),
    })
}

fn standardize_find(
    arguments: &JsonValue,
    default_limit: u32,
) -> Result<FindArguments, AdapterError> {
    let resolution =
        resolve_invoke_standard_arguments::<InvokeFindStandardArguments>(arguments, default_limit)?;

    Ok(FindArguments {
        query: required_string_value(&resolution, ID_QUERY)?,
        limit: required_positive_value(&resolution, ID_LIMIT)?,
        page: required_positive_value(&resolution, ID_PAGE)?,
        options: options_from_resolution(&resolution),
    })
}

fn standardize_info(arguments: &JsonValue) -> Result<docnav_protocol::InfoArguments, AdapterError> {
    Ok(docnav_protocol::InfoArguments {
        options: raw_options(arguments),
    })
}

fn resolve_invoke_standard_arguments<P>(
    arguments: &JsonValue,
    default_limit: u32,
) -> Result<StandardParameterResolution, AdapterError>
where
    P: FieldDefs,
    P::DefinitionSet: AsRef<docnav_typed_fields::FieldDefSet>,
{
    let fields = P::field_defs().map_err(field_defs_error)?;
    let resolution = resolve_with_fields(&fields, arguments.clone(), default_limit)?;
    first_validation_error(&resolution)?;
    Ok(resolution)
}

fn resolve_with_fields<D>(
    fields: &D,
    direct_input: JsonValue,
    default_limit: u32,
) -> Result<StandardParameterResolution, AdapterError>
where
    D: AsRef<docnav_typed_fields::FieldDefSet> + ?Sized,
{
    StandardParameterPipeline::new(fields)
        .with_direct_input_processing_id(DIRECT_PROCESSING)
        .with_config_processing_id(CONFIG_PROCESSING)
        .with_dynamic_default(identity_key(ID_LIMIT)?, json!(default_limit))
        .with_direct_input_passthrough_processing(native_options_processing()?)
        .with_passthrough_policy(EntryPassthroughPolicy::Delegate)
        .resolve(direct_input)
        .map_err(|error| AdapterError::internal(format!("invoke-standard-parameters:{error}")))
}

fn native_options_processing(
) -> Result<ProcessingBuild<'static, JsonValue, JsonValue>, AdapterError> {
    ProcessingBuild::new(DIRECT_PROCESSING, native_options_passthrough)
        .map_err(|error| AdapterError::internal(format!("invoke-passthrough-processing:{error}")))
}

pub(crate) fn native_options_passthrough(raw: JsonValue) -> JsonValue {
    raw_options(&raw)
        .map(Value::Object)
        .unwrap_or_else(|| Value::Object(Map::new()))
}

fn raw_options(raw: &JsonValue) -> Option<Map<String, Value>> {
    raw.get("options").and_then(Value::as_object).cloned()
}

fn first_validation_error(resolution: &StandardParameterResolution) -> Result<(), AdapterError> {
    if let Some(diagnostic) = resolution
        .diagnostics()
        .iter()
        .find_map(StandardParameterHandoff::as_validation)
    {
        return Err(validation_error(diagnostic));
    }
    Ok(())
}

fn validation_error(diagnostic: &StandardParameterValidationIssue) -> AdapterError {
    invalid_request_error(
        argument_field(diagnostic.identity.as_str()),
        validation_reason(diagnostic.identity.as_str()),
    )
}

fn argument_field(identity: &str) -> &'static str {
    match identity {
        ID_LIMIT => "arguments.limit",
        ID_PAGE => "arguments.page",
        ID_QUERY => "arguments.query",
        ID_REF => "arguments.ref",
        _ => "arguments",
    }
}

fn validation_reason(identity: &str) -> &'static str {
    match identity {
        ID_LIMIT => "limit must be a positive integer",
        ID_PAGE => "page must be a positive integer",
        ID_QUERY => "query must not be empty",
        ID_REF => "ref must not be empty",
        _ => "standard parameter validation failed",
    }
}

fn required_string_value(
    resolution: &StandardParameterResolution,
    identity: &str,
) -> Result<String, AdapterError> {
    let value = resolution.value(&identity_key(identity)?).ok_or_else(|| {
        AdapterError::internal(format!("missing-invoke-standard-parameter:{identity}"))
    })?;
    match &value.value {
        TypedValue::String(value) => Ok(value.clone()),
        _ => Err(AdapterError::internal(format!(
            "unexpected-invoke-standard-parameter-type:{identity}"
        ))),
    }
}

fn required_positive_value(
    resolution: &StandardParameterResolution,
    identity: &str,
) -> Result<PositiveInteger, AdapterError> {
    let value = resolution.value(&identity_key(identity)?).ok_or_else(|| {
        AdapterError::internal(format!("missing-invoke-standard-parameter:{identity}"))
    })?;
    let TypedValue::Integer(value) = value.value else {
        return Err(validation_error_for_identity(identity));
    };
    u32::try_from(value)
        .ok()
        .and_then(std::num::NonZeroU32::new)
        .ok_or_else(|| validation_error_for_identity(identity))
}

fn options_from_resolution(
    resolution: &StandardParameterResolution,
) -> Option<serde_json::Map<String, Value>> {
    let Value::Object(options) = passthrough_from_source(resolution)?.value.clone() else {
        return None;
    };
    (!options.is_empty()).then_some(options)
}

fn passthrough_from_source(resolution: &StandardParameterResolution) -> Option<&PassthroughValue> {
    resolution
        .passthrough()
        .iter()
        .find(|value| value.source.kind == StandardParameterSourceKind::DirectInput)
}

fn identity_key(identity: &str) -> Result<FieldIdentity, AdapterError> {
    FieldIdentity::new(identity).map_err(|error| {
        AdapterError::internal(format!(
            "invalid-invoke-standard-parameter-identity:{error}"
        ))
    })
}

fn validation_error_for_identity(identity: &str) -> AdapterError {
    invalid_request_error(argument_field(identity), validation_reason(identity))
}

fn invalid_request_error(field: &str, reason: &str) -> AdapterError {
    AdapterError::new(protocol_error_record_draft_with_summary::<
        typed_codes::protocol::InvalidRequest,
    >(
        reason,
        FieldReasonDetails::new(field, reason),
        DiagnosticSource::with_stage("docnav-adapter-sdk", "standard-parameters"),
    ))
}

fn field_defs_error(error: docnav_typed_fields::FieldDefSetBuildError) -> AdapterError {
    AdapterError::internal(format!("invoke-standard-parameter-defs:{error}"))
}
