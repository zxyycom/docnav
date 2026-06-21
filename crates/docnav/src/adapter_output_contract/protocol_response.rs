use docnav_protocol::{
    decode_protocol_response_value, DecodePipelineError, FailureResponse, Operation,
    ProtocolResponse, StableError, SuccessResponse, PROTOCOL_VERSION,
};
use serde_json::Value;

use crate::adapter_process::{parse_single_json, AdapterProcessOutput};

use super::adapter_invoke_failed;

pub fn protocol_response_from_output(
    adapter_id: &str,
    request_id: &str,
    operation: Operation,
    output: AdapterProcessOutput,
) -> Result<ProtocolResponse, StableError> {
    let context = ProtocolResponseContext::new(adapter_id, request_id, operation, &output);
    let value = parse_protocol_response_json(&context, &output.stdout)?;
    let response = decode_protocol_response(&context, value)?;
    validate_protocol_response_semantics(&context, &response)?;

    Ok(response)
}

struct ProtocolResponseContext<'a> {
    adapter_id: &'a str,
    request_id: &'a str,
    operation: Operation,
    exit_code: Option<i32>,
    stderr: &'a str,
}

impl<'a> ProtocolResponseContext<'a> {
    fn new(
        adapter_id: &'a str,
        request_id: &'a str,
        operation: Operation,
        output: &'a AdapterProcessOutput,
    ) -> Self {
        Self {
            adapter_id,
            request_id,
            operation,
            exit_code: output.exit_code,
            stderr: &output.stderr,
        }
    }

    fn adapter_invoke_failed(&self, reason: impl Into<String>) -> StableError {
        adapter_invoke_failed(
            self.adapter_id,
            reason,
            self.exit_code,
            self.stderr.to_owned(),
        )
    }
}

fn parse_protocol_response_json(
    context: &ProtocolResponseContext<'_>,
    stdout: &str,
) -> Result<Value, StableError> {
    parse_single_json(stdout).map_err(|reason| context.adapter_invoke_failed(reason))
}

fn decode_protocol_response(
    context: &ProtocolResponseContext<'_>,
    value: Value,
) -> Result<ProtocolResponse, StableError> {
    decode_protocol_response_value(value)
        .map_err(|error| context.adapter_invoke_failed(protocol_decode_failure_reason(error)))
}

fn protocol_decode_failure_reason<E>(error: DecodePipelineError<ProtocolResponse, E>) -> String
where
    E: std::fmt::Display,
{
    match error {
        DecodePipelineError::Schema(error) => {
            format!("protocol response schema validation failed: {error}")
        }
        DecodePipelineError::Deserialize(error) => {
            format!("failed to decode protocol response: {error}")
        }
        DecodePipelineError::Semantic { error, .. } => {
            format!("protocol response semantic validation failed: {error}")
        }
    }
}

fn validate_protocol_response_semantics(
    context: &ProtocolResponseContext<'_>,
    response: &ProtocolResponse,
) -> Result<(), StableError> {
    match response {
        ProtocolResponse::Success(success) => validate_success_response(context, success),
        ProtocolResponse::Failure(failure) => validate_failure_response(context, failure),
    }
}

fn validate_success_response(
    context: &ProtocolResponseContext<'_>,
    success: &SuccessResponse,
) -> Result<(), StableError> {
    validate_common_response_fields(context, &success.protocol_version, &success.request_id)?;
    if success.operation != context.operation {
        return Err(
            context.adapter_invoke_failed("response operation does not match invoke request")
        );
    }
    if context.exit_code != Some(0) {
        return Err(context
            .adapter_invoke_failed("adapter returned success response with non-zero exit status"));
    }
    Ok(())
}

fn validate_failure_response(
    context: &ProtocolResponseContext<'_>,
    failure: &FailureResponse,
) -> Result<(), StableError> {
    validate_common_response_fields(context, &failure.protocol_version, &failure.request_id)?;
    if failure.operation != Some(context.operation) {
        return Err(context
            .adapter_invoke_failed("failure response operation does not match invoke request"));
    }
    failure.error.validate_required_details().map_err(|error| {
        context.adapter_invoke_failed(format!(
            "stable error is missing required detail {} for {:?}",
            error.field, error.code
        ))
    })
}

fn validate_common_response_fields(
    context: &ProtocolResponseContext<'_>,
    protocol_version: &str,
    request_id: &str,
) -> Result<(), StableError> {
    if protocol_version != PROTOCOL_VERSION {
        return Err(
            context.adapter_invoke_failed("protocol version does not match current contract")
        );
    }
    if request_id != context.request_id {
        return Err(
            context.adapter_invoke_failed("response request_id does not match invoke request")
        );
    }
    Ok(())
}
