//! Conformance vector types for cross-implementation acceptance testing.
//!
//! Conformance vectors are committed JSON fixtures that describe input
//! payloads, view kinds, renderer config overrides, and order-independent
//! assertions.  Implementations (Rust or future ports)
//! consume the same vector files and verify the same semantic assertions.
//!
//! # Stable assertion scope
//!
//! Assertions focus on **block pointer**, **byte length**, and **block
//! payload**.  The following are **excluded** from the stable contract and
//! MUST NOT be asserted:
//!
//! - Header JSON key order
//! - Multi-block output order
//! - Byte-for-byte output consistency
//!
//! # Vector file format
//!
//! Each JSON file under `tests/fixtures/conformance/` contains a single
//! conformance vector with this shape:
//!
//! ```json
//! {
//!   "description": "...",
//!   "view_kind": "read",
//!   "config_override": null,
//!   "expected_failure": null,
//!   "input": { ... },
//!   "assertions": [
//!     {"type": "block", "pointer": "/content", "byte_length": 42,
//!      "payload": "exact payload", "payload_contains": "expected text"},
//!     {"type": "no_blocks"},
//!     {"type": "header_field", "pointer": "/ref", "value": "L5"},
//!     {"type": "header_contains", "text": "\"ref\""},
//!     {"type": "output_contains", "text": "[block /content"},
//!     {"type": "output_not_contains", "text": "[block"},
//!     {"type": "no_cr_in_framing"}
//!   ]
//! }
//! ```

use serde::Deserialize;
use serde_json::Value;

// ── Conformance vector ─────────────────────────────────────────────────────

/// A single committed conformance vector.
///
/// Describes an input value, view kind, optional config override, optional
/// expected failure, and a list of order-independent assertions.
#[derive(Clone, Debug, Deserialize)]
pub struct ConformanceVector {
    /// Human-readable description of what this vector tests.
    pub description: String,
    /// The readable view kind: `"outline"`, `"read"`, `"find"`, `"info"`,
    /// `"error"`, or `"warning"`.
    pub view_kind: String,
    /// Optional block-pointer override for the view kind.
    /// When present, replaces the default `ViewBlockConfig::blocks` for the
    /// view kind in this vector.
    #[serde(default)]
    pub config_override: Option<ConfigOverride>,
    /// When present, the renderer is expected to fail with the given
    /// error id and optional message substring.
    #[serde(default)]
    pub expected_failure: Option<ExpectedFailure>,
    /// The JSON value to pass to the renderer.
    pub input: Value,
    /// Order-independent assertions to verify against the renderer output.
    pub assertions: Vec<Assertion>,
}

// ── Config override ────────────────────────────────────────────────────────

/// Override the block-pointer list for a single view kind within one vector.
#[derive(Clone, Debug, Deserialize)]
pub struct ConfigOverride {
    /// JSON Pointer strings declaring which fields are block-eligible.
    pub blocks: Vec<String>,
}

// ── Expected failure ───────────────────────────────────────────────────────

/// Describes an expected renderer failure.
#[derive(Clone, Debug, Deserialize)]
pub struct ExpectedFailure {
    /// The stable error id, e.g. `"readable_view_render_failed"`.
    pub error_id: String,
    /// An optional substring that must appear in the error message.
    #[serde(default)]
    pub message_contains: Option<String>,
}

// ── Assertions ─────────────────────────────────────────────────────────────

/// An individual order-independent assertion.
///
/// Each variant carries only the fields that are stable across
/// implementations.  Variants are deserialized from the `"type"` field
/// using `rename_all = "snake_case"`.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Assertion {
    /// A block section with the given pointer exists in the output.
    ///
    /// - `pointer` — the JSON Pointer (e.g. `"/content"`).
    /// - `byte_length` — exact UTF-8 byte length of the block payload
    ///   (when present, the rendered block's `bytes=` value must match).
    /// - `payload_contains` — a substring that must appear in the block
    ///   payload (when present).
    /// - `payload` — the exact block payload (when present).
    Block {
        pointer: String,
        #[serde(default)]
        byte_length: Option<u64>,
        #[serde(default)]
        payload: Option<String>,
        #[serde(default)]
        payload_contains: Option<String>,
    },
    /// No `[block …]` / `[endblock …]` sections appear in the output.
    NoBlocks,
    /// A field at the given JSON Pointer in the **header JSON** has the
    /// expected `Value`.  The comparison is structural and does NOT depend
    /// on key order within objects.
    HeaderField {
        /// JSON Pointer into the header value.
        pointer: String,
        /// Expected value after JSON deserialization.
        value: Value,
    },
    /// The header JSON (pretty-printed) contains the given text substring.
    HeaderContains { text: String },
    /// The full output string contains the given text substring.
    OutputContains { text: String },
    /// The full output string does NOT contain the given text substring.
    OutputNotContains { text: String },
    /// The output framing contains no CR (`\r`, 0x0D) bytes.
    NoCrInFraming,
}
