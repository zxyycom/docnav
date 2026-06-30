use std::io::Write;

use docnav_diagnostics::{
    typed_codes, AdapterConfigSourceDetails, BoundaryDiagnosticCode, DiagnosticRecordDraft,
    DiagnosticSource, FieldReasonDetails,
};
use docnav_protocol::{
    Document, FindArguments, InfoArguments, Operation, OperationArguments, OutlineArguments,
    ReadArguments, RequestEnvelope, PROTOCOL_VERSION,
};
use docnav_standard_parameters::StandardParameterConfigSourceIssue;

use super::super::args::{parse_operation_options, DirectCliInputError, DirectOperationOptions};
use super::super::output::{handler_error, write_operation_output, DirectOutputMode};
use super::{DirectCliConfig, DirectCliContext};
use crate::{
    execute_operation, invoke_once, output::emit_boundary_diagnostic, Adapter, AdapterError,
};

struct DirectOperationInvocation {
    request: RequestEnvelope,
    output: DirectOutputMode,
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
        Err(error) => return operation_input_error(operation, args, &error, stdout, stderr),
    };

    run_operation_request(context.adapter, &invocation, stdout, stderr)
}

fn operation_invocation(
    operation: Operation,
    args: &[String],
    config: &DirectCliConfig<'_>,
) -> Result<DirectOperationInvocation, DirectCliInputError> {
    let options = parse_operation_options(operation, args, config)?;
    let output = options.output;
    let request = operation_request(operation, options, config.request_id)?;

    Ok(DirectOperationInvocation { request, output })
}

fn operation_request(
    operation: Operation,
    options: DirectOperationOptions,
    request_id: &str,
) -> Result<RequestEnvelope, DirectCliInputError> {
    let path = options.path.clone();
    let arguments = match operation {
        Operation::Outline => OperationArguments::Outline(OutlineArguments {
            limit: options.limit,
            page: options.page,
            options: options.protocol_options(),
        }),
        Operation::Read => {
            let Some(ref_id) = options.ref_id.clone() else {
                return Err("read requires --ref <ref>".into());
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
                return Err("find requires --query <text>".into());
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
        return invoke_request(adapter, &invocation.request, stdout, stderr);
    }

    match execute_operation(adapter, &invocation.request) {
        Ok(result) => write_operation_output(result, invocation.output, stdout, stderr),
        Err(error) => handler_error(
            error,
            invocation.output,
            Some(invocation.request.operation),
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

fn operation_input_error<W: Write, E: Write>(
    operation: Operation,
    args: &[String],
    error: &DirectCliInputError,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    handler_error(
        input_adapter_error(error),
        requested_output(args),
        Some(operation),
        stdout,
        stderr,
    )
}

fn requested_output(args: &[String]) -> DirectOutputMode {
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if let Some(value) = token.strip_prefix("--output=") {
            return output_mode_from_value(value);
        }
        if token == "--output" {
            return args
                .get(index + 1)
                .map_or(DirectOutputMode::ReadableView, |value| {
                    output_mode_from_value(value)
                });
        }
        index += 1;
    }
    DirectOutputMode::ReadableView
}

fn output_mode_from_value(value: &str) -> DirectOutputMode {
    match value {
        "readable-json" => DirectOutputMode::ReadableJson,
        "protocol-json" => DirectOutputMode::ProtocolJson,
        _ => DirectOutputMode::ReadableView,
    }
}

fn input_adapter_error(error: &DirectCliInputError) -> AdapterError {
    AdapterError::new(DiagnosticRecordDraft::new::<
        typed_codes::protocol::InvalidRequest,
    >(
        error.to_string(),
        input_details(error),
        DiagnosticSource::with_stage("docnav-adapter-sdk", "direct-cli"),
    ))
}

fn input_details(error: &DirectCliInputError) -> FieldReasonDetails {
    match error {
        DirectCliInputError::Message(message) => {
            let (reason, received) = argv_reason(message);
            let mut details = FieldReasonDetails::new("argv", reason);
            details.received = received;
            details.accepted = Some(vec!["valid document operation arguments".to_owned()]);
            details
        }
        DirectCliInputError::ConfigSource(issue) => config_source_details(issue),
    }
}

fn argv_reason(message: &str) -> (&'static str, Option<String>) {
    if let Some(token) = message.strip_prefix("unknown argument ") {
        return ("unknown_argument", Some(token.to_owned()));
    }
    if let Some(token) = message.strip_prefix("extra positional argument ") {
        return ("extra_positional", Some(token.to_owned()));
    }
    if let Some((tokens, _)) = message.split_once(" is not used by ") {
        return ("operation_inapplicable_argument", Some(tokens.to_owned()));
    }
    ("invalid_argument", None)
}

fn config_source_details(issue: &StandardParameterConfigSourceIssue) -> FieldReasonDetails {
    let field = match issue.source_level.as_str() {
        "user" => "user_config_path",
        _ => "project_config_path",
    };
    let mut details = FieldReasonDetails::new(field, issue.reason_code.clone());
    details.path = Some(issue.path.clone());
    details.received = Some(issue.path.clone());
    details.config_issues = Some(vec![AdapterConfigSourceDetails::new(
        &issue.source_level,
        &issue.path_origin,
        &issue.path,
        &issue.reason_code,
    )]);
    details
}
