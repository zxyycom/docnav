use std::io::{self, Write};

use docnav_diagnostics::{attach_warnings_to_value, write_warning_text_lines};
use docnav_json_io::write_json_value_pretty;
use docnav_output::{
    write_document_error, write_document_response, DocumentOutputMode, ProtocolOutputContext,
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
        document_output_mode(output_mode),
        protocol,
        warnings,
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
mod tests {
    // @case WB-CORE-OUTPUT-001
    use super::*;
    use crate::cli::warning::{CliWarningDetails, CliWarningEffect, CLI_ARGV_IGNORED};
    use docnav_protocol::{
        Entry, OperationResult, OutlineResult, ProtocolResponse, ReadResult, StableError,
        StableErrorCode,
    };
    use serde_json::json;
    use std::collections::BTreeMap;

    fn cli_argv_warning(tokens: &[&str]) -> CliWarning {
        CliWarning {
            id: CLI_ARGV_IGNORED,
            reason: "test warning".into(),
            effect: CliWarningEffect::OperationContinued,
            details: CliWarningDetails::CliArgv {
                tokens: tokens.iter().map(|s| s.to_string()).collect(),
            },
        }
    }

    fn error_details(map: &[(&str, &str)]) -> BTreeMap<String, Value> {
        map.iter().map(|(k, v)| (k.to_string(), json!(v))).collect()
    }

    fn write_success(outcome: CommandOutcome, warnings: &[CliWarning]) -> (Vec<u8>, Vec<u8>) {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = write_outcome(outcome, warnings, &mut stdout, &mut stderr);
        assert_eq!(exit, 0);
        (stdout, stderr)
    }

    #[test]
    fn plain_text_outcome_writes_text_directly() {
        let outcome = CommandOutcome::plain_text("hello world");
        let (stdout, _) = write_success(outcome, &[]);
        let output = String::from_utf8(stdout).unwrap();
        assert!(output.contains("hello world"));
        assert!(!output.trim().starts_with('{'));
    }

    #[test]
    fn non_document_json_warnings_stay_in_json_payload() {
        let outcome = CommandOutcome::json(json!({"config": "ok"}));
        let warning = cli_argv_warning(&["--extra"]);
        let (stdout, stderr) = write_success(outcome, &[warning]);
        assert!(stderr.is_empty());
        let value: Value = serde_json::from_slice(&stdout).unwrap();
        assert_eq!(value["config"], "ok");
        assert_eq!(value["warnings"][0]["id"], "cli_argv_ignored");
    }

    #[test]
    fn document_readable_view_uses_shared_output_facade() {
        let response = ProtocolResponse::success(
            PROTOCOL_VERSION,
            "request-1",
            OperationResult::Read(ReadResult {
                ref_id: "R1".into(),
                content: "body".into(),
                content_type: "text/plain".into(),
                cost: "1 lines | 4 bytes".into(),
                page: None,
            }),
        );
        let outcome = outcome_for_response(response, OutputMode::ReadableView).unwrap();
        let (stdout, _) = write_success(outcome, &[]);
        let output = String::from_utf8(stdout).unwrap();
        assert!(output.contains("\"$block\": \"/content\""));
        assert!(output.contains("[block /content bytes=4]"));
    }

    #[test]
    fn document_readable_json_embeds_warnings() {
        let response = ProtocolResponse::success(
            PROTOCOL_VERSION,
            "request-1",
            OperationResult::Outline(OutlineResult {
                entries: vec![Entry {
                    ref_id: "R1".into(),
                    display: "Test".into(),
                }],
                page: None,
            }),
        );
        let outcome = outcome_for_response(response, OutputMode::ReadableJson).unwrap();
        let warning = cli_argv_warning(&["--extra"]);
        let (stdout, stderr) = write_success(outcome, &[warning]);
        assert!(stderr.is_empty());
        let value: Value = serde_json::from_slice(&stdout).unwrap();
        assert_eq!(value["entries"][0]["ref"], "R1");
        assert_eq!(value["warnings"][0]["id"], "cli_argv_ignored");
        assert!(value.get("protocol_version").is_none());
    }

    #[test]
    fn document_protocol_json_warnings_go_to_stderr() {
        let response = ProtocolResponse::success(
            PROTOCOL_VERSION,
            "request-1",
            OperationResult::Outline(OutlineResult {
                entries: vec![],
                page: None,
            }),
        );
        let outcome = outcome_for_response(response, OutputMode::ProtocolJson).unwrap();
        let warning = cli_argv_warning(&["--extra"]);
        let (stdout, stderr) = write_success(outcome, &[warning]);
        let stdout: Value = serde_json::from_slice(&stdout).unwrap();
        assert!(stdout.get("warnings").is_none());
        assert_eq!(stdout["protocol_version"], PROTOCOL_VERSION);
        let stderr = String::from_utf8(stderr).unwrap();
        assert!(stderr.contains("cli_argv_ignored"));
    }

    #[test]
    fn readable_error_uses_document_facade_and_exit_policy_stays_local() {
        let error = AppError::new(StableError {
            code: StableErrorCode::RefNotFound,
            message: "No content found for ref `L99`".into(),
            details: error_details(&[("ref", "L99")]),
            guidance: Some(vec!["Check available entries.".into()]),
        });
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = write_error(ErrorOutput {
            error: &error,
            output_mode: OutputMode::ReadableView,
            operation: Some(Operation::Read),
            warnings: &[],
            stdout: &mut stdout,
            stderr: &mut stderr,
        });
        assert_eq!(exit, DocnavExitCode::DocumentError.code());
        let output = String::from_utf8(stdout).unwrap();
        assert!(output.contains("[block /error bytes="));
        assert!(output.contains("\"code\": \"REF_NOT_FOUND\""));
    }

    #[test]
    fn output_mode_values_remain_unchanged() {
        assert_eq!(OutputMode::ReadableView.as_str(), "readable-view");
        assert_eq!(OutputMode::ReadableJson.as_str(), "readable-json");
        assert_eq!(OutputMode::ProtocolJson.as_str(), "protocol-json");
    }
}
