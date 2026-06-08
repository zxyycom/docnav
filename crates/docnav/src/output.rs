use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use docnav_protocol::{
    Operation, OperationResult, ProtocolResponse, StableError, StableErrorCode, PROTOCOL_VERSION,
};
use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::cli::{CliWarning, OutputMode};
use crate::error::{exit_code_for_error, AppError, DocnavExitCode};

pub struct CommandOutcome {
    output: CommandOutput,
    exit_code: DocnavExitCode,
    warnings: Vec<CliWarning>,
}

enum CommandOutput {
    Text(String),
    Json(Value),
    ProtocolJson(Value),
}

impl CommandOutcome {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            output: CommandOutput::Text(text.into()),
            exit_code: DocnavExitCode::Success,
            warnings: Vec::new(),
        }
    }

    pub fn text_with_exit(text: impl Into<String>, exit_code: DocnavExitCode) -> Self {
        Self {
            output: CommandOutput::Text(text.into()),
            exit_code,
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

    pub fn protocol_json(value: Value) -> Self {
        Self {
            output: CommandOutput::ProtocolJson(value),
            exit_code: DocnavExitCode::Success,
            warnings: Vec::new(),
        }
    }

    pub fn protocol_json_with_exit(value: Value, exit_code: DocnavExitCode) -> Self {
        Self {
            output: CommandOutput::ProtocolJson(value),
            exit_code,
            warnings: Vec::new(),
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<CliWarning>) -> Self {
        self.warnings = warnings;
        self
    }
}

pub fn outcome_for_response(response: ProtocolResponse, output: OutputMode) -> CommandOutcome {
    match response {
        ProtocolResponse::Success(success) => match output {
            OutputMode::ProtocolJson => {
                CommandOutcome::protocol_json(serde_json::to_value(success).unwrap_or(Value::Null))
            }
            OutputMode::ReadableJson => CommandOutcome::json(readable_result(&success.result)),
            OutputMode::Text => CommandOutcome::text(text_result(&success.result)),
        },
        ProtocolResponse::Failure(failure) => {
            let exit_code = exit_code_for_error(failure.error.code);
            match output {
                OutputMode::ProtocolJson => CommandOutcome::protocol_json_with_exit(
                    serde_json::to_value(failure).unwrap_or(Value::Null),
                    exit_code,
                ),
                OutputMode::ReadableJson => {
                    CommandOutcome::json_with_exit(stable_error_readable(&failure.error), exit_code)
                }
                OutputMode::Text => {
                    CommandOutcome::text_with_exit(stable_error_text(&failure.error), exit_code)
                }
            }
        }
    }
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
        CommandOutput::Text(text) => write_text(&text, &combined_warnings, stdout),
        CommandOutput::Json(value) => write_json(add_warnings(value, &combined_warnings), stdout),
        CommandOutput::ProtocolJson(value) => {
            write_json(value, stdout).and_then(|_| write_cli_warnings(&combined_warnings, stderr))
        }
    };

    match result {
        Ok(()) => outcome.exit_code.code(),
        Err(error) => write_io_error(error, stderr),
    }
}

pub fn write_error<W: Write, E: Write>(
    error: &AppError,
    output_mode: OutputMode,
    operation: Option<Operation>,
    warnings: &[CliWarning],
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let result = match output_mode {
        OutputMode::Text => write_text_error(error.error(), warnings, stdout),
        OutputMode::ReadableJson => write_readable_error(error.error(), warnings, stdout),
        OutputMode::ProtocolJson => {
            let response = ProtocolResponse::failure(
                PROTOCOL_VERSION,
                request_id(),
                operation,
                error.error().clone(),
            );
            write_json_value(&response, stdout).and_then(|_| write_cli_warnings(warnings, stderr))
        }
    };

    match result {
        Ok(()) => error.exit_code().code(),
        Err(io_error) => write_io_error(io_error, stderr),
    }
}

fn write_text<W: Write>(text: &str, warnings: &[CliWarning], stdout: &mut W) -> io::Result<()> {
    writeln!(stdout, "{text}")?;
    write_cli_warnings(warnings, stdout)
}

fn write_text_error<W: Write>(
    error: &StableError,
    warnings: &[CliWarning],
    stdout: &mut W,
) -> io::Result<()> {
    writeln!(stdout, "{}", stable_error_text(error))?;
    write_cli_warnings(warnings, stdout)
}

fn write_readable_error<W: Write>(
    error: &StableError,
    warnings: &[CliWarning],
    stdout: &mut W,
) -> io::Result<()> {
    let value = add_warnings(stable_error_readable(error), warnings);
    write_json(value, stdout)
}

fn readable_result(result: &OperationResult) -> Value {
    serde_json::to_value(result).unwrap_or(Value::Null)
}

fn text_result(result: &OperationResult) -> String {
    match result {
        OperationResult::Outline(result) => {
            let mut lines = result
                .entries
                .iter()
                .map(|entry| format!("{} | {}", entry.ref_id, entry.display))
                .collect::<Vec<_>>();
            lines.push(format!("page: {}", page_label(result.page)));
            lines.join("\n")
        }
        OperationResult::Read(result) => {
            let mut text = format!("ref: {}\n{}", result.ref_id, result.content);
            if !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(&format!(
                "content_type: {}\ncost: {}\npage: {}",
                result.content_type,
                result.cost,
                page_label(result.page)
            ));
            text
        }
        OperationResult::Find(result) => {
            let mut lines = result
                .matches
                .iter()
                .map(|entry| format!("{} | {}", entry.ref_id, entry.display))
                .collect::<Vec<_>>();
            lines.push(format!("page: {}", page_label(result.page)));
            lines.join("\n")
        }
        OperationResult::Info(result) => format!(
            "{}\ncapabilities: {}",
            result.display,
            result
                .capabilities
                .iter()
                .map(Operation::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn stable_error_readable(error: &StableError) -> Value {
    json!({
        "code": error.code,
        "error": error.message,
        "details": error.details,
        "guidance": error.guidance.clone().unwrap_or_default(),
    })
}

fn stable_error_text(error: &StableError) -> String {
    let mut lines = vec![
        format!("error: {}", error_code_label(error.code)),
        format!("message: {}", error.message),
    ];
    if !error.details.is_empty() {
        lines.push(format!(
            "details: {}",
            serde_json::to_string(&error.details)
                .unwrap_or_else(|_| "<unserializable details>".to_owned())
        ));
    }
    if let Some(guidance) = &error.guidance {
        for item in guidance {
            lines.push(format!("guidance: {item}"));
        }
    }
    lines.join("\n")
}

fn add_warnings(mut value: Value, warnings: &[CliWarning]) -> Value {
    if warnings.is_empty() {
        return value;
    }
    let warnings = serde_json::to_value(warnings).unwrap_or_else(|_| Value::Array(Vec::new()));
    match &mut value {
        Value::Object(object) => {
            object.insert("warnings".to_owned(), warnings);
            value
        }
        _ => {
            let mut object = Map::new();
            object.insert("value".to_owned(), value);
            object.insert("warnings".to_owned(), warnings);
            Value::Object(object)
        }
    }
}

fn write_json<W: Write>(value: Value, writer: &mut W) -> io::Result<()> {
    write_json_value(&value, writer)
}

fn write_json_value<W: Write, T: Serialize>(value: &T, writer: &mut W) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *writer, value).map_err(io::Error::other)?;
    writeln!(writer)
}

fn write_cli_warnings<W: Write>(warnings: &[CliWarning], writer: &mut W) -> io::Result<()> {
    for warning in warnings {
        let ignored_tokens =
            serde_json::to_string(&warning.ignored_tokens).map_err(io::Error::other)?;
        write!(
            writer,
            "warning: ignored_tokens={}, kind={}, reason={}",
            ignored_tokens,
            warning.kind.as_str(),
            warning.reason
        )?;
        if let Some(adapter_id) = &warning.adapter_id {
            write!(writer, ", adapter_id={adapter_id}")?;
        }
        if let Some(stage) = &warning.stage {
            write!(writer, ", stage={stage}")?;
        }
        if let Some(code) = &warning.code {
            write!(writer, ", code={code}")?;
        }
        writeln!(writer)?;
    }
    Ok(())
}

fn error_code_label(code: StableErrorCode) -> String {
    serde_json::to_value(code)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("{code:?}"))
}

fn page_label(page: Option<docnav_protocol::PositiveInteger>) -> String {
    page.map(|page| page.get().to_string())
        .unwrap_or_else(|| "null".to_owned())
}

fn write_io_error<E: Write>(error: io::Error, stderr: &mut E) -> i32 {
    let _ = writeln!(stderr, "failed to write docnav output: {error}");
    DocnavExitCode::InternalError.code()
}

fn request_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("docnav-{nanos}")
}
