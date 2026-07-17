use docnav_protocol::{Operation, PositiveInteger};
use docnav_typed_fields::ValueKind;

/// Prepared input for an outline strategy.
///
/// Navigation constructs this value after source resolution and standard type
/// materialization. Adapter-specific semantic validation may still reject it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutlineInput {
    /// Normalized document path selected for the operation.
    pub document_path: String,
    /// One-based page to read.
    pub page: PositiveInteger,
    /// Maximum result budget for the page.
    pub limit: PositiveInteger,
    /// Resolved adapter-scoped heading limit, when the selected catalog view
    /// binds one for this operation.
    pub max_heading_level: Option<i64>,
}

/// Prepared input for a read strategy.
///
/// Navigation constructs this value after source resolution and standard type
/// materialization. Adapter-specific semantic validation may still reject it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadInput {
    /// Normalized document path selected for the operation.
    pub document_path: String,
    /// Opaque adapter-owned ref to read.
    pub ref_id: String,
    /// One-based page to read.
    pub page: PositiveInteger,
    /// Maximum content budget for the page.
    pub limit: PositiveInteger,
}

/// Prepared input for a find strategy.
///
/// Navigation constructs this value after source resolution and standard type
/// materialization. Adapter-specific semantic validation may still reject it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FindInput {
    /// Normalized document path selected for the operation.
    pub document_path: String,
    /// Query text to match.
    pub query: String,
    /// One-based page to read.
    pub page: PositiveInteger,
    /// Maximum result budget for the page.
    pub limit: PositiveInteger,
    /// Resolved adapter-scoped heading limit, when the selected catalog view
    /// binds one for this operation.
    pub max_heading_level: Option<i64>,
}

/// Prepared input for an info strategy.
///
/// Navigation constructs this value after source resolution and standard type
/// materialization. Adapter-specific semantic validation may still reject it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InfoInput {
    /// Normalized document path selected for the operation.
    pub document_path: String,
}

/// Closed strategy-input variants shared by navigation and linked adapters.
///
/// This enum contains prepared operation facts only. It does not expose raw
/// sources, protocol envelopes, parameter declarations, or a generic value
/// lookup surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StandardOperationInput {
    Outline(OutlineInput),
    Read(ReadInput),
    Find(FindInput),
    Info(InfoInput),
}

impl StandardOperationInput {
    /// Returns the operation represented by this prepared input.
    pub const fn operation(&self) -> Operation {
        match self {
            Self::Outline(_) => Operation::Outline,
            Self::Read(_) => Operation::Read,
            Self::Find(_) => Operation::Find,
            Self::Info(_) => Operation::Info,
        }
    }
}

/// Compile-time catalog targets for strategy-visible standard input fields.
///
/// Fixed operation facts such as document path, ref, and query are deliberately
/// absent: navigation maps them directly rather than through the product
/// parameter catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardInputBinding {
    OutlinePage,
    OutlineLimit,
    OutlineMaxHeadingLevel,
    ReadPage,
    ReadLimit,
    FindPage,
    FindLimit,
    FindMaxHeadingLevel,
}

impl StandardInputBinding {
    /// Returns the only operation whose standard input contains this target.
    pub const fn operation(self) -> Operation {
        match self {
            Self::OutlinePage | Self::OutlineLimit | Self::OutlineMaxHeadingLevel => {
                Operation::Outline
            }
            Self::ReadPage | Self::ReadLimit => Operation::Read,
            Self::FindPage | Self::FindLimit | Self::FindMaxHeadingLevel => Operation::Find,
        }
    }

    /// Returns the canonical typed-field kind accepted by this target.
    pub const fn expected_value_kind(self) -> ValueKind {
        match self {
            Self::OutlinePage
            | Self::OutlineLimit
            | Self::OutlineMaxHeadingLevel
            | Self::ReadPage
            | Self::ReadLimit
            | Self::FindPage
            | Self::FindLimit
            | Self::FindMaxHeadingLevel => ValueKind::Integer,
        }
    }
}
