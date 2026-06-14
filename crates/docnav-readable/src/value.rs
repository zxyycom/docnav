//! Single-path typed payload → `serde_json::Value` API.
//!
//! Both `readable-json` and `readable-view` outputs derive from the same
//! complete JSON value produced by [`to_readable_value`].

use serde::Serialize;
use serde_json::Value;

use crate::error::RenderError;

/// Convert a typed readable payload into a complete `serde_json::Value`.
///
/// This is the **single entry point** from typed payload to JSON value.
/// - For `readable-json`: the returned value is serialized directly.
/// - For `readable-view`: the same value is passed to the renderer which
///   applies block replacement and framing according to the renderer config.
pub fn to_readable_value<T: Serialize>(payload: &T) -> Result<Value, RenderError> {
    serde_json::to_value(payload).map_err(RenderError::serialization_failed)
}

// ── Test DTOs ──────────────────────────────────────────────────────────────
//
// Lightweight payload structs used in unit tests and conformance vectors.
// These are NOT the production types; production types live in other crates
// and implement `Serialize` to work with `to_readable_value`.

/// Test DTO for `read` payloads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TestReadPayload {
    #[serde(rename = "ref")]
    pub ref_id: String,
    pub content: String,
    pub content_type: String,
    pub cost: String,
    pub page: Option<u32>,
}

/// Test DTO for `outline` payloads (no block fields expected).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TestOutlinePayload {
    pub entries: Vec<TestEntry>,
    pub page: Option<u32>,
}

/// Test DTO for a single outline entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TestEntry {
    #[serde(rename = "ref")]
    pub ref_id: String,
    pub display: String,
}

/// Test DTO for `find` payloads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TestFindPayload {
    pub matches: Vec<TestEntry>,
    pub page: Option<u32>,
}

/// Test DTO for `info` payloads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TestInfoPayload {
    pub display: String,
    pub capabilities: Vec<String>,
}

/// Test DTO for readable error payloads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TestErrorPayload {
    pub code: String,
    pub error: String,
    pub details: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidance: Option<Vec<String>>,
}

/// Test DTO for readable warning payloads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TestWarningPayload {
    pub code: String,
    pub warning: String,
    pub details: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidance: Option<Vec<String>>,
}
