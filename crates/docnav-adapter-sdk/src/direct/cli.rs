use std::io::{Read, Write};

use docnav_protocol::{
    Document, FindArguments, InfoArguments, Operation, OperationArguments, OutlineArguments,
    ReadArguments, RequestEnvelope, PROTOCOL_VERSION,
};

use super::args::{
    parse_operation_options, parse_probe, parse_protocol_only_output, NativeOptionSpec,
};
use super::output::{handler_error, write_operation_output, DirectOutputMode, DirectTextFormatter};
use crate::{emit_diagnostic, execute_operation, invoke_once, run_command, Adapter, SdkCommand};

pub struct DirectCliConfig<'a, T> {
    pub usage: &'a str,
    pub request_id: &'a str,
    pub default_limit_chars: u32,
    pub native_options: &'a [NativeOptionSpec],
    pub text_formatter: T,
}

pub fn run_direct_cli<A, I, R, W, E, T>(
    adapter: &A,
    args: I,
    stdin: R,
    mut stdout: W,
    mut stderr: E,
    config: DirectCliConfig<'_, T>,
) -> i32
where
    A: Adapter,
    I: IntoIterator<Item = String>,
    R: Read,
    W: Write,
    E: Write,
    T: DirectTextFormatter,
{
    let args: Vec<String> = args.into_iter().collect();
    let Some(command) = args.first().map(String::as_str) else {
        return usage(&mut stderr, config.usage);
    };

    match command {
        "manifest" => match parse_protocol_only_output(&args[1..]) {
            Ok(()) => run_command(
                adapter,
                SdkCommand::Manifest,
                std::io::empty(),
                stdout,
                stderr,
            ),
            Err(message) => input_error(&mut stderr, &message),
        },
        "probe" => match parse_probe(&args[1..]) {
            Ok(path) => run_command(
                adapter,
                SdkCommand::Probe { path },
                std::io::empty(),
                stdout,
                stderr,
            ),
            Err(message) => input_error(&mut stderr, &message),
        },
        "invoke" => {
            if args.len() != 1 {
                return input_error(&mut stderr, "invoke does not accept positional arguments");
            }
            run_command(adapter, SdkCommand::Invoke, stdin, stdout, stderr)
        }
        "outline" => run_operation(
            adapter,
            Operation::Outline,
            &args[1..],
            &config,
            &mut stdout,
            &mut stderr,
        ),
        "read" => run_operation(
            adapter,
            Operation::Read,
            &args[1..],
            &config,
            &mut stdout,
            &mut stderr,
        ),
        "find" => run_operation(
            adapter,
            Operation::Find,
            &args[1..],
            &config,
            &mut stdout,
            &mut stderr,
        ),
        "info" => run_operation(
            adapter,
            Operation::Info,
            &args[1..],
            &config,
            &mut stdout,
            &mut stderr,
        ),
        _ => usage(&mut stderr, config.usage),
    }
}

fn run_operation<A, W, E, T>(
    adapter: &A,
    operation: Operation,
    args: &[String],
    config: &DirectCliConfig<'_, T>,
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    A: Adapter,
    W: Write,
    E: Write,
    T: DirectTextFormatter,
{
    let options = match parse_operation_options(
        operation,
        args,
        config.default_limit_chars,
        config.native_options,
    ) {
        Ok(options) => options,
        Err(message) => return input_error(stderr, &message),
    };
    let output = options.output;
    let request = match operation_request(operation, options, config.request_id) {
        Ok(request) => request,
        Err(message) => return input_error(stderr, &message),
    };

    run_operation_request(
        adapter,
        &request,
        output,
        &config.text_formatter,
        stdout,
        stderr,
    )
}

fn operation_request(
    operation: Operation,
    options: super::DirectOperationOptions,
    request_id: &str,
) -> Result<RequestEnvelope, String> {
    let path = options.path.clone();
    let arguments = match operation {
        Operation::Outline => OperationArguments::Outline(OutlineArguments {
            limit_chars: options.limit_chars,
            page: options.page,
            options: options.protocol_options(),
        }),
        Operation::Read => {
            let Some(ref_id) = options.ref_id.clone() else {
                return Err("read requires --ref <ref>".to_owned());
            };
            OperationArguments::Read(ReadArguments {
                ref_id,
                limit_chars: options.limit_chars,
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
                limit_chars: options.limit_chars,
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

fn run_operation_request<A, T, W, E>(
    adapter: &A,
    request: &RequestEnvelope,
    output: DirectOutputMode,
    text_formatter: &T,
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    A: Adapter,
    T: DirectTextFormatter,
    W: Write,
    E: Write,
{
    if output == DirectOutputMode::ProtocolJson {
        return invoke_request(adapter, request, stdout, stderr);
    }

    match execute_operation(adapter, request) {
        Ok(result) => write_operation_output(result, output, text_formatter, stdout, stderr),
        Err(error) => handler_error(error, output, stdout, stderr),
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
            let _ = emit_diagnostic(stderr, &format!("failed to serialize request: {error}"));
            return crate::AdapterExitCode::InternalError.code();
        }
    };
    invoke_once(adapter, input.as_slice(), stdout, stderr)
}

fn usage<E: Write>(stderr: &mut E, message: &str) -> i32 {
    input_error(stderr, message)
}

fn input_error<E: Write>(stderr: &mut E, message: &str) -> i32 {
    let _ = emit_diagnostic(stderr, message);
    crate::AdapterExitCode::ProtocolError.code()
}
