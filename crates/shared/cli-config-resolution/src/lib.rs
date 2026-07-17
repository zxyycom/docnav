#![forbid(unsafe_code)]
//! Framework-independent source extraction and canonical parameter resolution.
//!
//! Parameter declarations, validation, merge metadata, and typed materialization
//! are owned by [`docnav_typed_fields`] and re-exported from this crate.

mod diagnostics;
mod resolution;
mod source;

pub use diagnostics::{CandidateInvalidReason, DiagnosticReason, ResolutionDiagnostic};
pub use docnav_typed_fields::{
    ActualValueKind, BuildError, CliBooleanEncoding, CliProcessingMetadata, DefaultMetadata,
    ExpectedFieldShape, FieldBound, FieldDef, FieldDefBuilder, FieldDefSet, FieldDefSetBuildError,
    FieldDefSetBuilder, FieldIdentity, FieldLength, FieldPath, FieldStringEnum, FieldValidation,
    FieldValidationErrors, FieldValueMap, JsonValue, MergeStrategy, ProcessStrategy, ProcessingId,
    ProcessingInputKind, ProcessingLocator, SchemaMetadataView, TypedValue, ValidationFailure,
    ValidationReason, ValueKind,
};
pub use resolution::{
    resolve, CandidateTrace, FieldResolution, FieldTrace, MaterializationError,
    ResolutionInputError, ResolutionResult,
};
pub use source::{
    extract_env, CandidateInput, Source, SourceCandidate, SourceError, SourceId, SourceKind,
    SourceLocator,
};
