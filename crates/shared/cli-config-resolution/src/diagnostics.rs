use crate::field::{FieldIdentity, ValidationReason};
use crate::source::{SourceId, SourceLocator};
use crate::value::ReceivedValueKind;

#[derive(Clone, Debug, PartialEq)]
pub struct ResolutionDiagnostic {
    pub field: FieldIdentity,
    pub source_id: Option<SourceId>,
    pub locator: Option<SourceLocator>,
    pub received_kind: Option<ReceivedValueKind>,
    pub reason: DiagnosticReason,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DiagnosticReason {
    SourceInvalid { reason: String },
    ValidationFailed(ValidationReason),
    MissingRequired,
    MergeConflict(MergeConflictReason),
    AmbiguousPriority { priority: i32 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum MergeConflictReason {
    DenyConflict,
    SamePriorityReplace,
}
