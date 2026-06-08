use std::time::{SystemTime, UNIX_EPOCH};

use docnav_protocol::{
    validate_protocol_request_value, Document, FindArguments, InfoArguments, Operation,
    OperationArguments, OutlineArguments, ProtocolResponse, ReadArguments, RequestEnvelope,
    StableError, PROTOCOL_VERSION,
};

use crate::contract::{adapter_invoke_failed, protocol_response_from_output};
use crate::process::run_invoke;
use crate::project::NormalizedDocumentPath;
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
    validate_protocol_request_value(&value).map_err(|error| {
        StableError::invalid_request(
            "protocol_request",
            format!("protocol request schema validation failed: {error}"),
        )
    })?;
    request.operation_arguments()?;
    Ok(())
}

fn protocol_request(
    document: &NormalizedDocumentPath,
    request: &DocumentRequest,
) -> Result<RequestEnvelope, StableError> {
    let arguments = match request.operation {
        Operation::Outline => OperationArguments::Outline(OutlineArguments {
            limit_chars: request.limit_chars.ok_or_else(|| {
                StableError::invalid_request("limit_chars", "outline requires limit_chars")
            })?,
            page: request
                .page
                .ok_or_else(|| StableError::invalid_request("page", "outline requires page"))?,
            options: None,
        }),
        Operation::Read => OperationArguments::Read(ReadArguments {
            ref_id: request
                .ref_id
                .clone()
                .ok_or_else(|| StableError::invalid_request("ref", "read requires ref"))?,
            limit_chars: request.limit_chars.ok_or_else(|| {
                StableError::invalid_request("limit_chars", "read requires limit_chars")
            })?,
            page: request
                .page
                .ok_or_else(|| StableError::invalid_request("page", "read requires page"))?,
            options: None,
        }),
        Operation::Find => OperationArguments::Find(FindArguments {
            query: request
                .query
                .clone()
                .ok_or_else(|| StableError::invalid_request("query", "find requires query"))?,
            limit_chars: request.limit_chars.ok_or_else(|| {
                StableError::invalid_request("limit_chars", "find requires limit_chars")
            })?,
            page: request
                .page
                .ok_or_else(|| StableError::invalid_request("page", "find requires page"))?,
            options: None,
        }),
        Operation::Info => OperationArguments::Info(InfoArguments { options: None }),
    };

    Ok(RequestEnvelope {
        protocol_version: PROTOCOL_VERSION.to_owned(),
        request_id: request_id(),
        operation: request.operation,
        document: Document {
            path: document.adapter_path.clone(),
        },
        arguments,
    })
}

fn request_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("docnav-{nanos}")
}
