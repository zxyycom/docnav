use std::time::{SystemTime, UNIX_EPOCH};

use docnav_protocol::{
    validate_protocol_request_value, Document, FindArguments, InfoArguments, Operation,
    OperationArguments, OperationResult, OutlineArguments, ProtocolResponse, ReadArguments,
    RequestEnvelope, StableError, PROTOCOL_VERSION,
};
use serde_json::{json, Value};

use crate::cli::OutputMode;
use crate::contract::{adapter_invoke_failed, protocol_response_from_output};
use crate::error::exit_code_for_error;
use crate::output::CommandOutcome;
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

pub fn outcome_for_response(response: ProtocolResponse, output: OutputMode) -> CommandOutcome {
    match response {
        ProtocolResponse::Success(success) => match output {
            OutputMode::ProtocolJson => {
                CommandOutcome::protocol_json(serde_json::to_value(success).unwrap_or(Value::Null))
            }
            OutputMode::ReadableJson => CommandOutcome::json(readable_result(&success.result)),
            OutputMode::Text => CommandOutcome::text(text_result(&success.result)),
        },
        ProtocolResponse::Failure(failure) => {
            let exit_code = exit_code_for_error(failure.error.code);
            match output {
                OutputMode::ProtocolJson => CommandOutcome::protocol_json_with_exit(
                    serde_json::to_value(failure).unwrap_or(Value::Null),
                    exit_code,
                ),
                OutputMode::ReadableJson => CommandOutcome::json_with_exit(
                    json!({
                        "code": failure.error.code,
                        "error": failure.error.message,
                        "details": failure.error.details,
                        "guidance": failure.error.guidance.unwrap_or_default(),
                    }),
                    exit_code,
                ),
                OutputMode::Text => {
                    CommandOutcome::text_with_exit(text_error(&failure.error), exit_code)
                }
            }
        }
    }
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

fn readable_result(result: &OperationResult) -> Value {
    serde_json::to_value(result).unwrap_or(Value::Null)
}

fn text_result(result: &OperationResult) -> String {
    match result {
        OperationResult::Outline(result) => {
            let mut lines = result
                .entries
                .iter()
                .map(|entry| format!("{} | {}", entry.ref_id, entry.display))
                .collect::<Vec<_>>();
            lines.push(format!("page: {}", page_label(result.page)));
            lines.join("\n")
        }
        OperationResult::Read(result) => {
            let mut text = format!("ref: {}\n{}", result.ref_id, result.content);
            if !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(&format!(
                "content_type: {}\ncost: {}\npage: {}",
                result.content_type,
                result.cost,
                page_label(result.page)
            ));
            text
        }
        OperationResult::Find(result) => {
            let mut lines = result
                .matches
                .iter()
                .map(|entry| format!("{} | {}", entry.ref_id, entry.display))
                .collect::<Vec<_>>();
            lines.push(format!("page: {}", page_label(result.page)));
            lines.join("\n")
        }
        OperationResult::Info(result) => format!(
            "{}\ncapabilities: {}",
            result.display,
            result
                .capabilities
                .iter()
                .map(Operation::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn text_error(error: &StableError) -> String {
    let mut lines = vec![
        format!("error: {}", error_code_label(error.code)),
        format!("message: {}", error.message),
    ];
    if !error.details.is_empty() {
        lines.push(format!(
            "details: {}",
            serde_json::to_string(&error.details)
                .unwrap_or_else(|_| "<unserializable details>".to_owned())
        ));
    }
    if let Some(guidance) = &error.guidance {
        for item in guidance {
            lines.push(format!("guidance: {item}"));
        }
    }
    lines.join("\n")
}

fn error_code_label(code: docnav_protocol::StableErrorCode) -> String {
    serde_json::to_value(code)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("{code:?}"))
}

fn page_label(page: Option<docnav_protocol::PositiveInteger>) -> String {
    page.map(|page| page.get().to_string())
        .unwrap_or_else(|| "null".to_owned())
}

fn request_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("docnav-{nanos}")
}
