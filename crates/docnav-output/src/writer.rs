use std::io::Write;

use docnav_diagnostics::{write_warning_text_lines, DiagnosticRecord};
use docnav_json_io::write_json_value_pretty;
use docnav_protocol::{
    FailureResponse, OperationResult, ProtocolError, ProtocolResponse, SuccessResponse,
    PROTOCOL_VERSION,
};
use docnav_readable::ReadableViewKind;

use crate::readable::{
    add_warnings, protocol_error_readable, readable_payload, view_kind_for_result,
    write_readable_view_value,
};
use crate::{
    DocumentOutputError, DocumentOutputMode, DocumentOutputOptions, DocumentOutputStatus,
    ProtocolOutputContext,
};

pub fn write_document_response<W, E>(
    response: &ProtocolResponse,
    mode: DocumentOutputMode,
    diagnostics: &[DiagnosticRecord],
    stdout: &mut W,
    stderr: &mut E,
) -> Result<DocumentOutputStatus, DocumentOutputError>
where
    W: Write,
    E: Write,
{
    let output = DocumentOutputOptions::new(mode, diagnostics);

    if output.mode == DocumentOutputMode::ProtocolJson {
        return write_protocol_response(response, output, stdout, stderr);
    }

    match response {
        ProtocolResponse::Success(success) => {
            write_success_response(success, output, stdout, stderr)
        }
        ProtocolResponse::Failure(failure) => {
            write_failure_response(failure, output, stdout, stderr)
        }
    }
}

fn write_protocol_response<W, E>(
    response: &ProtocolResponse,
    output: DocumentOutputOptions<'_>,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<DocumentOutputStatus, DocumentOutputError>
where
    W: Write,
    E: Write,
{
    let warnings = output.warning_projections();
    write_json_value_pretty(response, stdout).map_err(DocumentOutputError::StdoutJson)?;
    write_warning_text_lines(&warnings, stderr).map_err(DocumentOutputError::StderrWarning)?;
    Ok(response_status(response))
}

fn response_status(response: &ProtocolResponse) -> DocumentOutputStatus {
    match response {
        ProtocolResponse::Success(_) => DocumentOutputStatus::Success,
        ProtocolResponse::Failure(failure) => DocumentOutputStatus::Failure(failure.error.clone()),
    }
}

fn write_success_response<W, E>(
    success: &SuccessResponse,
    output: DocumentOutputOptions<'_>,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<DocumentOutputStatus, DocumentOutputError>
where
    W: Write,
    E: Write,
{
    write_document_success(
        &success.result,
        success.request_id.as_str(),
        output,
        stdout,
        stderr,
    )?;
    Ok(DocumentOutputStatus::Success)
}

fn write_failure_response<W, E>(
    failure: &FailureResponse,
    output: DocumentOutputOptions<'_>,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<DocumentOutputStatus, DocumentOutputError>
where
    W: Write,
    E: Write,
{
    let context = ProtocolOutputContext::new(
        failure.protocol_version.as_str(),
        failure.request_id.as_str(),
        failure.operation,
    );
    write_document_failure(&failure.error, context, output, stdout, stderr)?;
    Ok(DocumentOutputStatus::Failure(failure.error.clone()))
}

pub fn write_document_result<W, E>(
    result: &OperationResult,
    request_id: &str,
    output: DocumentOutputOptions<'_>,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), DocumentOutputError>
where
    W: Write,
    E: Write,
{
    write_document_success(result, request_id, output, stdout, stderr)
}

fn write_document_success<W, E>(
    result: &OperationResult,
    request_id: &str,
    output: DocumentOutputOptions<'_>,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), DocumentOutputError>
where
    W: Write,
    E: Write,
{
    match output.mode {
        DocumentOutputMode::ReadableView => {
            let value = readable_payload(result)?;
            let warnings = output.warning_projections();
            write_readable_view_value(value, view_kind_for_result(result), &warnings, stdout)
        }
        DocumentOutputMode::ReadableJson => {
            let warnings = output.warning_projections();
            let value = add_warnings(readable_payload(result)?, &warnings);
            write_json_value_pretty(&value, stdout).map_err(DocumentOutputError::StdoutJson)
        }
        DocumentOutputMode::ProtocolJson => {
            let response = ProtocolResponse::success(PROTOCOL_VERSION, request_id, result.clone());
            write_json_value_pretty(&response, stdout).map_err(DocumentOutputError::StdoutJson)?;
            let warnings = output.warning_projections();
            write_warning_text_lines(&warnings, stderr).map_err(DocumentOutputError::StderrWarning)
        }
    }
}

pub fn write_document_error<W, E>(
    error: &ProtocolError,
    protocol: ProtocolOutputContext<'_>,
    output: DocumentOutputOptions<'_>,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), DocumentOutputError>
where
    W: Write,
    E: Write,
{
    write_document_failure(error, protocol, output, stdout, stderr)
}

pub fn write_document_diagnostic_error<W, E>(
    error: &DiagnosticRecord,
    protocol: ProtocolOutputContext<'_>,
    output: DocumentOutputOptions<'_>,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), DocumentOutputError>
where
    W: Write,
    E: Write,
{
    let stable = ProtocolError::from_diagnostic_record(error)
        .ok_or(DocumentOutputError::DiagnosticProjection)?;
    write_document_failure(&stable, protocol, output, stdout, stderr)
}

fn write_document_failure<W, E>(
    error: &ProtocolError,
    protocol: ProtocolOutputContext<'_>,
    output: DocumentOutputOptions<'_>,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), DocumentOutputError>
where
    W: Write,
    E: Write,
{
    match output.mode {
        DocumentOutputMode::ReadableView => {
            let warnings = output.warning_projections();
            write_readable_view_value(
                protocol_error_readable(error),
                ReadableViewKind::Error,
                &warnings,
                stdout,
            )
        }
        DocumentOutputMode::ReadableJson => {
            let warnings = output.warning_projections();
            let value = add_warnings(protocol_error_readable(error), &warnings);
            write_json_value_pretty(&value, stdout).map_err(DocumentOutputError::StdoutJson)
        }
        DocumentOutputMode::ProtocolJson => {
            let response = ProtocolResponse::failure(
                protocol.protocol_version,
                protocol.request_id,
                protocol.operation,
                error.clone(),
            );
            write_json_value_pretty(&response, stdout).map_err(DocumentOutputError::StdoutJson)?;
            let warnings = output.warning_projections();
            write_warning_text_lines(&warnings, stderr).map_err(DocumentOutputError::StderrWarning)
        }
    }
}
