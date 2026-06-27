#![allow(dead_code)]

use docnav_diagnostics::{typed_codes, DiagnosticSource, FieldReasonDetails};
use docnav_protocol::protocol_error_record_draft_with_summary;
use docnav_protocol::{
    FindArguments, Operation, OperationArguments, OutlineArguments, PositiveInteger, ReadArguments,
    RequestEnvelope,
};
use docnav_standard_parameters::{
    find_query_field, ids, limit_chars_field as standard_limit_chars_field,
    page_field as standard_page_field, read_ref_field, EntryPassthroughPolicy, PassthroughValue,
    StandardParameterHandoff, StandardParameterPipeline, StandardParameterResolution,
    StandardParameterSourceKind, StandardParameterValidationIssue,
};
use docnav_typed_fields::{FieldDefs, FieldIdentity, JsonValue, ProcessingBuild, TypedValue};
use serde_json::{Map, Value};

use crate::constants::fields;
use crate::AdapterError;

const DIRECT_PROCESSING: &str = "direct";
const CONFIG_PROCESSING: &str = "config";

use ids::{LIMIT_CHARS as ID_LIMIT_CHARS, PAGE as ID_PAGE, QUERY as ID_QUERY, REF as ID_REF};

#[derive(Debug, FieldDefs)]
struct InvokeOutlineStandardArguments {
    #[field(group)]
    content_window: InvokeContentWindowArguments,
}

#[derive(Debug, FieldDefs)]
struct InvokeReadStandardArguments {
    #[field(read_ref_field(DIRECT_PROCESSING))]
    ref_id: String,
    #[field(group)]
    content_window: InvokeContentWindowArguments,
}

#[derive(Debug, FieldDefs)]
struct InvokeFindStandardArguments {
    #[field(find_query_field(DIRECT_PROCESSING))]
    query: String,
    #[field(group)]
    content_window: InvokeContentWindowArguments,
}

#[derive(Debug, FieldDefs)]
struct InvokeContentWindowArguments {
    #[field(standard_page_field(DIRECT_PROCESSING))]
    page: i64,
    #[field(standard_limit_chars_field(DIRECT_PROCESSING))]
    limit_chars: i64,
}

pub(crate) fn standardize_invoke_request(
    request: &RequestEnvelope,
) -> Result<RequestEnvelope, AdapterError> {
    let arguments = match (&request.operation, &request.arguments) {
        (Operation::Outline, OperationArguments::Outline(arguments)) => {
            OperationArguments::Outline(standardize_outline(arguments)?)
        }
        (Operation::Read, OperationArguments::Read(arguments)) => {
            OperationArguments::Read(standardize_read(arguments)?)
        }
        (Operation::Find, OperationArguments::Find(arguments)) => {
            OperationArguments::Find(standardize_find(arguments)?)
        }
        (Operation::Info, OperationArguments::Info(arguments)) => {
            OperationArguments::Info(arguments.clone())
        }
        _ => {
            return Err(AdapterError::invalid_request(
                fields::ARGUMENTS,
                format!("arguments do not match operation {}", request.operation),
            ))
        }
    };

    Ok(RequestEnvelope {
        protocol_version: request.protocol_version.clone(),
        request_id: request.request_id.clone(),
        operation: request.operation,
        document: request.document.clone(),
        arguments,
    })
}

fn standardize_outline(arguments: &OutlineArguments) -> Result<OutlineArguments, AdapterError> {
    let resolution =
        resolve_invoke_standard_arguments::<InvokeOutlineStandardArguments, _>(arguments)?;

    Ok(OutlineArguments {
        limit_chars: required_positive_value(&resolution, ID_LIMIT_CHARS)?,
        page: required_positive_value(&resolution, ID_PAGE)?,
        options: options_from_resolution(&resolution),
    })
}

fn standardize_read(arguments: &ReadArguments) -> Result<ReadArguments, AdapterError> {
    let resolution =
        resolve_invoke_standard_arguments::<InvokeReadStandardArguments, _>(arguments)?;

    Ok(ReadArguments {
        ref_id: required_string_value(&resolution, ID_REF)?,
        limit_chars: required_positive_value(&resolution, ID_LIMIT_CHARS)?,
        page: required_positive_value(&resolution, ID_PAGE)?,
        options: options_from_resolution(&resolution),
    })
}

fn standardize_find(arguments: &FindArguments) -> Result<FindArguments, AdapterError> {
    let resolution =
        resolve_invoke_standard_arguments::<InvokeFindStandardArguments, _>(arguments)?;

    Ok(FindArguments {
        query: required_string_value(&resolution, ID_QUERY)?,
        limit_chars: required_positive_value(&resolution, ID_LIMIT_CHARS)?,
        page: required_positive_value(&resolution, ID_PAGE)?,
        options: options_from_resolution(&resolution),
    })
}

fn resolve_invoke_standard_arguments<P, A>(
    arguments: &A,
) -> Result<StandardParameterResolution, AdapterError>
where
    P: FieldDefs,
    P::DefinitionSet: AsRef<docnav_typed_fields::FieldDefSet>,
    A: serde::Serialize,
{
    let direct_input = serde_json::to_value(arguments).map_err(serialize_error)?;
    let fields = P::field_defs().map_err(field_defs_error)?;
    let resolution = resolve_with_fields(&fields, direct_input)?;
    first_validation_error(&resolution)?;
    Ok(resolution)
}

fn resolve_with_fields<D>(
    fields: &D,
    direct_input: JsonValue,
) -> Result<StandardParameterResolution, AdapterError>
where
    D: AsRef<docnav_typed_fields::FieldDefSet> + ?Sized,
{
    StandardParameterPipeline::new(fields)
        .with_direct_input_processing_id(DIRECT_PROCESSING)
        .with_config_processing_id(CONFIG_PROCESSING)
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
    raw.get("options")
        .and_then(Value::as_object)
        .cloned()
        .map(Value::Object)
        .unwrap_or_else(|| Value::Object(Map::new()))
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
        ID_LIMIT_CHARS => "arguments.limit_chars",
        ID_PAGE => "arguments.page",
        ID_QUERY => "arguments.query",
        ID_REF => "arguments.ref",
        _ => "arguments",
    }
}

fn validation_reason(identity: &str) -> &'static str {
    match identity {
        ID_LIMIT_CHARS => "limit_chars must be a positive integer",
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

fn serialize_error(error: serde_json::Error) -> AdapterError {
    AdapterError::internal(format!("serialize-invoke-arguments:{error}"))
}

fn field_defs_error(error: docnav_typed_fields::FieldDefSetBuildError) -> AdapterError {
    AdapterError::internal(format!("invoke-standard-parameter-defs:{error}"))
}
