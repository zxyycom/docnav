use std::io::{self, Write};

use docnav_diagnostics::{
    write_warning_text_lines, BoundaryDiagnosticCode, DiagnosticRecord, DiagnosticSource,
    DiagnosticStack, Warning,
};
use docnav_output::{
    write_document_diagnostic_error, write_document_result, DocumentOutputError,
    DocumentOutputMode, DocumentOutputOptions, ProtocolOutputContext,
};
use docnav_protocol::{OperationResult, PROTOCOL_VERSION};

use crate::constants::diagnostics;
use crate::{output::emit_boundary_diagnostic, AdapterError, AdapterExitCode};

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
    let diagnostics = diagnostic_records_for_warnings(warnings);
    match output {
        DirectOutputMode::ReadableView | DirectOutputMode::ReadableJson => {
            match write_document_result(
                &result,
                "adapter-direct",
                DocumentOutputOptions::new(document_output_mode(output), &diagnostics),
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
    let (error_record, diagnostics) = diagnostic_error_records(stable, warnings);
    let write_exit = match output {
        DirectOutputMode::ReadableView | DirectOutputMode::ReadableJson => {
            let protocol = ProtocolOutputContext::new(PROTOCOL_VERSION, "adapter-direct", None);
            match write_document_diagnostic_error(
                &error_record,
                protocol,
                DocumentOutputOptions::new(document_output_mode(output), &diagnostics),
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
    let diagnostics = diagnostic_records_for_warnings(warnings);
    match write_diagnostic_warnings(&diagnostics, stderr) {
        Ok(()) => exit_code,
        Err(error) => {
            let _ = emit_boundary_diagnostic(
                stderr,
                BoundaryDiagnosticCode::FailedToWriteCliWarning,
                format!("{}: {error}", diagnostics::FAILED_TO_WRITE_CLI_WARNING),
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
        DocumentOutputError::StderrWarning(error) => {
            let _ = emit_boundary_diagnostic(
                stderr,
                BoundaryDiagnosticCode::FailedToWriteCliWarning,
                format!("{}: {error}", diagnostics::FAILED_TO_WRITE_CLI_WARNING),
            );
            AdapterExitCode::IoError.code()
        }
    }
}

fn write_diagnostic_warnings<W: Write>(
    diagnostics: &[DiagnosticRecord],
    writer: &mut W,
) -> io::Result<()> {
    let warnings = diagnostics
        .iter()
        .filter_map(Warning::from_record)
        .collect::<Vec<_>>();
    write_warning_text_lines(&warnings, writer)
}

fn diagnostic_records_for_warnings(warnings: &[DirectCliWarning]) -> Vec<DiagnosticRecord> {
    let mut diagnostics = DiagnosticStack::new();
    push_warning_diagnostics(&mut diagnostics, warnings);
    let mut records = diagnostics.snapshot();
    records.reverse();
    records
}

fn diagnostic_error_records(
    error: &docnav_protocol::StableError,
    warnings: &[DirectCliWarning],
) -> (DiagnosticRecord, Vec<DiagnosticRecord>) {
    let mut diagnostics = DiagnosticStack::new();
    push_warning_diagnostics(&mut diagnostics, warnings);
    let error_id = diagnostics
        .push(error.to_record_draft(DiagnosticSource::with_stage("adapter-direct", "error")))
        .expect("stable errors must satisfy diagnostic details rules");
    let error_record = diagnostics
        .get(error_id)
        .expect("pushed diagnostic record exists")
        .clone();
    let mut records = diagnostics.snapshot();
    records.reverse();
    (error_record, records)
}

fn push_warning_diagnostics(diagnostics: &mut DiagnosticStack, warnings: &[DirectCliWarning]) {
    for warning in warnings {
        if let Some(draft) =
            warning.to_record_draft(DiagnosticSource::with_stage("adapter-direct", "cli"))
        {
            let _ = diagnostics.push(draft);
        }
    }
}

#[cfg(test)]
mod tests;
