use std::io::{self, Write};

use docnav_diagnostics::{
    attach_warnings_to_value, write_warning_text_lines, DiagnosticRecord, DiagnosticSource,
    DiagnosticStack, Warning,
};
use docnav_json_io::write_json_value_pretty;
use docnav_output::{
    write_document_diagnostic_error, write_document_response, DocumentOutputMode,
    DocumentOutputOptions, ProtocolOutputContext,
};
use docnav_protocol::{generate_request_id, Operation, ProtocolResponse, PROTOCOL_VERSION};
use serde_json::Value;

use crate::cli::{CliWarning, OutputMode};
use crate::error::{exit_code_for_error, AppError, AppResult, DocnavExitCode};

pub struct CommandOutcome {
    output: CommandOutput,
    exit_code: DocnavExitCode,
    diagnostics: DiagnosticStack,
}

enum CommandOutput {
    PlainText(String),
    Json(Value),
    DocumentResponse {
        response: ProtocolResponse,
        mode: DocumentOutputMode,
    },
}

impl CommandOutcome {
    pub fn plain_text(text: impl Into<String>) -> Self {
        Self {
            output: CommandOutput::PlainText(text.into()),
            exit_code: DocnavExitCode::Success,
            diagnostics: DiagnosticStack::new(),
        }
    }

    pub fn json(value: Value) -> Self {
        Self {
            output: CommandOutput::Json(value),
            exit_code: DocnavExitCode::Success,
            diagnostics: DiagnosticStack::new(),
        }
    }

    pub fn json_with_exit(value: Value, exit_code: DocnavExitCode) -> Self {
        Self {
            output: CommandOutput::Json(value),
            exit_code,
            diagnostics: DiagnosticStack::new(),
        }
    }

    fn document_response(response: ProtocolResponse, mode: OutputMode) -> Self {
        let exit_code = match &response {
            ProtocolResponse::Success(_) => DocnavExitCode::Success,
            ProtocolResponse::Failure(failure) => exit_code_for_error(failure.error.code),
        };
        Self {
            output: CommandOutput::DocumentResponse {
                response,
                mode: document_output_mode(mode),
            },
            exit_code,
            diagnostics: DiagnosticStack::new(),
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<CliWarning>) -> Self {
        push_warning_diagnostics(
            &mut self.diagnostics,
            &warnings,
            DiagnosticSource::with_stage("docnav", "runtime"),
        );
        self
    }
}

pub fn outcome_for_response(
    response: ProtocolResponse,
    output: OutputMode,
) -> AppResult<CommandOutcome> {
    Ok(CommandOutcome::document_response(response, output))
}

pub fn write_outcome<W: Write, E: Write>(
    outcome: CommandOutcome,
    diagnostics: DiagnosticStack,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let mut diagnostic_records = diagnostic_records_for_projection(&diagnostics);
    diagnostic_records.extend(diagnostic_records_for_projection(&outcome.diagnostics));
    let combined_warnings = warning_projections(&diagnostic_records);

    let result = match outcome.output {
        CommandOutput::PlainText(text) => write_plain_text(&text, &combined_warnings, stdout),
        CommandOutput::Json(value) => {
            write_json(attach_warnings_to_value(value, &combined_warnings), stdout)
                .map_err(io::Error::other)
        }
        CommandOutput::DocumentResponse { response, mode } => {
            write_document_response(&response, mode, &diagnostic_records, stdout, stderr)
                .map(|_| ())
                .map_err(io::Error::other)
        }
    };

    match result {
        Ok(()) => outcome.exit_code.code(),
        Err(error) => write_io_error(error, stderr),
    }
}

pub fn write_error<W: Write, E: Write>(request: ErrorOutput<'_, W, E>) -> i32 {
    let ErrorOutput {
        error,
        output_mode,
        operation,
        mut diagnostics,
        stdout,
        stderr,
    } = request;
    let error_id = diagnostics
        .push(
            error
                .error()
                .to_record_draft(DiagnosticSource::with_stage("docnav", "error")),
        )
        .expect("stable errors must satisfy diagnostic details rules");
    let error_record = diagnostics
        .get(error_id)
        .expect("pushed diagnostic record exists")
        .clone();
    let diagnostic_records = diagnostic_records_for_projection(&diagnostics);
    let request_id = generate_request_id();
    let protocol = ProtocolOutputContext::new(PROTOCOL_VERSION, &request_id, operation);
    let result = write_document_diagnostic_error(
        &error_record,
        protocol,
        DocumentOutputOptions::new(document_output_mode(output_mode), &diagnostic_records),
        stdout,
        stderr,
    )
    .map_err(io::Error::other);

    match result {
        Ok(()) => error.exit_code().code(),
        Err(io_error) => write_io_error(io_error, stderr),
    }
}

pub struct ErrorOutput<'a, W: Write, E: Write> {
    pub error: &'a AppError,
    pub output_mode: OutputMode,
    pub operation: Option<Operation>,
    pub diagnostics: DiagnosticStack,
    pub stdout: &'a mut W,
    pub stderr: &'a mut E,
}

fn document_output_mode(mode: OutputMode) -> DocumentOutputMode {
    match mode {
        OutputMode::ReadableView => DocumentOutputMode::ReadableView,
        OutputMode::ReadableJson => DocumentOutputMode::ReadableJson,
        OutputMode::ProtocolJson => DocumentOutputMode::ProtocolJson,
    }
}

fn write_plain_text<W: Write>(
    text: &str,
    warnings: &[CliWarning],
    stdout: &mut W,
) -> io::Result<()> {
    writeln!(stdout, "{text}")?;
    write_cli_warnings(warnings, stdout)
}

fn write_json<W: Write>(value: Value, writer: &mut W) -> Result<(), docnav_json_io::JsonIoError> {
    write_json_value_pretty(&value, writer)
}

fn write_cli_warnings<W: Write>(warnings: &[CliWarning], writer: &mut W) -> io::Result<()> {
    write_warning_text_lines(warnings, writer)
}

fn push_warning_diagnostics(
    stack: &mut DiagnosticStack,
    warnings: &[CliWarning],
    source: DiagnosticSource,
) {
    for warning in warnings {
        if let Some(draft) = warning.to_record_draft(source.clone()) {
            let _ = stack.push(draft);
        }
    }
}

fn diagnostic_records_for_projection(stack: &DiagnosticStack) -> Vec<DiagnosticRecord> {
    let mut records = stack.snapshot();
    records.reverse();
    records
}

fn warning_projections(records: &[DiagnosticRecord]) -> Vec<CliWarning> {
    records.iter().filter_map(Warning::from_record).collect()
}

fn write_io_error<E: Write>(error: io::Error, stderr: &mut E) -> i32 {
    let _ = writeln!(stderr, "failed to write docnav output: {error}");
    DocnavExitCode::InternalError.code()
}

#[cfg(test)]
mod tests;
