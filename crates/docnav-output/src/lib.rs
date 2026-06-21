use std::fmt;
use std::io::{self, Write};

use docnav_diagnostics::{attach_warnings_to_value, write_warning_text_lines, Warning};
use docnav_json_io::{write_json_value_pretty, JsonIoError};
use docnav_protocol::{
    FailureResponse, Operation, OperationResult, ProtocolResponse, StableError, SuccessResponse,
    PROTOCOL_VERSION,
};
use docnav_readable::{render_readable_view, to_readable_value, ReadableViewKind, RenderError};
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentOutputMode {
    ReadableView,
    ReadableJson,
    ProtocolJson,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentOutputStatus {
    Success,
    Failure(StableError),
}

#[derive(Debug)]
pub enum DocumentOutputError {
    ReadablePayload(RenderError),
    ReadableViewRender(RenderError),
    StdoutJson(JsonIoError),
    StdoutWrite(io::Error),
    StderrWarning(io::Error),
}

impl fmt::Display for DocumentOutputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadablePayload(error) => write!(formatter, "readable payload failed: {error}"),
            Self::ReadableViewRender(error) => {
                write!(formatter, "readable_view_render_failed: {error}")
            }
            Self::StdoutJson(error) => write!(formatter, "failed to write JSON output: {error}"),
            Self::StdoutWrite(error) => write!(formatter, "failed to write output: {error}"),
            Self::StderrWarning(error) => write!(formatter, "failed to write CLI warning: {error}"),
        }
    }
}

impl std::error::Error for DocumentOutputError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadablePayload(error) | Self::ReadableViewRender(error) => Some(error),
            Self::StdoutJson(error) => Some(error),
            Self::StdoutWrite(error) | Self::StderrWarning(error) => Some(error),
        }
    }
}

pub struct ProtocolOutputContext<'a> {
    pub protocol_version: &'a str,
    pub request_id: &'a str,
    pub operation: Option<Operation>,
}

impl<'a> ProtocolOutputContext<'a> {
    pub const fn new(
        protocol_version: &'a str,
        request_id: &'a str,
        operation: Option<Operation>,
    ) -> Self {
        Self {
            protocol_version,
            request_id,
            operation,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DocumentOutputOptions<'a> {
    mode: DocumentOutputMode,
    warnings: &'a [Warning],
}

impl<'a> DocumentOutputOptions<'a> {
    pub const fn new(mode: DocumentOutputMode, warnings: &'a [Warning]) -> Self {
        Self { mode, warnings }
    }
}

pub fn write_document_response<W, E>(
    response: &ProtocolResponse,
    mode: DocumentOutputMode,
    warnings: &[Warning],
    stdout: &mut W,
    stderr: &mut E,
) -> Result<DocumentOutputStatus, DocumentOutputError>
where
    W: Write,
    E: Write,
{
    let output = DocumentOutputOptions::new(mode, warnings);

    if output.mode == DocumentOutputMode::ProtocolJson {
        return write_protocol_response(response, output.warnings, stdout, stderr);
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
    warnings: &[Warning],
    stdout: &mut W,
    stderr: &mut E,
) -> Result<DocumentOutputStatus, DocumentOutputError>
where
    W: Write,
    E: Write,
{
    write_json_value_pretty(response, stdout).map_err(DocumentOutputError::StdoutJson)?;
    write_warning_text_lines(warnings, stderr).map_err(DocumentOutputError::StderrWarning)?;
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
            write_readable_view_value(value, view_kind_for_result(result), output.warnings, stdout)
        }
        DocumentOutputMode::ReadableJson => {
            let value = add_warnings(readable_payload(result)?, output.warnings);
            write_json_value_pretty(&value, stdout).map_err(DocumentOutputError::StdoutJson)
        }
        DocumentOutputMode::ProtocolJson => {
            let response = ProtocolResponse::success(PROTOCOL_VERSION, request_id, result.clone());
            write_json_value_pretty(&response, stdout).map_err(DocumentOutputError::StdoutJson)?;
            write_warning_text_lines(output.warnings, stderr)
                .map_err(DocumentOutputError::StderrWarning)
        }
    }
}

pub fn write_document_error<W, E>(
    error: &StableError,
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

fn write_document_failure<W, E>(
    error: &StableError,
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
        DocumentOutputMode::ReadableView => write_readable_view_value(
            stable_error_readable(error),
            ReadableViewKind::Error,
            output.warnings,
            stdout,
        ),
        DocumentOutputMode::ReadableJson => {
            let value = add_warnings(stable_error_readable(error), output.warnings);
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
            write_warning_text_lines(output.warnings, stderr)
                .map_err(DocumentOutputError::StderrWarning)
        }
    }
}

pub fn readable_payload(result: &OperationResult) -> Result<Value, DocumentOutputError> {
    to_readable_value(result).map_err(DocumentOutputError::ReadablePayload)
}

pub fn view_kind_for_result(result: &OperationResult) -> ReadableViewKind {
    match result {
        OperationResult::Outline(_) => ReadableViewKind::Outline,
        OperationResult::Read(_) => ReadableViewKind::Read,
        OperationResult::Find(_) => ReadableViewKind::Find,
        OperationResult::Info(_) => ReadableViewKind::Info,
    }
}

pub fn stable_error_readable(error: &StableError) -> Value {
    json!({
        "code": error.code,
        "error": error.message,
        "details": error.details,
        "guidance": error.guidance.clone().unwrap_or_default(),
    })
}

pub fn add_warnings(value: Value, warnings: &[Warning]) -> Value {
    attach_warnings_to_value(value, warnings)
}

fn write_readable_view_value<W: Write>(
    value: Value,
    kind: ReadableViewKind,
    warnings: &[Warning],
    stdout: &mut W,
) -> Result<(), DocumentOutputError> {
    let value = add_warnings(value, warnings);
    let rendered = render_readable_view(
        &value,
        kind,
        &docnav_readable::RendererConfig::default_config(),
    )
    .map_err(DocumentOutputError::ReadableViewRender)?;
    stdout
        .write_all(rendered.as_bytes())
        .map_err(DocumentOutputError::StdoutWrite)
}

#[cfg(test)]
mod tests {
    // @case WB-OUTPUT-DOCUMENT-001
    use super::*;
    use docnav_diagnostics::{Warning, CLI_ARGV_IGNORED};
    use docnav_protocol::{Entry, OutlineResult, ReadResult, StableErrorCode};
    use serde_json::json;
    use std::collections::BTreeMap;

