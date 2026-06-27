use std::fmt;
use std::io;

use docnav_diagnostics::{DiagnosticRecord, WarningProjection};
use docnav_json_io::JsonIoError;
use docnav_protocol::{Operation, ProtocolError};
use docnav_readable::RenderError;

mod readable;
mod writer;

pub use readable::{add_warnings, protocol_error_readable, readable_payload, view_kind_for_result};
pub use writer::{
    write_document_diagnostic_error, write_document_error, write_document_response,
    write_document_result,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentOutputMode {
    ReadableView,
    ReadableJson,
    ProtocolJson,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentOutputStatus {
    Success,
    Failure(ProtocolError),
}

#[derive(Debug)]
pub enum DocumentOutputError {
    DiagnosticProjection,
    ReadablePayload(RenderError),
    ReadableViewRender(RenderError),
    StdoutJson(JsonIoError),
    StdoutWrite(io::Error),
    StderrWarning(io::Error),
}

impl fmt::Display for DocumentOutputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DiagnosticProjection => {
                formatter.write_str("failed to project diagnostic output")
            }
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
            Self::DiagnosticProjection => None,
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
    diagnostics: &'a [DiagnosticRecord],
}

impl<'a> DocumentOutputOptions<'a> {
    pub const fn new(mode: DocumentOutputMode, diagnostics: &'a [DiagnosticRecord]) -> Self {
        Self { mode, diagnostics }
    }

    pub(crate) fn warning_projections(self) -> Vec<WarningProjection> {
        self.diagnostics
            .iter()
            .filter_map(WarningProjection::from_record)
            .collect()
    }
}

#[cfg(test)]
mod tests;
