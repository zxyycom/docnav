use std::io::{Read, Write};

use docnav_protocol::Operation;

use super::super::args::{command_names, parse_probe, parse_protocol_only_options};
use super::super::output::append_cli_warnings_to_stderr;
use super::operation::run_operation;
use super::{input_error, DirectCliContext, DirectCommandInvocation};
use crate::{invoke::invoke_once_with_default_limit_chars, run_command, Adapter, SdkCommand};

pub(super) fn run_direct_command<A, R, W, E>(
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
    invoke_once_with_default_limit_chars(
        context.adapter,
        context.config.default_limit_chars,
        stdin,
        stdout,
        stderr,
    )
}
