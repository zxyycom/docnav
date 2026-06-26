use std::io::{self, Write};

use docnav_diagnostics::write_warning_text_lines;
use docnav_output::{
    write_document_error, write_document_result, DocumentOutputError, DocumentOutputMode,
    DocumentOutputOptions, ProtocolOutputContext,
};
use docnav_protocol::{OperationResult, PROTOCOL_VERSION};

use crate::constants::diagnostics;
use crate::{emit_diagnostic, AdapterError, AdapterExitCode};

use super::warnings::DirectCliWarning;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectOutputMode {
    /// 默认文档输出：readable-view（pretty JSON header + block sections）。
    ReadableView,
    ReadableJson,
    ProtocolJson,
}

pub(super) fn write_operation_output<W, E>(
    result: OperationResult,
    output: DirectOutputMode,
    warnings: &[DirectCliWarning],
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    W: Write,
    E: Write,
{
    match output {
        DirectOutputMode::ReadableView | DirectOutputMode::ReadableJson => {
            match write_document_result(
                &result,
                "adapter-direct",
                DocumentOutputOptions::new(document_output_mode(output), warnings),
                stdout,
                stderr,
            ) {
                Ok(()) => AdapterExitCode::Success.code(),
                Err(error) => document_output_error(error, stderr),
            }
        }
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
        DirectOutputMode::ReadableView | DirectOutputMode::ReadableJson => {
            let protocol = ProtocolOutputContext::new(PROTOCOL_VERSION, "adapter-direct", None);
            match write_document_error(
                stable,
                protocol,
                DocumentOutputOptions::new(document_output_mode(output), warnings),
                stdout,
                stderr,
            ) {
                Ok(()) => AdapterExitCode::Success.code(),
                Err(error) => document_output_error(error, stderr),
            }
        }
        DirectOutputMode::ProtocolJson => unreachable!("protocol-json is handled before dispatch"),
    };
    if write_exit == AdapterExitCode::Success.code() {
        exit_code.code()
    } else {
        write_exit
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

fn document_output_mode(output: DirectOutputMode) -> DocumentOutputMode {
    match output {
        DirectOutputMode::ReadableView => DocumentOutputMode::ReadableView,
        DirectOutputMode::ReadableJson => DocumentOutputMode::ReadableJson,
        DirectOutputMode::ProtocolJson => DocumentOutputMode::ProtocolJson,
    }
}

fn document_output_error<E: Write>(error: DocumentOutputError, stderr: &mut E) -> i32 {
    match error {
        DocumentOutputError::ReadableViewRender(error) => {
            let _ = emit_diagnostic(stderr, &format!("readable_view_render_failed: {error}"));
            AdapterExitCode::InternalError.code()
        }
        DocumentOutputError::StdoutWrite(error) => {
            let _ = emit_diagnostic(
                stderr,
                &format!("{}: {error}", diagnostics::FAILED_TO_WRITE_READABLE_VIEW),
            );
            AdapterExitCode::IoError.code()
        }
        DocumentOutputError::ReadablePayload(error) => {
            let _ = emit_diagnostic(
                stderr,
                &format!("{}: {error}", diagnostics::FAILED_TO_WRITE_JSON),
            );
            AdapterExitCode::IoError.code()
        }
        DocumentOutputError::StdoutJson(error) => {
            let _ = emit_diagnostic(
                stderr,
                &format!("{}: {error}", diagnostics::FAILED_TO_WRITE_JSON),
            );
            AdapterExitCode::IoError.code()
        }
        DocumentOutputError::StderrWarning(error) => {
            let _ = emit_diagnostic(
                stderr,
                &format!("{}: {error}", diagnostics::FAILED_TO_WRITE_CLI_WARNING),
            );
            AdapterExitCode::IoError.code()
        }
    }
}

fn write_cli_warnings<W: Write>(warnings: &[DirectCliWarning], writer: &mut W) -> io::Result<()> {
    write_warning_text_lines(warnings, writer)
}

#[cfg(test)]
mod tests;
