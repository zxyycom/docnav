use docnav_diagnostics::BoundaryDiagnosticCode;
use docnav_protocol::{
    decode_protocol_request_value, extract_request_context_from_value, DecodePipelineError,
    FailureResponse, Operation, OperationArguments, OperationResult, ProtocolResponse,
    RequestEnvelope, StableError, PROTOCOL_VERSION, UNKNOWN_REQUEST_ID,
};
use serde_json::Value;
use std::io::{Read, Write};

use crate::boundary::validated_manifest;
use crate::constants::{diagnostics, fields};
use crate::output::{
    emit_boundary_diagnostic, write_adapter_boundary_error, write_protocol_response,
};
use crate::standard_parameters::standardize_invoke_request;
use crate::{Adapter, AdapterExitCode, AdapterResult};

struct InvokeFailure {
    response: ProtocolResponse,
    diagnostic: Option<(BoundaryDiagnosticCode, String)>,
    exit_code: AdapterExitCode,
}

struct InvokeResponse {
    response: ProtocolResponse,
    exit_code: AdapterExitCode,
}

type InvokeResult<T> = Result<T, Box<InvokeFailure>>;

pub fn invoke_once<A, R, W, E>(adapter: &A, mut stdin: R, mut stdout: W, mut stderr: E) -> i32
where
    A: Adapter,
    R: Read,
    W: Write,
    E: Write,
{
    if let Err(exit_code) = validate_manifest_for_invoke(adapter, &mut stderr) {
        return exit_code;
    }

    let request = match read_and_decode_request(&mut stdin) {
        Ok(request) => request,
        Err(failure) => return write_invoke_failure(*failure, &mut stdout, &mut stderr),
    };
    let request = match standardize_invoke_request(&request) {
        Ok(request) => request,
        Err(error) => {
            return write_invoke_failure(
                InvokeFailure {
                    response: ProtocolResponse::failure_for_request(&request, error),
                    diagnostic: None,
                    exit_code: AdapterExitCode::ProtocolError,
                },
                &mut stdout,
                &mut stderr,
            )
        }
    };

    let response = execute_request(adapter, &request);
    write_protocol_response(
        &response.response,
        &mut stdout,
        &mut stderr,
        response.exit_code,
    )
}

fn validate_manifest_for_invoke<A, E>(adapter: &A, stderr: &mut E) -> Result<(), i32>
where
    A: Adapter,
    E: Write,
{
    validated_manifest(adapter)
        .map(|_| ())
        .map_err(|error| write_adapter_boundary_error(&error, stderr))
}

fn read_and_decode_request<R>(stdin: &mut R) -> InvokeResult<RequestEnvelope>
where
    R: Read,
{
    let input = read_request_text(stdin)?;
    let request_value = parse_request_json(&input)?;
    decode_request_value(request_value)
}

fn read_request_text<R>(stdin: &mut R) -> InvokeResult<String>
where
    R: Read,
{
    let mut input = String::new();
    if let Err(error) = stdin.read_to_string(&mut input) {
        return Err(Box::new(read_request_failure(error)));
    }

    Ok(input)
}

fn read_request_failure(error: std::io::Error) -> InvokeFailure {
    let reason = error.to_string();
    InvokeFailure {
        response: ProtocolResponse::Failure(FailureResponse::unparsed(
            StableError::invalid_request(fields::REQUEST, reason),
        )),
        diagnostic: Some((
            BoundaryDiagnosticCode::FailedToReadRequest,
            format!("{}: {error}", diagnostics::FAILED_TO_READ_REQUEST),
        )),
        exit_code: AdapterExitCode::IoError,
    }
}

fn parse_request_json(input: &str) -> InvokeResult<Value> {
    serde_json::from_str(input).map_err(|error| {
        let reason = error.to_string();
        Box::new(InvokeFailure {
            response: ProtocolResponse::failure(
                PROTOCOL_VERSION,
                UNKNOWN_REQUEST_ID,
                None,
                StableError::invalid_request(fields::REQUEST, reason),
            ),
            diagnostic: Some((
                BoundaryDiagnosticCode::InvalidRequestJson,
                format!("{}: {error}", diagnostics::INVALID_REQUEST_JSON),
            )),
            exit_code: AdapterExitCode::ProtocolError,
        })
    })
}

fn decode_request_value(request_value: Value) -> InvokeResult<RequestEnvelope> {
    let context = extract_request_context_from_value(&request_value);
    let request_id = context
        .request_id
        .clone()
        .unwrap_or_else(|| UNKNOWN_REQUEST_ID.to_owned());

    let request = match decode_protocol_request_value(request_value) {
        Ok(request) => request,
        Err(DecodePipelineError::Schema(error)) => {
            let reason = error.to_string();
            return Err(Box::new(InvokeFailure {
                response: ProtocolResponse::failure(
                    PROTOCOL_VERSION,
                    request_id.clone(),
                    context.operation,
                    StableError::invalid_request(fields::REQUEST, reason),
                ),
                diagnostic: Some((
                    BoundaryDiagnosticCode::RequestSchemaValidationFailed,
                    format!("{}: {error}", diagnostics::REQUEST_SCHEMA_VALIDATION_FAILED),
                )),
                exit_code: AdapterExitCode::ProtocolError,
            }));
        }
        Err(DecodePipelineError::Deserialize(error)) => {
            let reason = error.to_string();
            return Err(Box::new(InvokeFailure {
                response: ProtocolResponse::failure(
                    PROTOCOL_VERSION,
                    request_id,
                    context.operation,
                    StableError::invalid_request(fields::REQUEST, reason),
                ),
                diagnostic: Some((
                    BoundaryDiagnosticCode::RequestDeserializationFailed,
                    format!("{}: {error}", diagnostics::REQUEST_DESERIALIZATION_FAILED),
                )),
                exit_code: AdapterExitCode::ProtocolError,
            }));
        }
        Err(DecodePipelineError::Semantic { value, error }) => {
            return Err(Box::new(InvokeFailure {
                response: ProtocolResponse::failure_for_request(&value, error),
                diagnostic: None,
                exit_code: AdapterExitCode::ProtocolError,
            }));
        }
    };
    Ok(request)
}

fn execute_request<A>(adapter: &A, request: &RequestEnvelope) -> InvokeResponse
where
    A: Adapter,
{
    match execute_operation(adapter, request) {
        Ok(result) => {
            let response = ProtocolResponse::success(
                request.protocol_version.clone(),
                request.request_id.clone(),
                result,
            );
            InvokeResponse {
                response,
                exit_code: AdapterExitCode::Success,
            }
        }
        Err(error) => {
            let exit_code = error.exit_code();
            let response = ProtocolResponse::failure_for_request(request, error.into_error());
            InvokeResponse {
                response,
                exit_code,
            }
        }
    }
}

fn write_invoke_failure<W, E>(failure: InvokeFailure, stdout: &mut W, stderr: &mut E) -> i32
where
    W: Write,
    E: Write,
{
    if let Some((code, diagnostic)) = failure.diagnostic {
        let _ = emit_boundary_diagnostic(stderr, code, diagnostic);
    }
    write_protocol_response(&failure.response, stdout, stderr, failure.exit_code)
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
