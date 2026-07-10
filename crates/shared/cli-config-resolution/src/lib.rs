#![forbid(unsafe_code)]
//! Framework-independent CLI/config resolution core.
//!
//! Derive macros and framework integrations such as `clap` or serde-backed
//! config loading belong in companion crates.

mod diagnostics;
mod explain;
mod field;
mod resolution;
mod source;
mod value;

pub use diagnostics::{DiagnosticReason, MergeConflictReason, ResolutionDiagnostic};
pub use explain::{CandidateExplanation, FieldExplanation, ResolutionExplanation};
pub use field::{
    DefaultMetadata, DynamicDefaultMetadata, FieldBuildError, FieldConstraints, FieldContract,
    FieldContractBuilder, FieldIdentity, FieldProjection, FieldProjectionDeclaration, FieldSet,
    FieldSetBuildError, FieldSetBuilder, ProjectionKey, ValidationFailure, ValidationReason,
};
pub use resolution::{
    CandidateTrace, CandidateTraceState, FieldResolution, FieldTrace, MaterializationError,
    MergeStrategy, ResolutionResult, ResolvedValueMap, Resolver,
};
pub use source::{
    CandidateState, CliFlagSource, ConfigDocumentSource, ConfigPath, ConfigPathError, CustomSource,
    DefaultSource, EnvVarSource, RawSourceValue, SourceCandidate, SourceCollection,
    SourceCollectionError, SourceExplicitness, SourceExtractor, SourceId, SourceKind,
    SourceLoadState, SourceLocator, SourceSpec,
};
pub use value::{ReceivedValueKind, Value, ValueKind, ValueMap};

#[cfg(test)]
mod tests;
