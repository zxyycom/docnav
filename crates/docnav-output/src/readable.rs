use std::io::Write;

use docnav_diagnostics::{attach_warnings_to_value, WarningProjection};
use docnav_protocol::{OperationResult, ProtocolError};
use docnav_readable::{render_readable_view, to_readable_value, ReadableViewKind};
use serde_json::{json, Value};

use crate::DocumentOutputError;

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

pub fn protocol_error_readable(error: &ProtocolError) -> Value {
    json!({
        "code": error.code().protocol_code(),
        "error": error.message(),
        "details": error.details(),
        "guidance": error.guidance().unwrap_or_default(),
    })
}

pub fn add_warnings(value: Value, warnings: &[WarningProjection]) -> Value {
    attach_warnings_to_value(value, warnings)
}

pub(crate) fn write_readable_view_value<W: Write>(
    value: Value,
    kind: ReadableViewKind,
    warnings: &[WarningProjection],
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
