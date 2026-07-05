//! Single-path typed payload → `serde_json::Value` API.
//!
//! Document output orchestration decides whether the returned value is written
//! as `readable-json` or passed to the readable-view renderer.

use serde::Serialize;
use serde_json::Value;

use crate::error::RenderError;

/// Convert a typed readable payload into a complete `serde_json::Value`.
///
/// This is the **single entry point** from typed payload to JSON value.
/// The caller owns output mode dispatch.
pub fn to_readable_value<T: Serialize>(payload: &T) -> Result<Value, RenderError> {
    serde_json::to_value(payload).map_err(RenderError::serialization_failed)
}
