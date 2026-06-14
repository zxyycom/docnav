//! Readable view kind — one variant per document operation / error.
//!
//! Used by the renderer config to select the correct block-field list.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Identifies the kind of readable view for renderer config lookup.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReadableViewKind {
    Outline,
    Read,
    Find,
    Info,
    /// Represents a readable error payload.
    Error,
    /// Represents a readable warning payload.
    Warning,
}

impl ReadableViewKind {
    /// Lowercase string used for diagnostics and config keys.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Outline => "outline",
            Self::Read => "read",
            Self::Find => "find",
            Self::Info => "info",
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

impl fmt::Display for ReadableViewKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
