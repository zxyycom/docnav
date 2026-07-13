use std::io::{self, Write};

use docnav_json_io::write_json_value_pretty;
use docnav_navigation::NavigationCommandOutcome;
use docnav_output::{
    write_document_diagnostic_error, write_document_response, DocumentOutputError,
    DocumentOutputMode, ProtocolOutputContext,
};
use docnav_protocol::{generate_request_id, Operation, ProtocolResponse, PROTOCOL_VERSION};
use serde_json::Value;

use crate::cli::OutputMode;
use crate::error::{exit_code_for_diagnostic, AppError, AppResult, DocnavExitCode};
use crate::invocation_log::DocumentInvocationLog;

const MAX_FATAL_DIAGNOSTIC_CHARS: usize = 512;

pub struct CommandOutcome {
    output: CommandOutput,
    exit_code: DocnavExitCode,
}

enum CommandOutput {
    PlainText(String),
    Json(Value),
    DocumentResponse {
        outcome: Box<NavigationCommandOutcome>,
        mode: DocumentOutputMode,
        invocation_log: Option<DocumentInvocationLog>,
    },
}

impl CommandOutcome {
    pub fn plain_text(text: impl Into<String>) -> Self {
        Self {
            output: CommandOutput::PlainText(text.into()),
            exit_code: DocnavExitCode::Success,
        }
    }

    pub fn json(value: Value) -> Self {
        Self {
            output: CommandOutput::Json(value),
            exit_code: DocnavExitCode::Success,
        }
    }

    pub fn json_with_exit(value: Value, exit_code: DocnavExitCode) -> Self {
        Self {
            output: CommandOutput::Json(value),
            exit_code,
        }
    }

    fn document_response(
        outcome: NavigationCommandOutcome,
        mode: OutputMode,
        invocation_log: Option<DocumentInvocationLog>,
    ) -> Self {
        let exit_code = match &outcome.response {
            ProtocolResponse::Success(_) => DocnavExitCode::Success,
            ProtocolResponse::Failure(failure) => exit_code_for_diagnostic(failure.error.code()),
        };
        Self {
            output: CommandOutput::DocumentResponse {
                outcome: Box::new(outcome),
                mode: document_output_mode(mode),
                invocation_log,
            },
            exit_code,
        }
    }
}

pub fn outcome_for_response(
    outcome: NavigationCommandOutcome,
    output: OutputMode,
    invocation_log: Option<DocumentInvocationLog>,
) -> AppResult<CommandOutcome> {
    Ok(CommandOutcome::document_response(
        outcome,
        output,
        invocation_log,
    ))
}

pub fn write_outcome<W: Write, E: Write>(
    outcome: CommandOutcome,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    match outcome.output {
        CommandOutput::PlainText(text) => match write_plain_text(&text, stdout) {
            Ok(()) => outcome.exit_code.code(),
            Err(error) => write_io_error(error, stderr),
        },
        CommandOutput::Json(value) => match write_json(value, stdout) {
            Ok(()) => outcome.exit_code.code(),
            Err(error) => write_io_error(io::Error::other(error), stderr),
        },
        CommandOutput::DocumentResponse {
            outcome: navigation_outcome,
            mode,
            invocation_log,
        } => {
            let operation = response_operation(&navigation_outcome.response);
            match write_document_response(&navigation_outcome.response, mode, stdout, stderr) {
                Ok(_) => {
                    if let Some(invocation_log) = invocation_log {
                        invocation_log.record_outcome(&navigation_outcome);
                    }
                    outcome.exit_code.code()
                }
                Err(error) => {
                    let error_code = error.primary_error_id().to_owned();
                    let error_summary = error.to_string();
                    let exit_code =
                        write_document_output_error(error, mode, operation, stdout, stderr);
                    if let Some(invocation_log) = invocation_log.as_ref() {
                        invocation_log.record_output_projection_error(
                            &navigation_outcome,
                            &error_code,
                            error_summary,
                        );
                    }
                    exit_code
                }
            }
        }
    }
}

pub fn write_error<W: Write, E: Write>(request: ErrorOutput<'_, W, E>) -> i32 {
    let ErrorOutput {
        error,
        output_mode,
        operation,
        stdout,
        stderr,
    } = request;
    let error_record = match error.diagnostic().clone().into_record() {
        Ok(record) => record,
        Err(error) => return write_io_error(io::Error::other(error), stderr),
    };
    let request_id = generate_request_id();
    let protocol = ProtocolOutputContext::new(PROTOCOL_VERSION, &request_id, operation);
    let result = write_document_diagnostic_error(
        &error_record,
        protocol,
        document_output_mode(output_mode),
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

fn cli_output_mode(mode: DocumentOutputMode) -> OutputMode {
    match mode {
        DocumentOutputMode::ReadableView => OutputMode::ReadableView,
        DocumentOutputMode::ReadableJson => OutputMode::ReadableJson,
        DocumentOutputMode::ProtocolJson => OutputMode::ProtocolJson,
    }
}

fn response_operation(response: &ProtocolResponse) -> Option<Operation> {
    match response {
        ProtocolResponse::Success(success) => Some(success.operation),
        ProtocolResponse::Failure(failure) => failure.operation,
    }
}

fn write_document_output_error<W: Write, E: Write>(
    error: DocumentOutputError,
    mode: DocumentOutputMode,
    operation: Option<Operation>,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    if matches!(&error, DocumentOutputError::ReadableViewRender(_)) {
        return write_bounded_fatal_diagnostic(&error, stderr);
    }

    if !error.can_project_as_primary_diagnostic() {
        return write_io_error(io::Error::other(error), stderr);
    }

    let app_error = AppError::internal(error.primary_error_id());
    write_error(ErrorOutput {
        error: &app_error,
        output_mode: cli_output_mode(mode),
        operation,
        stdout,
        stderr,
    })
}

fn write_bounded_fatal_diagnostic<E: Write>(error: &DocumentOutputError, stderr: &mut E) -> i32 {
    let message = format!("failed to write docnav output: {error}");
    let bounded = message
        .chars()
        .take(MAX_FATAL_DIAGNOSTIC_CHARS)
        .collect::<String>();
    let _ = writeln!(stderr, "{bounded}");
    DocnavExitCode::InternalError.code()
}

fn write_plain_text<W: Write>(text: &str, stdout: &mut W) -> io::Result<()> {
    writeln!(stdout, "{text}")
}

fn write_json<W: Write>(value: Value, writer: &mut W) -> Result<(), docnav_json_io::JsonIoError> {
    write_json_value_pretty(&value, writer)
}

fn write_io_error<E: Write>(error: io::Error, stderr: &mut E) -> i32 {
    let _ = writeln!(stderr, "failed to write docnav output: {error}");
    DocnavExitCode::InternalError.code()
}

#[cfg(test)]
mod tests;
