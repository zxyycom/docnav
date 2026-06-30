use docnav_diagnostics::{typed_codes, DiagnosticSource, FieldReasonDetails};
use docnav_protocol::protocol_error_record_draft_with_summary;
use docnav_protocol::{
    FindArguments, Operation, OperationArguments, OutlineArguments, PositiveInteger,
    RawRequestEnvelope, ReadArguments, RequestEnvelope,
};
use docnav_standard_parameters::{
    config_pagination_enabled_field, configurable_limit_field, find_query_field, ids,
    page_field as standard_page_field, read_ref_field, EntryPassthroughPolicy,
    StandardParameterConfigSourceIssue, StandardParameterHandoff, StandardParameterPipeline,
    StandardParameterResolution, StandardParameterValidationIssue, MAX_PAGINATION_LIMIT,
};
use docnav_typed_fields::{FieldDefs, FieldIdentity, JsonValue, TypedValue};
use serde_json::json;

use crate::AdapterError;

mod config;
mod native_options;
mod passthrough;

pub(crate) use config::{adapter_config_source_issue, InvokeStandardParameterConfig};
pub(crate) use passthrough::native_options_passthrough;

use config::loaded_config_source;
use native_options::validated_native_options;
use passthrough::{native_options_processing, options_from_resolution, raw_options};

const DIRECT_PROCESSING: &str = "direct";
const CONFIG_PROCESSING: &str = "config";
const DEFAULT_PAGE: i64 = 1;

use ids::{
    LIMIT as ID_LIMIT, PAGE as ID_PAGE, PAGINATION_ENABLED as ID_PAGINATION_ENABLED,
    QUERY as ID_QUERY, REF as ID_REF,
};

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
    #[field(invoke_pagination_enabled_field())]
    pagination_enabled: bool,
    #[field(invoke_page_field())]
    page: i64,
    #[field(invoke_limit_field())]
    limit: i64,
}

fn invoke_page_field() -> docnav_typed_fields::FieldDefBuilder<i64> {
    standard_page_field(DIRECT_PROCESSING).default_static(DEFAULT_PAGE)
}

fn invoke_limit_field() -> docnav_typed_fields::FieldDefBuilder<i64> {
    configurable_limit_field(DIRECT_PROCESSING, CONFIG_PROCESSING)
}

fn invoke_pagination_enabled_field() -> docnav_typed_fields::FieldDefBuilder<bool> {
    config_pagination_enabled_field(CONFIG_PROCESSING).default_static(true)
}

