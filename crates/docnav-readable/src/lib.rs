//! `docnav-readable` — readable payload and readable-view rendering helpers.
//!
//! This crate provides:
//!
//! - A **single-path** typed-payload → `serde_json::Value` API so that
//!   `readable-json` and `readable-view` derive from the same complete value.
//! - A **repo-internal renderer config** that declares which JSON Pointer
//!   fields should be rendered as out-of-band blocks per view kind.
//! - A **readable-view renderer** that emits a pretty JSON header followed
//!   by length-delimited `[block …]` / `[endblock …]` sections.
//! - **Conformance vectors** and test DTOs for verifying renderer correctness.
//!
//! # Architecture boundary
//!
//! This crate owns readable payload/value conversion, renderer config,
//! `ReadableViewKind`, readable-view block framing, and conformance vectors.
//! It does **not** own output mode dispatch, protocol envelopes, warning
//! placement, adapter routing, document parsing, or CLI/MCP wiring.

pub mod config;
pub mod conformance;
pub mod error;
pub mod renderer;
pub mod value;
pub mod view_kind;

// Re-export key types for convenience.
pub use config::{BlockPointer, RendererConfig, ViewBlockConfig};
pub use error::RenderError;
pub use renderer::{render_readable_view, RenderedBlock};
pub use value::to_readable_value;
pub use view_kind::ReadableViewKind;
