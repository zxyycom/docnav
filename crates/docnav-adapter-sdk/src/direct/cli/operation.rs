use std::io::Write;

use docnav_diagnostics::BoundaryDiagnosticCode;
use docnav_protocol::{
    Document, FindArguments, InfoArguments, Operation, OperationArguments, OutlineArguments,
    ReadArguments, RequestEnvelope, PROTOCOL_VERSION,
};

use super::super::args::{parse_operation_options, DirectOperationOptions};
use super::super::output::{
    append_cli_warnings_to_stderr, handler_error, write_operation_output, DirectOutputMode,
};
use super::super::warnings::DirectCliWarning;
use super::{input_error, DirectCliConfig, DirectCliContext};
use crate::{execute_operation, invoke_once, output::emit_boundary_diagnostic, Adapter};

struct DirectOperationInvocation {
    request: RequestEnvelope,
    output: DirectOutputMode,
    warnings: Vec<DirectCliWarning>,
}

pub(super) fn run_operation<A, W, E>(
    context: &DirectCliContext<'_, A>,
    operation: Operation,
    args: &[String],
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    A: Adapter,
    W: Write,
    E: Write,
{
    let invocation = match operation_invocation(operation, args, &context.config) {
        Ok(invocation) => invocation,
        Err(message) => return input_error(stderr, &message),
    };

    run_operation_request(context.adapter, &invocation, stdout, stderr)
}

fn operation_invocation(
    operation: Operation,
    args: &[String],
    config: &DirectCliConfig<'_>,
) -> Result<DirectOperationInvocation, String> {
    let mut options = parse_operation_options(operation, args, config)?;
    let output = options.output;
    let warnings = std::mem::take(&mut options.warnings);
    let request = operation_request(operation, options, config.request_id)?;

    Ok(DirectOperationInvocation {
        request,
        output,
        warnings,
    })
}

fn operation_request(
    operation: Operation,
    options: DirectOperationOptions,
    request_id: &str,
) -> Result<RequestEnvelope, String> {
    let path = options.path.clone();
    let arguments = match operation {
        Operation::Outline => OperationArguments::Outline(OutlineArguments {
            limit: options.limit,
            page: options.page,
            options: options.protocol_options(),
        }),
        Operation::Read => {
            let Some(ref_id) = options.ref_id.clone() else {
                return Err("read requires --ref <ref>".to_owned());
            };
            OperationArguments::Read(ReadArguments {
                ref_id,
                limit: options.limit,
                page: options.page,
                options: options.protocol_options(),
            })
        }
        Operation::Find => {
            let Some(query) = options.query.clone() else {
                return Err("find requires --query <text>".to_owned());
            };
            OperationArguments::Find(FindArguments {
                query,
                limit: options.limit,
                page: options.page,
                options: options.protocol_options(),
            })
        }
        Operation::Info => OperationArguments::Info(InfoArguments {
            options: options.protocol_options(),
        }),
    };

    Ok(RequestEnvelope {
        protocol_version: PROTOCOL_VERSION.to_owned(),
        request_id: request_id.to_owned(),
        operation,
        document: Document { path },
        arguments,
    })
}

fn run_operation_request<A, W, E>(
    adapter: &A,
    invocation: &DirectOperationInvocation,
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    A: Adapter,
    W: Write,
    E: Write,
{
    if invocation.output == DirectOutputMode::ProtocolJson {
        let exit_code = invoke_request(adapter, &invocation.request, stdout, stderr);
        return append_cli_warnings_to_stderr(exit_code, &invocation.warnings, stderr);
    }

    match execute_operation(adapter, &invocation.request) {
        Ok(result) => write_operation_output(
            result,
            invocation.output,
            &invocation.warnings,
            stdout,
            stderr,
        ),
        Err(error) => handler_error(
            error,
            invocation.output,
            &invocation.warnings,
            stdout,
            stderr,
        ),
    }
}

fn invoke_request<A, W, E>(
    adapter: &A,
    request: &RequestEnvelope,
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    A: Adapter,
    W: Write,
    E: Write,
{
    let input = match serde_json::to_vec(request) {
        Ok(input) => input,
        Err(error) => {
            let _ = emit_boundary_diagnostic(
                stderr,
                BoundaryDiagnosticCode::FailedToSerialize,
                format!("failed to serialize request: {error}"),
            );
            return crate::AdapterExitCode::InternalError.code();
        }
    };
    invoke_once(adapter, input.as_slice(), stdout, stderr)
}
