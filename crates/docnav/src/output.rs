use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use docnav_protocol::{
    Operation, OperationResult, ProtocolResponse, StableError, PROTOCOL_VERSION,
};
use docnav_readable::{render_readable_view, to_readable_value, ReadableViewKind, RendererConfig};
use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::cli::{CliWarning, OutputMode};
use crate::error::{exit_code_for_error, AppError, AppResult, DocnavExitCode};

// ── CommandOutcome ───────────────────────────────────────────────────────────

/// The result of executing a command, ready for output writing.
///
/// Holds the output payload, exit code, and outcome-level warnings
/// (e.g. adapter candidate warnings). CLI argv warnings are passed
/// separately to the write functions.
pub struct CommandOutcome {
    output: CommandOutput,
    exit_code: DocnavExitCode,
    warnings: Vec<CliWarning>,
}

/// Internal output representation.
///
/// - `PlainText`: non-document plain text (help, version, diagnostics).
/// - `ReadableView`: readable JSON payload pending readable-view rendering.
///   Rendering happens in `write_outcome` after warnings are merged.
/// - `Json`: structured JSON for readable-json document output and config commands.
/// - `ProtocolJson`: full protocol response envelope.
enum CommandOutput {
    PlainText(String),
    ReadableView {
        value: Value,
        kind: ReadableViewKind,
    },
    Json(Value),
    ProtocolJson(Value),
}

impl CommandOutcome {
    // ── plain text (non-document) ─────────────────────────────────────────

    /// Create a plain-text outcome for non-document commands (help, version).
    pub fn plain_text(text: impl Into<String>) -> Self {
        Self {
            output: CommandOutput::PlainText(text.into()),
            exit_code: DocnavExitCode::Success,
            warnings: Vec::new(),
        }
    }

    // ── readable-view (document) ──────────────────────────────────────────

    /// Create a readable-view outcome from a readable JSON payload.
    /// Rendering is deferred to `write_outcome` so that warnings can be
    /// merged into the JSON header first.
    pub fn readable_view(value: Value, kind: ReadableViewKind) -> Self {
        Self {
            output: CommandOutput::ReadableView { value, kind },
            exit_code: DocnavExitCode::Success,
            warnings: Vec::new(),
        }
    }

    pub fn readable_view_with_exit(
        value: Value,
        kind: ReadableViewKind,
        exit_code: DocnavExitCode,
    ) -> Self {
        Self {
            output: CommandOutput::ReadableView { value, kind },
            exit_code,
            warnings: Vec::new(),
        }
    }

    // ── json (readable-json + config output) ──────────────────────────────

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

    // ── protocol-json ─────────────────────────────────────────────────────

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

    /// Attach outcome-level warnings (e.g. adapter candidate warnings).
    pub fn with_warnings(mut self, warnings: Vec<CliWarning>) -> Self {
        self.warnings = warnings;
        self
    }
}

// ── outcome dispatch ─────────────────────────────────────────────────────────

/// Build a `CommandOutcome` from a protocol response and output mode.
///
/// Success results and stable errors are first mapped to a complete readable
/// JSON payload. For `ReadableView`, the payload is stored pending rendering
/// in `write_outcome`; for `ReadableJson`, it is stored directly; for
/// `ProtocolJson`, the full protocol envelope is used.
pub fn outcome_for_response(
    response: ProtocolResponse,
    output: OutputMode,
) -> AppResult<CommandOutcome> {
    match response {
        ProtocolResponse::Success(success) => match output {
            OutputMode::ProtocolJson => Ok(CommandOutcome::protocol_json(
                serde_json::to_value(success).unwrap_or(Value::Null),
            )),
            OutputMode::ReadableJson => Ok(CommandOutcome::json(readable_payload(&success.result))),
            OutputMode::ReadableView => {
                let value = readable_payload(&success.result);
                let kind = view_kind_for_result(&success.result);
                Ok(CommandOutcome::readable_view(value, kind))
            }
        },
        ProtocolResponse::Failure(failure) => {
            let exit_code = exit_code_for_error(failure.error.code);
            match output {
                OutputMode::ProtocolJson => Ok(CommandOutcome::protocol_json_with_exit(
                    serde_json::to_value(failure).unwrap_or(Value::Null),
                    exit_code,
                )),
                OutputMode::ReadableJson => Ok(CommandOutcome::json_with_exit(
                    stable_error_readable(&failure.error),
                    exit_code,
                )),
                OutputMode::ReadableView => Ok(CommandOutcome::readable_view_with_exit(
                    stable_error_readable(&failure.error),
                    ReadableViewKind::Error,
                    exit_code,
                )),
            }
        }
    }
}