    fn warning() -> Warning {
        Warning::cli_argv_ignored(vec!["--future".to_owned()], "unknown CLI flag ignored")
    }

    fn read_result() -> OperationResult {
        OperationResult::Read(ReadResult {
            ref_id: "R1".into(),
            content: "body".into(),
            content_type: "text/plain".into(),
            cost: "1 lines | 4 bytes".into(),
            page: None,
        })
    }

    #[test]
    fn readable_json_success_embeds_warnings_without_protocol_envelope() {
        let result = OperationResult::Outline(OutlineResult {
            entries: vec![Entry {
                ref_id: "R1".into(),
                display: "Intro".into(),
            }],
            page: None,
        });
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        write_document_result(
            &result,
            "request-1",
            DocumentOutputOptions::new(DocumentOutputMode::ReadableJson, &[warning()]),
            &mut stdout,
            &mut stderr,
        )
        .unwrap();

        assert!(stderr.is_empty());
        let value: Value = serde_json::from_slice(&stdout).unwrap();
        assert_eq!(value["entries"][0]["ref"], "R1");
        assert_eq!(value["warnings"][0]["id"], CLI_ARGV_IGNORED.as_str());
        assert!(value.get("protocol_version").is_none());
    }

    #[test]
    fn protocol_json_success_writes_warning_to_stderr_only() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        write_document_result(
            &read_result(),
            "request-1",
            DocumentOutputOptions::new(DocumentOutputMode::ProtocolJson, &[warning()]),
            &mut stdout,
            &mut stderr,
        )
        .unwrap();

        let stdout: Value = serde_json::from_slice(&stdout).unwrap();
        assert_eq!(stdout["request_id"], "request-1");
        assert!(stdout.get("warnings").is_none());
        let stderr = String::from_utf8(stderr).unwrap();
        assert!(stderr.contains("cli_argv_ignored"));
    }

    #[test]
    fn readable_view_read_uses_block_renderer() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        write_document_result(
            &read_result(),
            "request-1",
            DocumentOutputOptions::new(DocumentOutputMode::ReadableView, &[]),
            &mut stdout,
            &mut stderr,
        )
        .unwrap();

        assert!(stderr.is_empty());
        let output = String::from_utf8(stdout).unwrap();
        assert!(output.contains("\"$block\": \"/content\""));
        assert!(output.contains("[block /content bytes=4]"));
        assert!(output.contains("body"));
    }

    #[test]
    fn readable_error_keeps_code_details_and_guidance() {
        let error = StableError {
            code: StableErrorCode::RefNotFound,
            message: "not found".into(),
            details: BTreeMap::from([("ref".to_owned(), json!("R99"))]),
            guidance: Some(vec!["Run outline first.".into()]),
        };
        let value = stable_error_readable(&error);
        assert_eq!(value["code"], "REF_NOT_FOUND");
        assert_eq!(value["details"]["ref"], "R99");
        assert_eq!(value["guidance"][0], "Run outline first.");
    }

    #[test]
    fn response_failure_returns_failure_status() {
        let error = StableError::ref_not_found("R99");
        let response =
            ProtocolResponse::failure(PROTOCOL_VERSION, "request-1", Some(Operation::Read), error);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let status = write_document_response(
            &response,
            DocumentOutputMode::ReadableJson,
            &[],
            &mut stdout,
            &mut stderr,
        )
        .unwrap();

        assert!(matches!(status, DocumentOutputStatus::Failure(_)));
        let value: Value = serde_json::from_slice(&stdout).unwrap();
        assert_eq!(value["code"], "REF_NOT_FOUND");
    }
}
