use std::io::{self, Write};

use docnav_json_io::write_json_value_pretty;
use docnav_navigation::NavigationCommandOutcome;
use docnav_output::{
    render_readable_response, write_document_response, DocumentOutputError, OutputPlan,
};
use docnav_protocol::{
    generate_request_id, Operation, ProtocolError, ProtocolResponse, PROTOCOL_VERSION,
};
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
        plan: OutputPlan,
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
                plan: document_output_plan(mode),
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
            plan,
            invocation_log,
        } => match write_document_response(&navigation_outcome.response, plan, stdout) {
            Ok(()) => {
                if let Some(invocation_log) = invocation_log {
                    invocation_log.record_outcome(&navigation_outcome);
                }
                outcome.exit_code.code()
            }
            Err(error) => {
                let error_code = error.primary_error_id().to_owned();
                let error_summary = error.to_string();
                let exit_code = write_document_output_error(error, stderr);
                if let Some(invocation_log) = invocation_log.as_ref() {
                    invocation_log.record_output_projection_error(
                        &navigation_outcome,
                        &error_code,
                        error_summary,
                    );
                }
                exit_code
            }
        },
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
    let protocol_error = match ProtocolError::from_diagnostic_record(&error_record) {
        Some(error) => error,
        None => {
            return write_io_error(
                io::Error::other("failed to project docnav diagnostic into protocol error"),
                stderr,
            )
        }
    };
    let request_id = generate_request_id();
    let response =
        ProtocolResponse::failure(PROTOCOL_VERSION, request_id, operation, protocol_error);

    match write_document_response(&response, document_output_plan(output_mode), stdout) {
        Ok(()) => error.exit_code().code(),
        Err(output_error) => write_document_output_error(output_error, stderr),
    }
}

pub struct ErrorOutput<'a, W: Write, E: Write> {
    pub error: &'a AppError,
    pub output_mode: OutputMode,
    pub operation: Option<Operation>,
    pub stdout: &'a mut W,
    pub stderr: &'a mut E,
}

fn document_output_plan(mode: OutputMode) -> OutputPlan {
    match mode {
        OutputMode::ReadableView => OutputPlan::Rendered(render_readable_response),
        OutputMode::ProtocolJson => OutputPlan::ProtocolJson,
    }
}

fn write_document_output_error<E: Write>(error: DocumentOutputError, stderr: &mut E) -> i32 {
    if matches!(&error, DocumentOutputError::Render(_)) {
        return write_bounded_fatal_diagnostic(&error, stderr);
    }

    write_io_error(io::Error::other(error), stderr)
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
