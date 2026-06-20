//! Lightweight payload DTOs used by readable renderer tests.
//!
//! These are not production types. Production payloads live in other crates and
//! implement `Serialize` for `to_readable_value`.

use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TestReadPayload {
    #[serde(rename = "ref")]
    pub ref_id: String,
    pub content: String,
    pub content_type: String,
    pub cost: String,
    pub page: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TestOutlinePayload {
    pub entries: Vec<TestEntry>,
    pub page: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TestEntry {
    #[serde(rename = "ref")]
    pub ref_id: String,
    pub display: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TestFindPayload {
    pub matches: Vec<TestEntry>,
    pub page: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TestInfoPayload {
    pub display: String,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TestErrorPayload {
    pub code: String,
    pub error: String,
    pub details: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidance: Option<Vec<String>>,
}
