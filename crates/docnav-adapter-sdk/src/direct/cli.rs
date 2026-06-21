use std::io::{Read, Write};

use docnav_protocol::{
    Document, FindArguments, InfoArguments, Operation, OperationArguments, OutlineArguments,
    ReadArguments, RequestEnvelope, PROTOCOL_VERSION,
};

use super::args::{
    command_names, direct_cli_command, parse_operation_options, parse_probe,
    parse_protocol_only_options, DirectOperationOptions,
};
use super::native_options::NativeOptionSpec;
use super::output::{
    append_cli_warnings_to_stderr, handler_error, write_operation_output, DirectOutputMode,
};
use super::warnings::DirectCliWarning;
use crate::{emit_diagnostic, execute_operation, invoke_once, run_command, Adapter, SdkCommand};

pub struct DirectCliConfig<'a> {
    pub program_name: &'static str,
    pub usage: &'a str,
    pub request_id: &'a str,
    pub default_limit_chars: u32,
    pub native_options: &'a [NativeOptionSpec],
}

pub fn run_direct_cli<A, I, R, W, E>(
    adapter: &A,
    args: I,
    stdin: R,
    mut stdout: W,
    mut stderr: E,
    config: DirectCliConfig<'_>,
) -> i32
where
    A: Adapter,
    I: IntoIterator<Item = String>,
    R: Read,
    W: Write,
    E: Write,
{
    let args: Vec<String> = args.into_iter().collect();
    if let Some(help) = help_text(
        &args,
        config.program_name,
        config.native_options,
        config.default_limit_chars,
    ) {
        return write_help(&help, &mut stdout, &mut stderr);
    }

    let Some(command) = args.first().map(String::as_str) else {
        return usage(&mut stderr, config.usage);
    };
    if !is_known_command(
        command,
        config.program_name,
        config.native_options,
        config.default_limit_chars,
    ) {
        return usage(&mut stderr, config.usage);
    }

    let context = DirectCliContext { adapter, config };
    let command = DirectCommandInvocation {
        name: command,
        args: &args[1..],
    };
    run_direct_command(&context, command, stdin, &mut stdout, &mut stderr)
}

struct DirectCliContext<'a, A> {
    adapter: &'a A,
    config: DirectCliConfig<'a>,
}

struct DirectOperationInvocation {
    request: RequestEnvelope,
    output: DirectOutputMode,
    warnings: Vec<DirectCliWarning>,
}

struct DirectCommandInvocation<'a> {
    name: &'a str,
    args: &'a [String],
}

fn run_direct_command<A, R, W, E>(
    context: &DirectCliContext<'_, A>,
    command: DirectCommandInvocation<'_>,
    stdin: R,
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    A: Adapter,
    R: Read,
    W: Write,
    E: Write,
{
    match command.name {
        command_names::MANIFEST => run_manifest_command(context, command.args, stdout, stderr),
        command_names::PROBE => run_probe_command(context, command.args, stdout, stderr),
        command_names::INVOKE => run_invoke_command(context, command.args, stdin, stdout, stderr),
        command_names::OUTLINE => {
            run_operation(context, Operation::Outline, command.args, stdout, stderr)
        }
        command_names::READ => {
            run_operation(context, Operation::Read, command.args, stdout, stderr)
        }
        command_names::FIND => {
            run_operation(context, Operation::Find, command.args, stdout, stderr)
        }
        command_names::INFO => {
            run_operation(context, Operation::Info, command.args, stdout, stderr)
        }
        _ => unreachable!("known direct CLI commands are handled above"),
    }
}

fn run_manifest_command<A, W, E>(
    context: &DirectCliContext<'_, A>,
    args: &[String],
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    A: Adapter,
    W: Write,
    E: Write,
{
    match parse_protocol_only_options(args, context.config.native_options) {
        Ok(warnings) => {
            let exit_code = run_command(
                context.adapter,
                SdkCommand::Manifest,
                std::io::empty(),
                &mut *stdout,
                &mut *stderr,
            );
            append_cli_warnings_to_stderr(exit_code, &warnings, stderr)
        }
        Err(message) => input_error(stderr, &message),
    }
}

fn run_probe_command<A, W, E>(
    context: &DirectCliContext<'_, A>,
    args: &[String],
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    A: Adapter,
    W: Write,
    E: Write,
{
    match parse_probe(args, context.config.native_options) {
        Ok(options) => {
            let exit_code = run_command(
                context.adapter,
                SdkCommand::Probe { path: options.path },
                std::io::empty(),
                &mut *stdout,
                &mut *stderr,
            );
            append_cli_warnings_to_stderr(exit_code, &options.warnings, stderr)
        }
        Err(message) => input_error(stderr, &message),
    }
}

fn run_invoke_command<A, R, W, E>(
    context: &DirectCliContext<'_, A>,
    args: &[String],
    stdin: R,
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    A: Adapter,
    R: Read,
    W: Write,
    E: Write,
{
    if !args.is_empty() {
        return input_error(stderr, "invoke does not accept positional arguments");
    }
    run_command(context.adapter, SdkCommand::Invoke, stdin, stdout, stderr)
}

fn help_text(
    args: &[String],
    program_name: &'static str,
    native_options: &[NativeOptionSpec],
    default_limit_chars: u32,
) -> Option<String> {
    if !args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return None;
    }
    let mut root = direct_cli_command(program_name, native_options, default_limit_chars);
    let Some(first) = args.first().map(String::as_str) else {
        return Some(root.render_long_help().to_string());
    };
    if first == "--help" || first == "-h" {
        return Some(root.render_long_help().to_string());
    }
    root.find_subcommand_mut(first)
        .map(|command| command.render_long_help().to_string())
        .or_else(|| Some(root.render_long_help().to_string()))
}

fn is_known_command(
    command: &str,
    program_name: &'static str,
    native_options: &[NativeOptionSpec],
    default_limit_chars: u32,
) -> bool {
    direct_cli_command(program_name, native_options, default_limit_chars)
        .find_subcommand(command)
        .is_some()
}

fn run_operation<A, W, E>(
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
    let mut options = parse_operation_options(
        operation,
        args,
        config.default_limit_chars,
        config.native_options,
    )?;
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
            let _ = emit_diagnostic(stderr, &format!("failed to serialize request: {error}"));
            return crate::AdapterExitCode::InternalError.code();
        }
    };
    invoke_once(adapter, input.as_slice(), stdout, stderr)
}

fn usage<E: Write>(stderr: &mut E, message: &str) -> i32 {
    input_error(stderr, message)
}

fn write_help<W: Write, E: Write>(help: &str, stdout: &mut W, stderr: &mut E) -> i32 {
    match writeln!(stdout, "{help}") {
        Ok(()) => crate::AdapterExitCode::Success.code(),
        Err(error) => {
            let _ = emit_diagnostic(stderr, &format!("failed to write help output: {error}"));
            crate::AdapterExitCode::IoError.code()
        }
    }
}

fn input_error<E: Write>(stderr: &mut E, message: &str) -> i32 {
    let _ = emit_diagnostic(stderr, message);
    crate::AdapterExitCode::ProtocolError.code()
}
