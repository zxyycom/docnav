use std::io::{self, Write};

use docnav_protocol::{OperationResult, StableError};
use docnav_readable::{render_readable_view, to_readable_value, ReadableViewKind, RendererConfig};
use serde_json::{json, Map, Value};

use crate::constants::diagnostics;
use crate::{emit_diagnostic, AdapterError, AdapterExitCode};

use super::warnings::DirectCliWarning;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectOutputMode {
    /// 默认文档输出：readable-view（pretty JSON header + block sections）。
    ReadableView,
    ReadableJson,
    ProtocolJson,
}

// ── operation output dispatch ──────────────────────────────────────────────

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
    match output {
        DirectOutputMode::ReadableView => {
            write_readable_view_result(&result, warnings, stdout, stderr)
        }
        DirectOutputMode::ReadableJson => write_readable_result(&result, warnings, stdout, stderr),
        DirectOutputMode::ProtocolJson => unreachable!("protocol-json is handled before dispatch"),
    }
}

// ── error handler dispatch ─────────────────────────────────────────────────

pub(super) fn handler_error<W: Write, E: Write>(
    error: AdapterError,
    output: DirectOutputMode,
    warnings: &[DirectCliWarning],
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let exit_code = error.exit_code();
    let stable = error.error();
    let write_exit = match output {
        DirectOutputMode::ReadableView => {
            write_readable_view_error(stable, warnings, stdout, stderr)
        }
        DirectOutputMode::ReadableJson => write_readable_error(stable, warnings, stdout, stderr),
        DirectOutputMode::ProtocolJson => unreachable!("protocol-json is handled before dispatch"),
    };
    if write_exit == AdapterExitCode::Success.code() {
        exit_code.code()
    } else {
        write_exit
    }
}

// ── readable-view result / error ───────────────────────────────────────────

fn write_readable_view_result<W: Write, E: Write>(
    result: &OperationResult,
    warnings: &[DirectCliWarning],
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let value = match to_readable_value(result) {
        Ok(value) => value,
        Err(_) => {
            let _ = emit_diagnostic(
                stderr,
                &format!(
                    "{}: serialization failed",
                    diagnostics::FAILED_TO_WRITE_JSON
                ),
            );
            return AdapterExitCode::IoError.code();
        }
    };
    let kind = view_kind_for_result(result);
    write_readable_view_value(value, kind, warnings, stdout, stderr)
}

fn write_readable_view_error<W: Write, E: Write>(
    error: &StableError,
    warnings: &[DirectCliWarning],
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let value = stable_error_readable(error);
    write_readable_view_value(value, ReadableViewKind::Error, warnings, stdout, stderr)
}

fn write_readable_view_value<W: Write, E: Write>(
    value: Value,
    kind: ReadableViewKind,
    warnings: &[DirectCliWarning],
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let value = add_warnings(value, warnings);
    match render_readable_view(&value, kind, &readable_view_config()) {
        Ok(rendered) => match write!(stdout, "{rendered}") {
            Ok(()) => AdapterExitCode::Success.code(),
            Err(error) => {
                let _ = emit_diagnostic(
                    stderr,
                    &format!("{}: {error}", diagnostics::FAILED_TO_WRITE_READABLE_VIEW),
                );
                AdapterExitCode::IoError.code()
            }
        },
        Err(render_error) => {
            let _ = emit_diagnostic(
                stderr,
                &format!("readable_view_render_failed: {render_error}"),
            );
            AdapterExitCode::InternalError.code()
        }
    }
}

// ── readable-json result / error ───────────────────────────────────────────

fn write_readable_result<W: Write, E: Write>(
    result: &OperationResult,
    warnings: &[DirectCliWarning],
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let value = match to_readable_value(result) {
        Ok(value) => value,
        Err(_) => {
            let _ = emit_diagnostic(
                stderr,
                &format!(
                    "{}: serialization failed",
                    diagnostics::FAILED_TO_WRITE_JSON
                ),
            );
            return AdapterExitCode::IoError.code();
        }
    };
    let value = add_warnings(value, warnings);
    write_json_value(&value, stdout, stderr)
}

fn write_readable_error<W: Write, E: Write>(
    error: &StableError,
    warnings: &[DirectCliWarning],
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let value = add_warnings(stable_error_readable(error), warnings);
    write_json_value(&value, stdout, stderr)
}

