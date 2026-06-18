use docnav_protocol::{
    decode_protocol_request_value, extract_request_context_from_value, DecodePipelineError,
    FailureResponse, Operation, OperationArguments, OperationResult, ProtocolResponse,
    RequestEnvelope, StableError, PROTOCOL_VERSION, UNKNOWN_REQUEST_ID,
};
use serde_json::Value;
use std::io::{Read, Write};

use crate::boundary::validated_manifest;
use crate::constants::{diagnostics, fields};
use crate::output::{emit_diagnostic, write_adapter_boundary_error, write_protocol_response};
use crate::{Adapter, AdapterExitCode, AdapterResult};

pub fn invoke_once<A, R, W, E>(adapter: &A, mut stdin: R, mut stdout: W, mut stderr: E) -> i32
where
    A: Adapter,
    R: Read,
    W: Write,
    E: Write,
{
    match validated_manifest(adapter) {
        Ok(_) => {}
        Err(error) => return write_adapter_boundary_error(&error, &mut stderr),
    };
    let mut input = String::new();
    if let Err(error) = stdin.read_to_string(&mut input) {
        let response = ProtocolResponse::Failure(FailureResponse::unparsed(
            StableError::invalid_request(fields::REQUEST, error.to_string()),
        ));
        let _ = emit_diagnostic(
            &mut stderr,
            &format!("{}: {error}", diagnostics::FAILED_TO_READ_REQUEST),
        );
        return write_protocol_response(
            &response,
            &mut stdout,
            &mut stderr,
            AdapterExitCode::IoError,
        );
    }

    let request_value: Value = match serde_json::from_str(&input) {
        Ok(value) => value,
        Err(error) => {
            let response = ProtocolResponse::failure(
                PROTOCOL_VERSION,
                UNKNOWN_REQUEST_ID,
                None,
                StableError::invalid_request(fields::REQUEST, error.to_string()),
            );
            let _ = emit_diagnostic(
                &mut stderr,
                &format!("{}: {error}", diagnostics::INVALID_REQUEST_JSON),
            );
            return write_protocol_response(
                &response,
                &mut stdout,
                &mut stderr,
                AdapterExitCode::ProtocolError,
            );
        }
    };

    let context = extract_request_context_from_value(&request_value);
    let request_id = context
        .request_id
        .clone()
        .unwrap_or_else(|| UNKNOWN_REQUEST_ID.to_owned());

    let request = match decode_protocol_request_value(request_value) {
        Ok(request) => request,
        Err(DecodePipelineError::Schema(error)) => {
            let response = ProtocolResponse::failure(
                PROTOCOL_VERSION,
                request_id.clone(),
                context.operation,
                StableError::invalid_request(fields::REQUEST, error.to_string()),
            );
            let _ = emit_diagnostic(
                &mut stderr,
                &format!("{}: {error}", diagnostics::REQUEST_SCHEMA_VALIDATION_FAILED),
            );
            return write_protocol_response(
                &response,
                &mut stdout,
                &mut stderr,
                AdapterExitCode::ProtocolError,
            );
        }
        Err(DecodePipelineError::Deserialize(error)) => {
            let response = ProtocolResponse::failure(
                PROTOCOL_VERSION,
                request_id,
                context.operation,
                StableError::invalid_request(fields::REQUEST, error.to_string()),
            );
            let _ = emit_diagnostic(
                &mut stderr,
                &format!("{}: {error}", diagnostics::REQUEST_DESERIALIZATION_FAILED),
            );
            return write_protocol_response(
                &response,
                &mut stdout,
                &mut stderr,
                AdapterExitCode::ProtocolError,
            );
        }
        Err(DecodePipelineError::Semantic { value, error }) => {
            let response = ProtocolResponse::failure_for_request(&value, error);
            return write_protocol_response(
                &response,
                &mut stdout,
                &mut stderr,
                AdapterExitCode::ProtocolError,
            );
        }
    };

    match execute_operation(adapter, &request) {
        Ok(result) => {
            let response = ProtocolResponse::success(
                request.protocol_version.clone(),
                request.request_id.clone(),
                result,
            );
            write_protocol_response(
                &response,
                &mut stdout,
                &mut stderr,
                AdapterExitCode::Success,
            )
        }
        Err(error) => {
            let exit_code = error.exit_code();
            let response = ProtocolResponse::failure_for_request(&request, error.into_error());
            write_protocol_response(&response, &mut stdout, &mut stderr, exit_code)
        }
    }
}

pub fn execute_operation<A: Adapter>(
    adapter: &A,
    request: &RequestEnvelope,
) -> AdapterResult<OperationResult> {
    match (&request.operation, &request.arguments) {
        (Operation::Outline, OperationArguments::Outline(arguments)) => adapter
            .outline(request, arguments)
            .map(OperationResult::Outline),
        (Operation::Read, OperationArguments::Read(arguments)) => {
            adapter.read(request, arguments).map(OperationResult::Read)
        }
        (Operation::Find, OperationArguments::Find(arguments)) => {
            adapter.find(request, arguments).map(OperationResult::Find)
        }
        (Operation::Info, OperationArguments::Info(arguments)) => {
            adapter.info(request, arguments).map(OperationResult::Info)
        }
        _ => Err(StableError::invalid_request(
            fields::ARGUMENTS,
            format!("arguments do not match operation {}", request.operation),
        )
        .into()),
    }
}