pub(crate) fn standardize_invoke_request(
    request: &RawRequestEnvelope,
    config: InvokeStandardParameterConfig,
) -> Result<RequestEnvelope, AdapterError> {
    let arguments = match request.operation {
        Operation::Outline => {
            OperationArguments::Outline(standardize_outline(&request.arguments, &config)?)
        }
        Operation::Read => OperationArguments::Read(standardize_read(&request.arguments, &config)?),
        Operation::Find => OperationArguments::Find(standardize_find(&request.arguments, &config)?),
        Operation::Info => OperationArguments::Info(standardize_info(&request.arguments, &config)?),
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
    config: &InvokeStandardParameterConfig,
) -> Result<OutlineArguments, AdapterError> {
    let resolution =
        resolve_invoke_standard_arguments::<InvokeOutlineStandardArguments>(arguments, config)?;

    Ok(OutlineArguments {
        limit: finalized_limit(&resolution)?,
        page: required_positive_value(&resolution, ID_PAGE)?,
        options: validated_native_options(
            Operation::Outline,
            options_from_resolution(&resolution),
            &config.native_options,
        )?,
    })
}

fn standardize_read(
    arguments: &JsonValue,
    config: &InvokeStandardParameterConfig,
) -> Result<ReadArguments, AdapterError> {
    let resolution =
        resolve_invoke_standard_arguments::<InvokeReadStandardArguments>(arguments, config)?;

    Ok(ReadArguments {
        ref_id: required_string_value(&resolution, ID_REF)?,
        limit: finalized_limit(&resolution)?,
        page: required_positive_value(&resolution, ID_PAGE)?,
        options: validated_native_options(
            Operation::Read,
            options_from_resolution(&resolution),
            &config.native_options,
        )?,
    })
}

fn standardize_find(
    arguments: &JsonValue,
    config: &InvokeStandardParameterConfig,
) -> Result<FindArguments, AdapterError> {
    let resolution =
        resolve_invoke_standard_arguments::<InvokeFindStandardArguments>(arguments, config)?;

    Ok(FindArguments {
        query: required_string_value(&resolution, ID_QUERY)?,
        limit: finalized_limit(&resolution)?,
        page: required_positive_value(&resolution, ID_PAGE)?,
        options: validated_native_options(
            Operation::Find,
            options_from_resolution(&resolution),
            &config.native_options,
        )?,
    })
}

fn standardize_info(
    arguments: &JsonValue,
    config: &InvokeStandardParameterConfig,
) -> Result<docnav_protocol::InfoArguments, AdapterError> {
    Ok(docnav_protocol::InfoArguments {
        options: validated_native_options(
            Operation::Info,
            raw_options(arguments),
            &config.native_options,
        )?,
    })
}

fn resolve_invoke_standard_arguments<P>(
    arguments: &JsonValue,
    config: &InvokeStandardParameterConfig,
) -> Result<StandardParameterResolution, AdapterError>
where
    P: FieldDefs,
    P::DefinitionSet: AsRef<docnav_typed_fields::FieldDefSet>,
{
    let fields = P::field_defs().map_err(field_defs_error)?;
    let resolution = resolve_with_fields(&fields, arguments.clone(), config)?;
    first_validation_error(&resolution)?;
    Ok(resolution)
}

fn resolve_with_fields<D>(
    fields: &D,
    direct_input: JsonValue,
    config: &InvokeStandardParameterConfig,
) -> Result<StandardParameterResolution, AdapterError>
where
    D: AsRef<docnav_typed_fields::FieldDefSet> + ?Sized,
{
    let mut pipeline = StandardParameterPipeline::new(fields)
        .with_direct_input_processing_id(DIRECT_PROCESSING)
        .with_config_processing_id(CONFIG_PROCESSING)
        .with_dynamic_default(identity_key(ID_LIMIT)?, json!(config.default_limit))
        .with_direct_input_passthrough_processing(native_options_processing()?)
        .with_passthrough_policy(EntryPassthroughPolicy::Delegate);
    if let Some(descriptor) = &config.project_config {
        pipeline = pipeline.with_loaded_project_config(loaded_config_source(descriptor)?);
    }
    if let Some(descriptor) = &config.user_config {
        pipeline = pipeline.with_loaded_user_config(loaded_config_source(descriptor)?);
    }
    pipeline
        .resolve(direct_input)
        .map_err(|error| AdapterError::internal(format!("invoke-standard-parameters:{error}")))
}

fn first_validation_error(resolution: &StandardParameterResolution) -> Result<(), AdapterError> {
    if let Some(diagnostic) = resolution.diagnostics().first() {
        return Err(match diagnostic {
            StandardParameterHandoff::Validation(diagnostic) => validation_error(diagnostic),
            StandardParameterHandoff::ConfigSource(issue) => config_source_error(issue),
        });
    }
    Ok(())
}

fn validation_error(diagnostic: &StandardParameterValidationIssue) -> AdapterError {
    invalid_request_error(
        argument_field(diagnostic.identity.as_str()),
        validation_reason(diagnostic.identity.as_str()),
    )
}

fn config_source_error(issue: &StandardParameterConfigSourceIssue) -> AdapterError {
    AdapterError::new(issue.to_record_draft(DiagnosticSource::with_stage(
        "docnav-adapter-sdk",
        "standard-parameters",
    )))
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
        ID_PAGINATION_ENABLED => "pagination must be enabled or disabled",
        ID_QUERY => "query must not be empty",
        ID_REF => "ref must not be empty",
        _ => "standard parameter validation failed",
    }
}

fn finalized_limit(
    resolution: &StandardParameterResolution,
) -> Result<PositiveInteger, AdapterError> {
    let enabled = required_bool_value(resolution, ID_PAGINATION_ENABLED)?;
    if enabled {
        return required_positive_value(resolution, ID_LIMIT);
    }
    std::num::NonZeroU32::new(MAX_PAGINATION_LIMIT)
        .ok_or_else(|| validation_error_for_identity(ID_LIMIT))
}

fn required_bool_value(
    resolution: &StandardParameterResolution,
    identity: &str,
) -> Result<bool, AdapterError> {
    let value = resolution.value(&identity_key(identity)?).ok_or_else(|| {
        AdapterError::internal(format!("missing-invoke-standard-parameter:{identity}"))
    })?;
    let TypedValue::Boolean(value) = value.value else {
        return Err(validation_error_for_identity(identity));
    };
    Ok(value)
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

pub(super) fn invalid_request_error(field: &str, reason: &str) -> AdapterError {
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
