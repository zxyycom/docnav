use std::io::{self, Write};

use docnav_diagnostics::{attach_warnings_to_value, write_warning_text_lines};
use docnav_json_io::write_json_value_pretty;
use docnav_output::{
    write_document_error, write_document_response, DocumentOutputMode, DocumentOutputOptions,
    ProtocolOutputContext,
};
use docnav_protocol::{generate_request_id, Operation, ProtocolResponse, PROTOCOL_VERSION};
use serde_json::Value;

use crate::cli::{CliWarning, OutputMode};
use crate::error::{exit_code_for_error, AppError, AppResult, DocnavExitCode};

pub struct CommandOutcome {
    output: CommandOutput,
    exit_code: DocnavExitCode,
    warnings: Vec<CliWarning>,
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
            warnings: Vec::new(),
        }
    }

    pub fn json(value: Value) -> Self {
        Self {
            output: CommandOutput::Json(value),
            exit_code: DocnavExitCode::Success,
            warnings: Vec::new(),
        }
    }

    pub fn json_with_exit(value: Value, exit_code: DocnavExitCode) -> Self {
        Self {
            output: CommandOutput::Json(value),
            exit_code,
            warnings: Vec::new(),
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
            warnings: Vec::new(),
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<CliWarning>) -> Self {
        self.warnings = warnings;
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
    warnings: &[CliWarning],
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let mut combined_warnings = warnings.to_vec();
    combined_warnings.extend(outcome.warnings);

    let result = match outcome.output {
        CommandOutput::PlainText(text) => write_plain_text(&text, &combined_warnings, stdout),
        CommandOutput::Json(value) => {
            write_json(attach_warnings_to_value(value, &combined_warnings), stdout)
                .map_err(io::Error::other)
        }
        CommandOutput::DocumentResponse { response, mode } => {
            write_document_response(&response, mode, &combined_warnings, stdout, stderr)
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
        warnings,
        stdout,
        stderr,
    } = request;
    let request_id = generate_request_id();
    let protocol = ProtocolOutputContext::new(PROTOCOL_VERSION, &request_id, operation);
    let result = write_document_error(
        error.error(),
        protocol,
        DocumentOutputOptions::new(document_output_mode(output_mode), warnings),
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
    pub warnings: &'a [CliWarning],
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

fn write_io_error<E: Write>(error: io::Error, stderr: &mut E) -> i32 {
    let _ = writeln!(stderr, "failed to write docnav output: {error}");
    DocnavExitCode::InternalError.code()
}

#[cfg(test)]
mod tests;
