use std::io::Write;

use docnav_diagnostics::{BoundaryDiagnosticCode, DiagnosticRecord, DiagnosticStack};
use docnav_output::{
    write_document_diagnostic_error, write_document_result, DocumentOutputError,
    DocumentOutputMode, ProtocolOutputContext,
};
use docnav_protocol::{Operation, OperationResult, PROTOCOL_VERSION};

use crate::constants::diagnostics;
use crate::{output::emit_boundary_diagnostic, AdapterError, AdapterExitCode};

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
                document_output_mode(output),
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
    operation: Option<Operation>,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let exit_code = error.exit_code();
    let error_record = match diagnostic_error_record(error.diagnostic()) {
        Ok(records) => records,
        Err(error) => return document_output_error(error, stderr),
    };
    let write_exit = match output {
        DirectOutputMode::ReadableView
        | DirectOutputMode::ReadableJson
        | DirectOutputMode::ProtocolJson => {
            let protocol =
                ProtocolOutputContext::new(PROTOCOL_VERSION, "adapter-direct", operation);
            match write_document_diagnostic_error(
                &error_record,
                protocol,
                document_output_mode(output),
                stdout,
                stderr,
            ) {
                Ok(()) => AdapterExitCode::Success.code(),
                Err(error) => document_output_error(error, stderr),
            }
        }
    };
    if write_exit == AdapterExitCode::Success.code() {
        exit_code.code()
    } else {
        write_exit
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
        DocumentOutputError::DiagnosticProjection => {
            let _ = emit_boundary_diagnostic(
                stderr,
                BoundaryDiagnosticCode::FailedToWriteReadableView,
                "failed to project diagnostic output",
            );
            AdapterExitCode::InternalError.code()
        }
        DocumentOutputError::ReadableViewRender(error) => {
            let _ = emit_boundary_diagnostic(
                stderr,
                BoundaryDiagnosticCode::ReadableViewRenderFailed,
                format!("readable_view_render_failed: {error}"),
            );
            AdapterExitCode::InternalError.code()
        }
        DocumentOutputError::StdoutWrite(error) => {
            let _ = emit_boundary_diagnostic(
                stderr,
                BoundaryDiagnosticCode::FailedToWriteReadableView,
                format!("{}: {error}", diagnostics::FAILED_TO_WRITE_READABLE_VIEW),
            );
            AdapterExitCode::IoError.code()
        }
        DocumentOutputError::ReadablePayload(error) => {
            let _ = emit_boundary_diagnostic(
                stderr,
                BoundaryDiagnosticCode::FailedToWriteJson,
                format!("{}: {error}", diagnostics::FAILED_TO_WRITE_JSON),
            );
            AdapterExitCode::IoError.code()
        }
        DocumentOutputError::StdoutJson(error) => {
            let _ = emit_boundary_diagnostic(
                stderr,
                BoundaryDiagnosticCode::FailedToWriteJson,
                format!("{}: {error}", diagnostics::FAILED_TO_WRITE_JSON),
            );
            AdapterExitCode::IoError.code()
        }
    }
}

fn diagnostic_error_record(
    error: &docnav_diagnostics::DiagnosticRecordDraft,
) -> Result<DiagnosticRecord, DocumentOutputError> {
    let mut diagnostics = DiagnosticStack::new();
    let error_id = diagnostics
        .push(error.clone())
        .map_err(|_| DocumentOutputError::DiagnosticProjection)?;
    diagnostics
        .get(error_id)
        .ok_or(DocumentOutputError::DiagnosticProjection)
        .cloned()
}

#[cfg(test)]
mod tests;
