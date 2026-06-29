use std::io::{Read, Write};
use std::path::Path;

use docnav_diagnostics::BoundaryDiagnosticCode;

mod commands;
mod help;
mod operation;

use super::native_options::NativeOptionSpec;
use crate::{output::emit_boundary_diagnostic, Adapter};

use commands::run_direct_command;
use help::{help_text, is_known_command};

pub struct DirectCliConfig<'a> {
    pub adapter_id: &'static str,
    pub program_name: &'static str,
    pub usage: &'a str,
    pub request_id: &'a str,
    pub default_limit: u32,
    pub default_user_config_dir: Option<&'a Path>,
    pub native_options: &'a [NativeOptionSpec],
}

pub struct DirectCliInvocation<'a, A, I, R, W, E> {
    pub adapter: &'a A,
    pub args: I,
    pub stdin: R,
    pub stdout: W,
    pub stderr: E,
    pub config: DirectCliConfig<'a>,
}

pub fn run_direct_cli<A, I, R, W, E>(invocation: DirectCliInvocation<'_, A, I, R, W, E>) -> i32
where
    A: Adapter,
    I: IntoIterator<Item = String>,
    R: Read,
    W: Write,
    E: Write,
{
    let DirectCliInvocation {
        adapter,
        args,
        stdin,
        mut stdout,
        mut stderr,
        config,
    } = invocation;
    let args: Vec<String> = args.into_iter().collect();
    if let Some(help) = help_text(
        &args,
        config.program_name,
        config.native_options,
        config.default_limit,
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
        config.default_limit,
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

pub(super) struct DirectCliContext<'a, A> {
    adapter: &'a A,
    config: DirectCliConfig<'a>,
}

pub(super) struct DirectCommandInvocation<'a> {
    name: &'a str,
    args: &'a [String],
}

fn usage<E: Write>(stderr: &mut E, message: &str) -> i32 {
    input_error(stderr, message)
}

fn write_help<W: Write, E: Write>(help: &str, stdout: &mut W, stderr: &mut E) -> i32 {
    match writeln!(stdout, "{help}") {
        Ok(()) => crate::AdapterExitCode::Success.code(),
        Err(error) => {
            let _ = emit_boundary_diagnostic(
                stderr,
                BoundaryDiagnosticCode::FailedToWriteReadableView,
                format!("failed to write help output: {error}"),
            );
            crate::AdapterExitCode::IoError.code()
        }
    }
}

pub(super) fn input_error<E: Write>(stderr: &mut E, message: &str) -> i32 {
    let _ = emit_boundary_diagnostic(
        stderr,
        BoundaryDiagnosticCode::InvalidRequestJson,
        message.to_owned(),
    );
    crate::AdapterExitCode::ProtocolError.code()
}
