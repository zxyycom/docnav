use std::io::Write;

use docnav_diagnostics::{attach_warnings_to_value, Warning};
use docnav_protocol::{OperationResult, StableError};
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

pub(crate) fn write_readable_view_value<W: Write>(
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
