//! Typed payload → private readable `serde_json::Value` conversion.
//!
//! Readable presentation code passes the returned value to the readable-view
//! renderer. Document output orchestration owns output plan selection.

use serde::Serialize;
use serde_json::Value;

use crate::error::RenderError;

/// Convert a typed readable payload into a complete `serde_json::Value`.
///
/// The returned value is an internal renderer input, not a public output mode.
pub fn to_readable_value<T: Serialize>(payload: &T) -> Result<Value, RenderError> {
    serde_json::to_value(payload).map_err(RenderError::serialization_failed)
}
