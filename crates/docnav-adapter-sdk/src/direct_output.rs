use std::io::{self, Write};

use docnav_protocol::{ErrorDetails, OperationResult, StableError, StableErrorCode};
use serde::Serialize;

use crate::{emit_diagnostic, AdapterError, AdapterExitCode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectOutputMode {
    Text,
    ReadableJson,
    ProtocolJson,
}

pub trait DirectTextFormatter {
    fn write_text_result<W: Write>(
        &self,
        result: &OperationResult,
        stdout: &mut W,
    ) -> io::Result<()>;
}

pub(crate) fn write_operation_output<T, W, E>(
    result: OperationResult,
    output: DirectOutputMode,
    text_formatter: &T,
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    T: DirectTextFormatter,
    W: Write,
    E: Write,
{
    match output {
        DirectOutputMode::Text => write_text_result(&result, text_formatter, stdout, stderr),
        DirectOutputMode::ReadableJson => write_json_result(&result, stdout, stderr),
        DirectOutputMode::ProtocolJson => unreachable!("protocol-json is handled before dispatch"),
    }
}

pub(crate) fn handler_error<W: Write, E: Write>(
    error: AdapterError,
    output: DirectOutputMode,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let exit_code = error.exit_code();
    let stable = error.error();
    let write_exit = match output {
        DirectOutputMode::Text => write_text_error(stable, stdout, stderr),
        DirectOutputMode::ReadableJson => write_readable_error(stable, stdout, stderr),
        DirectOutputMode::ProtocolJson => unreachable!("protocol-json is handled before dispatch"),
    };
    if write_exit == AdapterExitCode::Success.code() {
        exit_code.code()
    } else {
        write_exit
    }
}

fn write_text_result<T, W, E>(
    result: &OperationResult,
    text_formatter: &T,
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    T: DirectTextFormatter,
    W: Write,
    E: Write,
{
    match text_formatter.write_text_result(result, stdout) {
        Ok(()) => AdapterExitCode::Success.code(),
        Err(error) => {
            let _ = emit_diagnostic(stderr, &format!("failed to write text output: {error}"));
            AdapterExitCode::IoError.code()
        }
    }
}

fn write_json_result<W: Write, E: Write, T: Serialize>(
    result: &T,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    match serde_json::to_writer(stdout, result) {
        Ok(()) => AdapterExitCode::Success.code(),
        Err(error) => {
            let _ = emit_diagnostic(stderr, &format!("failed to write JSON: {error}"));
            AdapterExitCode::IoError.code()
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct ReadableError {
    code: StableErrorCode,
    error: String,
    details: ErrorDetails,
    guidance: Vec<String>,
}

fn write_readable_error<W: Write, E: Write>(
    error: &StableError,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let readable = ReadableError {
        code: error.code,
        error: error.message.clone(),
        details: error.details.clone(),
        guidance: error.guidance.clone().unwrap_or_default(),
    };
    write_json_result(&readable, stdout, stderr)
}

fn write_text_error<W: Write, E: Write>(
    error: &StableError,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let write_result = writeln!(stdout, "error: {}", error_code_label(error.code))
        .and_then(|_| writeln!(stdout, "message: {}", error.message))
        .and_then(|_| {
            if error.details.is_empty() {
                Ok(())
            } else {
                writeln!(stdout, "details: {}", details_label(&error.details))
            }
        })
        .and_then(|_| {
            let Some(guidance) = &error.guidance else {
                return Ok(());
            };
            for item in guidance {
                writeln!(stdout, "guidance: {item}")?;
            }
            Ok(())
        });

    match write_result {
        Ok(()) => AdapterExitCode::Success.code(),
        Err(error) => {
            let _ = emit_diagnostic(stderr, &format!("failed to write text error: {error}"));
            AdapterExitCode::IoError.code()
        }
    }
}

fn error_code_label(code: StableErrorCode) -> String {
    serde_json::to_value(code)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("{code:?}"))
}

fn details_label(details: &ErrorDetails) -> String {
    details
        .iter()
        .map(|(key, value)| {
            value
                .as_str()
                .map(|value| format!("{key}={value}"))
                .unwrap_or_else(|| format!("{key}={value}"))
        })
        .collect::<Vec<_>>()
        .join(", ")
}
