use std::io::{self, Write};

use docnav_protocol::{ErrorDetails, OperationResult, StableError, StableErrorCode};
use serde::Serialize;

use crate::constants::diagnostics;
use crate::{emit_diagnostic, AdapterError, AdapterExitCode};

use super::warnings::DirectCliWarning;

// Warning 文本字段名来自 readable warning schema；仅用于 text/stderr 诊断展示。
mod warning_text {
    pub(super) const FIELD_IGNORED_TOKENS: &str = "ignored_tokens";
    pub(super) const FIELD_KIND: &str = "kind";
    pub(super) const FIELD_REASON: &str = "reason";
    pub(super) const PREFIX: &str = "warning";
}

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

pub(super) fn write_operation_output<T, W, E>(
    result: OperationResult,
    output: DirectOutputMode,
    text_formatter: &T,
    warnings: &[DirectCliWarning],
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    T: DirectTextFormatter,
    W: Write,
    E: Write,
{
    match output {
        DirectOutputMode::Text => {
            write_text_result(&result, text_formatter, warnings, stdout, stderr)
        }
        DirectOutputMode::ReadableJson => write_readable_result(&result, warnings, stdout, stderr),
        DirectOutputMode::ProtocolJson => unreachable!("protocol-json is handled before dispatch"),
    }
}

pub(super) fn handler_error<W: Write, E: Write>(
    error: AdapterError,
    output: DirectOutputMode,
    warnings: &[DirectCliWarning],
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let exit_code = error.exit_code();
    let stable = error.error();
    let write_exit = match output {
        DirectOutputMode::Text => write_text_error(stable, warnings, stdout, stderr),
        DirectOutputMode::ReadableJson => write_readable_error(stable, warnings, stdout, stderr),
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
    warnings: &[DirectCliWarning],
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    T: DirectTextFormatter,
    W: Write,
    E: Write,
{
    match text_formatter
        .write_text_result(result, stdout)
        .and_then(|_| write_cli_warnings(warnings, stdout))
    {
        Ok(()) => AdapterExitCode::Success.code(),
        Err(error) => {
            let _ = emit_diagnostic(
                stderr,
                &format!("{}: {error}", diagnostics::FAILED_TO_WRITE_TEXT_OUTPUT),
            );
            AdapterExitCode::IoError.code()
        }
    }
}

fn write_readable_result<W: Write, E: Write>(
    result: &OperationResult,
    warnings: &[DirectCliWarning],
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let readable = ReadableWithWarnings { result, warnings };
    write_json_result(&readable, stdout, stderr)
}

fn write_json_result<W: Write, E: Write, T: Serialize>(
    result: &T,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    match serde_json::to_writer(stdout, result) {
        Ok(()) => AdapterExitCode::Success.code(),
        Err(error) => {
            let _ = emit_diagnostic(
                stderr,
                &format!("{}: {error}", diagnostics::FAILED_TO_WRITE_JSON),
            );
            AdapterExitCode::IoError.code()
        }
    }
}

#[derive(Serialize)]
struct ReadableWithWarnings<'a> {
    #[serde(flatten)]
    result: &'a OperationResult,
    #[serde(skip_serializing_if = "warnings_is_empty")]
    warnings: &'a [DirectCliWarning],
}

#[derive(Clone, Debug, Serialize)]
struct ReadableError<'a> {
    code: StableErrorCode,
    error: String,
    details: ErrorDetails,
    guidance: Vec<String>,
    #[serde(skip_serializing_if = "warnings_is_empty")]
    warnings: &'a [DirectCliWarning],
}

fn write_readable_error<W: Write, E: Write>(
    error: &StableError,
    warnings: &[DirectCliWarning],
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let readable = ReadableError {
        code: error.code,
        error: error.message.clone(),
        details: error.details.clone(),
        guidance: error.guidance.clone().unwrap_or_default(),
        warnings,
    };
    write_json_result(&readable, stdout, stderr)
}

fn write_text_error<W: Write, E: Write>(
    error: &StableError,
    warnings: &[DirectCliWarning],
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
        })
        .and_then(|_| write_cli_warnings(warnings, stdout));

    match write_result {
        Ok(()) => AdapterExitCode::Success.code(),
        Err(error) => {
            let _ = emit_diagnostic(
                stderr,
                &format!("{}: {error}", diagnostics::FAILED_TO_WRITE_TEXT_ERROR),
            );
            AdapterExitCode::IoError.code()
        }
    }
}

pub(super) fn append_cli_warnings_to_stderr<W: Write>(
    exit_code: i32,
    warnings: &[DirectCliWarning],
    stderr: &mut W,
) -> i32 {
    match write_cli_warnings(warnings, stderr) {
        Ok(()) => exit_code,
        Err(error) => {
            let _ = emit_diagnostic(
                stderr,
                &format!("{}: {error}", diagnostics::FAILED_TO_WRITE_CLI_WARNING),
            );
            AdapterExitCode::IoError.code()
        }
    }
}

fn write_cli_warnings<W: Write>(warnings: &[DirectCliWarning], writer: &mut W) -> io::Result<()> {
    for warning in warnings {
        let ignored_tokens =
            serde_json::to_string(&warning.ignored_tokens).map_err(io::Error::other)?;
        writeln!(
            writer,
            "{}: {}={}, {}={}, {}={}",
            warning_text::PREFIX,
            warning_text::FIELD_IGNORED_TOKENS,
            ignored_tokens,
            warning_text::FIELD_KIND,
            warning.kind.as_str(),
            warning_text::FIELD_REASON,
            warning.reason
        )?;
    }
    Ok(())
}

fn warnings_is_empty(warnings: &&[DirectCliWarning]) -> bool {
    warnings.is_empty()
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
