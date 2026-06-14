//! Renderer error types for `docnav-readable`.
//!
//! All renderer failures use a single stable error id
//! `readable_view_render_failed` and carry a diagnostic message.

use std::fmt;

/// Represents a failure in readable-view rendering.
///
/// The error `id` is always `"readable_view_render_failed"`.
/// The `message` provides a human-readable diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderError {
    pub id: String,
    pub message: String,
}

impl RenderError {
    /// The stable renderer error id used by all readable-view failures.
    pub const ERROR_ID: &'static str = "readable_view_render_failed";

    /// Create a render error with the stable error id and a diagnostic message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            id: Self::ERROR_ID.to_owned(),
            message: message.into(),
        }
    }

    /// Create a render error for JSON serialization failure of the readable value.
    pub fn serialization_failed(source: impl fmt::Display) -> Self {
        Self::new(format!(
            "failed to serialize typed payload to readable JSON value: {source}"
        ))
    }

    /// Create a render error for a config validation failure.
    pub fn config_invalid(reason: impl Into<String>) -> Self {
        Self::new(format!(
            "renderer config validation failed: {}",
            reason.into()
        ))
    }

    /// Create a render error for a missing view config.
    pub fn view_config_missing(kind: &str) -> Self {
        Self::new(format!(
            "renderer config missing entry for view kind \"{kind}\""
        ))
    }

    /// Create a render error for a block pointer that does not resolve to a string field.
    pub fn block_field_not_string(pointer: &str) -> Self {
        Self::new(format!(
            "block pointer \"{pointer}\" does not resolve to a string field in the readable payload"
        ))
    }

    /// Create a render error for a block pointer that is missing from the value.
    pub fn block_field_missing(pointer: &str) -> Self {
        Self::new(format!(
            "block pointer \"{pointer}\" not found in the readable payload"
        ))
    }

    /// Create a render error for JSON header serialization failure.
    pub fn header_serialization_failed(source: impl fmt::Display) -> Self {
        Self::new(format!(
            "failed to serialize readable-view header JSON: {source}"
        ))
    }
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "[{}] {}", self.id, self.message)
    }
}

impl std::error::Error for RenderError {}
