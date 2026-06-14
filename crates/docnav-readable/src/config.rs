//! Repo-internal renderer config and block pointer types.
//!
//! The renderer config is a committed, code-owned contract that declares
//! which JSON Pointer fields are rendered as out-of-band blocks per
//! readable view kind. It is **not** user-configurable.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::error::RenderError;
use crate::view_kind::ReadableViewKind;

// ── Block pointer ──────────────────────────────────────────────────────────

/// A validated JSON Pointer that identifies a block-eligible string field
/// within a readable JSON value.
///
/// Valid pointers start with `"/"`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BlockPointer(String);

impl BlockPointer {
    /// Create a new `BlockPointer`, validating that it starts with `"/"`.
    pub fn new(pointer: impl Into<String>) -> Result<Self, RenderError> {
        let s = pointer.into();
        if !s.starts_with('/') {
            return Err(RenderError::config_invalid(format!(
                "block pointer \"{s}\" must start with '/'"
            )));
        }
        Ok(Self(s))
    }

    /// Return the pointer string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for BlockPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// ── View config ────────────────────────────────────────────────────────────

/// Block field declaration for a single readable view kind.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewBlockConfig {
    /// Ordered list of JSON Pointers whose string values are rendered as
    /// out-of-band blocks. An empty list means no blocks (pure JSON header).
    pub blocks: Vec<String>,
}

// ── Renderer config ────────────────────────────────────────────────────────

/// The repo-internal renderer config.
///
/// Maps each readable view kind to its block field declarations.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RendererConfig {
    pub views: BTreeMap<ReadableViewKind, ViewBlockConfig>,
}

impl RendererConfig {
    // ── committed default config ───────────────────────────────────────────

    /// Returns the committed, repo-internal default renderer config.
    ///
    /// This is the **single source of truth** for which JSON fields are
    /// rendered as blocks per view kind.  Changes to this function must be
    /// accompanied by conformance vector updates.
    pub fn default_config() -> Self {
        let mut views = BTreeMap::new();

        // outline – no blocks (all fields in JSON header)
        views.insert(
            ReadableViewKind::Outline,
            ViewBlockConfig { blocks: vec![] },
        );

        // read – content body is the primary block
        views.insert(
            ReadableViewKind::Read,
            ViewBlockConfig {
                blocks: vec!["/content".to_owned()],
            },
        );

        // find – no blocks
        views.insert(ReadableViewKind::Find, ViewBlockConfig { blocks: vec![] });

        // info – no blocks
        views.insert(ReadableViewKind::Info, ViewBlockConfig { blocks: vec![] });

        // readable error – the error message string is the block
        views.insert(
            ReadableViewKind::Error,
            ViewBlockConfig {
                blocks: vec!["/error".to_owned()],
            },
        );

        // warning – no blocks (all fields in JSON header)
        views.insert(
            ReadableViewKind::Warning,
            ViewBlockConfig { blocks: vec![] },
        );

        Self { views }
    }

    // ── validation ─────────────────────────────────────────────────────────

    /// Validate the renderer config, returning an error on the first
    /// violation.
    ///
    /// Checks:
    /// - Every pointer must start with `"/"`.
    /// - No duplicate pointers within a single view config.
    pub fn validate(&self) -> Result<(), RenderError> {
        for (kind, view) in &self.views {
            let mut seen = BTreeSet::new();

            for (i, pointer) in view.blocks.iter().enumerate() {
                // Validate pointer syntax.
                if !pointer.starts_with('/') {
                    return Err(RenderError::config_invalid(format!(
                        "view \"{kind}\" block pointer at index {i} ({pointer:?}) must start with '/'"
                    )));
                }

                // Detect duplicates.
                if !seen.insert(pointer.as_str()) {
                    return Err(RenderError::config_invalid(format!(
                        "view \"{kind}\" contains duplicate block pointer {pointer:?}"
                    )));
                }
            }
        }
        Ok(())
    }

    /// Look up the `ViewBlockConfig` for the given view kind.
    ///
    /// Returns an error if the config has no entry for `kind`.
    pub fn view_config(&self, kind: ReadableViewKind) -> Result<&ViewBlockConfig, RenderError> {
        self.views
            .get(&kind)
            .ok_or_else(|| RenderError::view_config_missing(kind.as_str()))
    }
}
