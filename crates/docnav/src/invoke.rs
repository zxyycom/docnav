use docnav_protocol::{
    decode_protocol_request_value, generate_request_id, DecodePipelineError, Document,
    FindArguments, InfoArguments, Operation, OperationArguments, OutlineArguments, PositiveInteger,
    ProtocolResponse, ReadArguments, RequestEnvelope, StableError, PROTOCOL_VERSION,
};

use crate::adapter_output_contract::{adapter_invoke_failed, protocol_response_from_output};
use crate::adapter_process::run_invoke;
use crate::project_paths::NormalizedDocumentPath;
use crate::registry::AdapterRecord;
use crate::runtime::DocumentRequest;

pub struct InvokeOutcome {
    pub request: RequestEnvelope,
    pub response: ProtocolResponse,
}

pub fn invoke_adapter(
    project_root: &std::path::Path,
    record: &AdapterRecord,
    document: &NormalizedDocumentPath,
    request: &DocumentRequest,
) -> Result<InvokeOutcome, StableError> {
    let protocol_request = protocol_request(document, request)?;
    validate_protocol_request(&protocol_request)?;
    let output = run_invoke(project_root, record, &protocol_request).map_err(|error| {
        adapter_invoke_failed(&record.id, error.reason, error.exit_code, error.stderr)
    })?;
    let response = protocol_response_from_output(
        &record.id,
        &protocol_request.request_id,
        request.operation,
        output,
    )?;
    Ok(InvokeOutcome {
        request: protocol_request,
        response,
    })
}

fn validate_protocol_request(request: &RequestEnvelope) -> Result<(), StableError> {
    let value = serde_json::to_value(request).map_err(|error| {
        StableError::internal_error(format!("serialize-protocol-request:{error}"))
    })?;
    match decode_protocol_request_value(value) {
        Ok(_) => Ok(()),
        Err(DecodePipelineError::Schema(error)) => Err(StableError::invalid_request(
            "protocol_request",
            format!("protocol request schema validation failed: {error}"),
        )),
        Err(DecodePipelineError::Deserialize(error)) => Err(StableError::internal_error(format!(
            "decode-protocol-request:{error}"
        ))),
        Err(DecodePipelineError::Semantic { error, .. }) => Err(error),
    }
}

fn protocol_request(
    document: &NormalizedDocumentPath,
    request: &DocumentRequest,
) -> Result<RequestEnvelope, StableError> {
    let arguments = operation_arguments(request)?;

    Ok(RequestEnvelope {
        protocol_version: PROTOCOL_VERSION.to_owned(),
        request_id: generate_request_id(),
        operation: request.operation,
        document: Document {
            path: document.adapter_path.clone(),
        },
        arguments,
    })
}

fn operation_arguments(request: &DocumentRequest) -> Result<OperationArguments, StableError> {
    match request.operation {
        Operation::Outline => Ok(OperationArguments::Outline(outline_arguments(request)?)),
        Operation::Read => Ok(OperationArguments::Read(read_arguments(request)?)),
        Operation::Find => Ok(OperationArguments::Find(find_arguments(request)?)),
        Operation::Info => Ok(OperationArguments::Info(info_arguments())),
    }
}

fn outline_arguments(request: &DocumentRequest) -> Result<OutlineArguments, StableError> {
    Ok(OutlineArguments {
        limit_chars: required_limit_chars(request, "outline")?,
        page: required_page(request, "outline")?,
        options: None,
    })
}

fn read_arguments(request: &DocumentRequest) -> Result<ReadArguments, StableError> {
    Ok(ReadArguments {
        ref_id: required_ref_id(request)?,
        limit_chars: required_limit_chars(request, "read")?,
        page: required_page(request, "read")?,
        options: None,
    })
}

fn find_arguments(request: &DocumentRequest) -> Result<FindArguments, StableError> {
    Ok(FindArguments {
        query: required_query(request)?,
        limit_chars: required_limit_chars(request, "find")?,
        page: required_page(request, "find")?,
        options: None,
    })
}

fn info_arguments() -> InfoArguments {
    InfoArguments { options: None }
}

fn required_limit_chars(
    request: &DocumentRequest,
    operation: &str,
) -> Result<PositiveInteger, StableError> {
    request
        .limit_chars
        .ok_or_else(|| missing_argument("limit_chars", operation, "limit_chars"))
}

fn required_page(
    request: &DocumentRequest,
    operation: &str,
) -> Result<PositiveInteger, StableError> {
    request
        .page
        .ok_or_else(|| missing_argument("page", operation, "page"))
}

fn required_ref_id(request: &DocumentRequest) -> Result<String, StableError> {
    request
        .ref_id
        .clone()
        .ok_or_else(|| missing_argument("ref", "read", "ref"))
}

fn required_query(request: &DocumentRequest) -> Result<String, StableError> {
    request
        .query
        .clone()
        .ok_or_else(|| missing_argument("query", "find", "query"))
}

fn missing_argument(field: &str, operation: &str, argument: &str) -> StableError {
    StableError::invalid_request(field, format!("{operation} requires {argument}"))
}
