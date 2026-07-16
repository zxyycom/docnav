use docnav_typed_fields::{FieldIdentity, JsonValue, ValidationFailure, ValueKind};

use crate::source::{SourceId, SourceKind, SourceLocator};

#[derive(Clone, Debug, PartialEq)]
pub struct ResolutionDiagnostic {
    pub field: FieldIdentity,
    pub source_id: Option<SourceId>,
    pub source_kind: Option<SourceKind>,
    pub locator: Option<SourceLocator>,
    pub raw: Option<JsonValue>,
    pub reason: DiagnosticReason,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DiagnosticReason {
    InvalidCandidate(CandidateInvalidReason),
    FinalValidation(ValidationFailure),
    MissingRequired(ValidationFailure),
    MergeConflict(Vec<SourceLocator>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum CandidateInvalidReason {
    Decode(String),
    Shape { expected: ValueKind },
    Validation(ValidationFailure),
}