// ── write functions ──────────────────────────────────────────────────────────

/// Write a `CommandOutcome` to stdout/stderr.
///
/// Combines CLI argv warnings with outcome-level warnings before writing.
/// For `ReadableView`, performs readable-view rendering after merging warnings.
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
        CommandOutput::ReadableView { value, kind } => {
            write_readable_view_outcome(value, kind, &combined_warnings, stdout, stderr)
        }
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

/// Write an `AppError` to stdout/stderr.
pub fn write_error<W: Write, E: Write>(
    error: &AppError,
    output_mode: OutputMode,
    operation: Option<Operation>,
    warnings: &[CliWarning],
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let result = match output_mode {
        OutputMode::ReadableView => {
            write_readable_view_error(error.error(), warnings, stdout, stderr)
        }
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

// ── readable payload constructors ────────────────────────────────────────────

/// Convert an `OperationResult` into the complete readable JSON payload.
///
/// Uses `docnav-readable::to_readable_value` as the single entry point
/// from typed payload to JSON value. Both `readable-view` and `readable-json`
/// derive from this same value.
fn readable_payload(result: &OperationResult) -> Value {
    to_readable_value(result).unwrap_or(Value::Null)
}

/// Map an `OperationResult` variant to the corresponding `ReadableViewKind`.
const fn view_kind_for_result(result: &OperationResult) -> ReadableViewKind {
    match result {
        OperationResult::Outline(_) => ReadableViewKind::Outline,
        OperationResult::Read(_) => ReadableViewKind::Read,
        OperationResult::Find(_) => ReadableViewKind::Find,
        OperationResult::Info(_) => ReadableViewKind::Info,
    }
}

/// Return the committed repo-internal readable-view renderer config.
fn readable_view_config() -> RendererConfig {
    RendererConfig::default_config()
}

// ── readable error payload ───────────────────────────────────────────────────

/// Convert a `StableError` into a readable JSON value.
///
/// Shared by readable-view error rendering and readable-json error output.
fn stable_error_readable(error: &StableError) -> Value {
    json!({
        "code": error.code,
        "error": error.message,
        "details": error.details,
        "guidance": error.guidance.clone().unwrap_or_default(),
    })
}

// ── warning injection ────────────────────────────────────────────────────────

/// Inject `warnings` into a JSON payload value.
///
/// If the payload is an object, warnings are added as a `"warnings"` key.
/// Otherwise the payload is wrapped in `{"value": ..., "warnings": ...}`.
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

// ── low-level writers ────────────────────────────────────────────────────────

fn write_plain_text<W: Write>(
    text: &str,
    warnings: &[CliWarning],
    stdout: &mut W,
) -> io::Result<()> {
    writeln!(stdout, "{text}")?;
    write_cli_warnings(warnings, stdout)
}

fn write_readable_view_outcome<W: Write, E: Write>(
    value: Value,
    kind: ReadableViewKind,
    warnings: &[CliWarning],
    stdout: &mut W,
    _stderr: &mut E,
) -> io::Result<()> {
    let value = add_warnings(value, warnings);
    match render_readable_view(&value, kind, &readable_view_config()) {
        Ok(rendered) => {
            // Write the pre-rendered readable-view string to stdout.
            write!(stdout, "{rendered}")?;
            Ok(())
        }
        Err(render_error) => {
            // Renderer failure: stderr diagnostic, empty stdout.
            // This is an internal error (unexpected config or payload shape).
            Err(io::Error::other(render_error))
        }
    }
}

fn write_readable_view_error<W: Write, E: Write>(
    error: &StableError,
    warnings: &[CliWarning],
    stdout: &mut W,
    _stderr: &mut E,
) -> io::Result<()> {
    let value = stable_error_readable(error);
    let value = add_warnings(value, warnings);
    match render_readable_view(&value, ReadableViewKind::Error, &readable_view_config()) {
        Ok(rendered) => {
            write!(stdout, "{rendered}")?;
            Ok(())
        }
        Err(render_error) => {
            // Treat as if the stderr is the right channel, but we can't write
            // to stdout. The caller handles stderr output for IO errors.
            Err(io::Error::other(render_error))
        }
    }
}

fn write_readable_error<W: Write>(
    error: &StableError,
    warnings: &[CliWarning],
    stdout: &mut W,
) -> io::Result<()> {
    let value = add_warnings(stable_error_readable(error), warnings);
    write_json(value, stdout)
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
        let details = serde_json::to_string(&warning.details).map_err(io::Error::other)?;
        write!(
            writer,
            "warning: id={}, effect={}, reason={}, details={}",
            warning.id.as_str(),
            warning.effect.as_str(),
            warning.reason.replace(['\r', '\n'], " "),
            details
        )?;
        writeln!(writer)?;
    }
    Ok(())
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

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::warning::{CliWarningDetails, CliWarningEffect, CliWarningId};
    use docnav_protocol::{
        Entry, FindResult, InfoResult, OutlineResult, ReadResult, StableErrorCode,
    };
    use std::collections::BTreeMap;

    // ── helpers ────────────────────────────────────────────────────────────

    fn cli_argv_warning(tokens: &[&str]) -> CliWarning {
        CliWarning {
            id: CliWarningId::CliArgvIgnored,
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

    // ── 2.8: readable payload construction ─────────────────────────────────

    #[test]
    fn readable_payload_outline_produces_json_value() {
        let result = OperationResult::Outline(OutlineResult {
            entries: vec![Entry {
                ref_id: "L1".into(),
                display: "Intro".into(),
            }],
            page: None,
        });
        let value = readable_payload(&result);
        assert!(value.is_object());
        assert_eq!(value["entries"][0]["ref"], "L1");
        assert_eq!(value["entries"][0]["display"], "Intro");
        // No protocol envelope fields.
        assert!(value.get("protocol_version").is_none());
    }

    #[test]
    fn readable_payload_read_preserves_content_field() {
        let result = OperationResult::Read(ReadResult {
            ref_id: "L5".into(),
            content: "## Section\n\nBody text.\n".into(),
            content_type: "text/markdown".into(),
            cost: "5 lines".into(),
            page: None,
        });
        let value = readable_payload(&result);
        assert_eq!(value["ref"], "L5");
        assert_eq!(value["content"], "## Section\n\nBody text.\n");
        assert_eq!(value["content_type"], "text/markdown");
    }

    // ── 2.8: view kind mapping ─────────────────────────────────────────────

    #[test]
    fn view_kind_outline() {
        let result = OperationResult::Outline(OutlineResult {
            entries: vec![],
            page: None,
        });
        assert_eq!(view_kind_for_result(&result), ReadableViewKind::Outline);
    }

    #[test]
    fn view_kind_read() {
        let result = OperationResult::Read(ReadResult {
            ref_id: "x".into(),
            content: "".into(),
            content_type: "text/plain".into(),
            cost: "".into(),
            page: None,
        });
        assert_eq!(view_kind_for_result(&result), ReadableViewKind::Read);
    }

    #[test]
    fn view_kind_find() {
        let result = OperationResult::Find(FindResult {
            matches: vec![],
            page: None,
        });
        assert_eq!(view_kind_for_result(&result), ReadableViewKind::Find);
    }

    #[test]
    fn view_kind_info() {
        let result = OperationResult::Info(InfoResult {
            display: "adapter v1".into(),
            capabilities: vec![],
        });
        assert_eq!(view_kind_for_result(&result), ReadableViewKind::Info);
    }

    // ── 2.8: PlainText channel ─────────────────────────────────────────────

    #[test]
    fn plain_text_outcome_writes_text_directly() {
        let outcome = CommandOutcome::plain_text("hello world");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = write_outcome(outcome, &[], &mut stdout, &mut stderr);
        assert_eq!(exit, 0);
        let output = String::from_utf8(stdout).unwrap();
        assert!(output.contains("hello world"));
        // PlainText output is NOT JSON.
        assert!(!output.trim().starts_with('{'));
    }

    #[test]
    fn plain_text_outcome_with_warnings() {
        let outcome = CommandOutcome::plain_text("version info");
        let warning = cli_argv_warning(&["--extra"]);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = write_outcome(outcome, &[warning], &mut stdout, &mut stderr);
        assert_eq!(exit, 0);
        let output = String::from_utf8(stdout).unwrap();
        assert!(output.contains("version info"));
        assert!(output.contains("cli_argv_ignored"));
    }

    // ── 2.8: ReadableView outcome rendering ────────────────────────────────

    #[test]
    fn readable_view_outline_renders_json_header() {
        let value = readable_payload(&OperationResult::Outline(OutlineResult {
            entries: vec![Entry {
                ref_id: "R1".into(),
                display: "Test".into(),
            }],
            page: None,
        }));
        let outcome = CommandOutcome::readable_view(value, ReadableViewKind::Outline);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = write_outcome(outcome, &[], &mut stdout, &mut stderr);
        assert_eq!(exit, 0);
        let output = String::from_utf8(stdout).unwrap();
        // Header JSON should contain the entry data.
        assert!(output.contains("\"ref\": \"R1\""));
        assert!(output.contains("\"display\": \"Test\""));
        // Outline has no blocks, so no block markers.
        assert!(!output.contains("[block"));
        assert!(!output.contains("[endblock"));
    }

    #[test]
    fn readable_view_read_has_content_block() {
        let value = readable_payload(&OperationResult::Read(ReadResult {
            ref_id: "R1".into(),
            content: "## Hello\n\nWorld\n".into(),
            content_type: "text/markdown".into(),
            cost: "2 lines".into(),
            page: None,
        }));
        let outcome = CommandOutcome::readable_view(value, ReadableViewKind::Read);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = write_outcome(outcome, &[], &mut stdout, &mut stderr);
        assert_eq!(exit, 0);
        let output = String::from_utf8(stdout).unwrap();
        // Header must have the block reference, not the original content.
        assert!(output.contains("\"$block\": \"/content\""));
        // Block section must be present.
        assert!(output.contains("[block /content bytes="));
        assert!(output.contains("## Hello\n\nWorld"));
        assert!(output.contains("[endblock /content]"));
        // Other fields remain in header.
        assert!(output.contains("\"ref\": \"R1\""));
        assert!(output.contains("\"content_type\": \"text/markdown\""));
    }

    #[test]
    fn readable_view_with_warnings_includes_warnings_in_header() {
        let value = readable_payload(&OperationResult::Outline(OutlineResult {
            entries: vec![],
            page: None,
        }));
        let outcome = CommandOutcome::readable_view(value, ReadableViewKind::Outline);
        let warning = cli_argv_warning(&["--extra"]);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = write_outcome(outcome, &[warning], &mut stdout, &mut stderr);
        assert_eq!(exit, 0);
        let output = String::from_utf8(stdout).unwrap();
        // Warnings should be in the JSON header.
        assert!(output.contains("\"warnings\""));
        assert!(output.contains("cli_argv_ignored"));
    }

    // ── 2.8: ReadableJson outcome ──────────────────────────────────────────

    #[test]
    fn readable_json_outcome_preserves_structure() {
        let value = readable_payload(&OperationResult::Outline(OutlineResult {
            entries: vec![Entry {
                ref_id: "R1".into(),
                display: "Test".into(),
            }],
            page: None,
        }));
        let outcome = CommandOutcome::json(value);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = write_outcome(outcome, &[], &mut stdout, &mut stderr);
        assert_eq!(exit, 0);
        let output = String::from_utf8(stdout).unwrap();
        // Should be valid JSON.
        let parsed: Value = serde_json::from_str(&output).expect("should be valid JSON");
        assert_eq!(parsed["entries"][0]["ref"], "R1");
        // No protocol envelope fields.
        assert!(parsed.get("protocol_version").is_none());
    }

    // ── 2.8: Protocol-json boundary unchanged ──────────────────────────────

    #[test]
    fn protocol_json_warnings_go_to_stderr() {
        let outcome = CommandOutcome::protocol_json(json!({
            "protocol_version": "0.1.0",
            "status": "success",
            "result": {}
        }));
        let warning = cli_argv_warning(&["--extra"]);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = write_outcome(outcome, &[warning], &mut stdout, &mut stderr);
        assert_eq!(exit, 0);
        let stdout_str = String::from_utf8(stdout).unwrap();
        let stderr_str = String::from_utf8(stderr).unwrap();
        // Warnings must be on stderr, not in stdout.
        assert!(!stdout_str.contains("cli_argv_ignored"));
        assert!(stderr_str.contains("cli_argv_ignored"));
        // stdout has protocol envelope.
        assert!(stdout_str.contains("protocol_version"));
    }

    // ── 2.8: Readable error ────────────────────────────────────────────────

    #[test]
    fn readable_error_payload_includes_guidance() {
        let error = StableError {
            code: StableErrorCode::RefNotFound,
            message: "ref not found".into(),
            details: error_details(&[("ref", "L99")]),
            guidance: Some(vec!["Try outline first.".into()]),
        };
        let value = stable_error_readable(&error);
        assert_eq!(value["code"], "REF_NOT_FOUND");
        assert_eq!(value["error"], "ref not found");
        assert!(value["guidance"].is_array());
        assert_eq!(value["guidance"][0], "Try outline first.");
    }

    #[test]
    fn readable_view_error_renders_with_block() {
        let error = StableError {
            code: StableErrorCode::RefNotFound,
            message: "No content found for ref `L99`".into(),
            details: error_details(&[("ref", "L99")]),
            guidance: Some(vec!["Check available entries.".into()]),
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = write_readable_view_error(&error, &[], &mut stdout, &mut stderr);
        assert!(result.is_ok());
        let output = String::from_utf8(stdout).unwrap();
        // Error message is in a block.
        assert!(output.contains("[block /error bytes="));
        assert!(output.contains("No content found for ref `L99`"));
        assert!(output.contains("[endblock /error]"));
        // Code and details remain in header.
        assert!(output.contains("\"code\": \"REF_NOT_FOUND\""));
        // guidance array stays in header JSON.
        assert!(output.contains("\"guidance\": ["));
    }

    // ── 2.8: renderer failure ──────────────────────────────────────────────

    #[test]
    fn readable_view_outcome_render_failure_returns_internal_error() {
        // Create a value that will fail rendering: a read payload without /content.
        let value = json!({"ref": "test", "not_content": 42});
        let outcome = CommandOutcome {
            output: CommandOutput::ReadableView {
                value,
                kind: ReadableViewKind::Read,
            },
            exit_code: DocnavExitCode::Success,
            warnings: vec![],
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = write_outcome(outcome, &[], &mut stdout, &mut stderr);
        assert_eq!(exit, DocnavExitCode::InternalError.code());
        let stderr_str = String::from_utf8(stderr).unwrap();
        assert!(stderr_str.contains("readable_view_render_failed"));
        // stdout should be empty (render failure).
        let stdout_str = String::from_utf8(stdout).unwrap();
        assert!(stdout_str.is_empty());
    }

    // ── 2.8: Output mode values ────────────────────────────────────────────

    #[test]
    fn output_mode_as_str_values() {
        assert_eq!(OutputMode::ReadableView.as_str(), "readable-view");
        assert_eq!(OutputMode::ReadableJson.as_str(), "readable-json");
        assert_eq!(OutputMode::ProtocolJson.as_str(), "protocol-json");
    }

    #[test]
    fn output_mode_from_str_valid_values() {
        assert_eq!(
            "readable-view".parse::<OutputMode>().unwrap(),
            OutputMode::ReadableView
        );
        assert_eq!(
            "readable-json".parse::<OutputMode>().unwrap(),
            OutputMode::ReadableJson
        );
        assert_eq!(
            "protocol-json".parse::<OutputMode>().unwrap(),
            OutputMode::ProtocolJson
        );
    }

    #[test]
    fn output_mode_from_str_invalid_value() {
        let err = "text".parse::<OutputMode>().unwrap_err();
        assert!(err.contains("invalid output value"));
        assert!(err.contains("text"));
        assert!(err.contains("readable-view"));
        assert!(err.contains("readable-json"));
        assert!(err.contains("protocol-json"));
    }
}
