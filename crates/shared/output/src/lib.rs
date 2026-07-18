use std::fmt;
use std::io;

use docnav_json_io::JsonIoError;
use docnav_protocol::ProtocolResponse;

mod readable;
mod writer;

pub use readable::render_readable_response;
pub use writer::write_document_response;

pub type RenderStrategy = fn(&ProtocolResponse) -> Result<String, RenderFailure>;

#[derive(Clone, Copy, Debug)]
pub enum OutputPlan {
    ProtocolJson,
    Rendered(RenderStrategy),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderFailure {
    message: String,
}

impl RenderFailure {
    pub const ERROR_ID: &'static str = "readable_view_render_failed";

    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl From<docnav_readable::RenderError> for RenderFailure {
    fn from(error: docnav_readable::RenderError) -> Self {
        Self::new(error.to_string())
    }
}

impl fmt::Display for RenderFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RenderFailure {}

#[derive(Debug)]
pub enum DocumentOutputError {
    Render(RenderFailure),
    StdoutJson(JsonIoError),
    StdoutWrite(io::Error),
}

impl DocumentOutputError {
    pub fn primary_error_id(&self) -> &'static str {
        match self {
            Self::Render(_) => RenderFailure::ERROR_ID,
            Self::StdoutJson(_) => "stdout-json-write-failed",
            Self::StdoutWrite(_) => "stdout-write-failed",
        }
    }
}

impl fmt::Display for DocumentOutputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Render(error) => {
                write!(formatter, "readable_view_render_failed: {error}")
            }
            Self::StdoutJson(error) => write!(formatter, "failed to write JSON output: {error}"),
            Self::StdoutWrite(error) => write!(formatter, "failed to write output: {error}"),
        }
    }
}

impl std::error::Error for DocumentOutputError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Render(error) => Some(error),
            Self::StdoutJson(error) => Some(error),
            Self::StdoutWrite(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests;