fn write_json_value<W: Write, E: Write>(value: &Value, stdout: &mut W, stderr: &mut E) -> i32 {
    match serde_json::to_writer(stdout, value) {
        Ok(()) => AdapterExitCode::Success.code(),
        Err(error) => {
            let _ = emit_diagnostic(
                stderr,
                &format!("{}: {error}", diagnostics::FAILED_TO_WRITE_JSON),
            );
            AdapterExitCode::IoError.code()
        }
    }
}

// ── readable payload construction ──────────────────────────────────────────

/// Map an `OperationResult` variant to the corresponding `ReadableViewKind`.
fn view_kind_for_result(result: &OperationResult) -> ReadableViewKind {
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

// ── stable error → readable JSON ───────────────────────────────────────────

fn stable_error_readable(error: &StableError) -> Value {
    json!({
        "code": error.code,
        "error": error.message,
        "details": error.details,
        "guidance": error.guidance.clone().unwrap_or_default(),
    })
}

// ── warning injection ──────────────────────────────────────────────────────

/// Inject `warnings` into a JSON payload value.
fn add_warnings(mut value: Value, warnings: &[DirectCliWarning]) -> Value {
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

// ── diagnostic helpers (stderr) ────────────────────────────────────────────

/// Append CLI warnings to stderr for protocol-json / manifest / probe commands.
pub(super) fn append_cli_warnings_to_stderr<W: Write>(
    exit_code: i32,
    warnings: &[DirectCliWarning],
    stderr: &mut W,
) -> i32 {
    match write_cli_warnings(warnings, stderr) {
        Ok(()) => exit_code,
        Err(error) => {
            let _ = emit_diagnostic(
                stderr,
                &format!("{}: {error}", diagnostics::FAILED_TO_WRITE_CLI_WARNING),
            );
            AdapterExitCode::IoError.code()
        }
    }
}

/// Write CLI warnings as stderr diagnostic text lines.
///
/// Used for protocol-json, manifest, and probe stderr diagnostics.
/// Not used for document output modes (readable-view / readable-json)
/// where warnings are embedded in the JSON payload.
fn write_cli_warnings<W: Write>(warnings: &[DirectCliWarning], writer: &mut W) -> io::Result<()> {
    for warning in warnings {
        let details = serde_json::to_string(&warning.details).map_err(io::Error::other)?;
        writeln!(
            writer,
            "warning: id={}, effect={}, reason={}, details={}",
            warning.id.as_str(),
            warning.effect.as_str(),
            warning.reason.replace(['\r', '\n'], " "),
            details
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailingStdout {
        attempted: bool,
    }

    impl Write for FailingStdout {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.attempted = true;
            assert!(
                !buffer.is_empty(),
                "stdout write should carry rendered output"
            );
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "stdout closed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn readable_view_renderer_failure_is_internal_error_without_stdout() {
        let value = json!({
            "ref": "missing-content",
            "content_type": "text/plain",
            "cost": "0 lines | 0 bytes",
            "page": null,
        });
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let exit =
            write_readable_view_value(value, ReadableViewKind::Read, &[], &mut stdout, &mut stderr);

        assert_eq!(exit, AdapterExitCode::InternalError.code());
        assert!(stdout.is_empty(), "renderer failures must not write stdout");
        let stderr = String::from_utf8(stderr).expect("stderr utf8");
        assert!(stderr.contains("readable_view_render_failed"));
        assert!(stderr.contains("/content"));
    }

    #[test]
    fn readable_view_stdout_write_failure_is_io_error_with_diagnostic() {
        let value = json!({
            "ref": "ok",
            "content": "body",
            "content_type": "text/plain",
            "cost": "1 lines | 4 bytes",
            "page": null,
        });
        let mut stdout = FailingStdout { attempted: false };
        let mut stderr = Vec::new();

        let exit =
            write_readable_view_value(value, ReadableViewKind::Read, &[], &mut stdout, &mut stderr);

        assert_eq!(exit, AdapterExitCode::IoError.code());
        assert!(
            stdout.attempted,
            "rendered readable-view should be written to stdout"
        );
        let stderr = String::from_utf8(stderr).expect("stderr utf8");
        assert!(stderr.contains("failed to write readable-view output"));
        assert!(stderr.contains("stdout closed"));
    }
}
