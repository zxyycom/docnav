mod code;
mod details;
mod record;
mod stack;

pub use code::{
    typed_codes, BoundaryDiagnosticCode, BoundaryDiagnosticMarker, DiagnosticCategory,
    DiagnosticCode, DiagnosticCodeMarker, DiagnosticEffect, DiagnosticProjectionRule,
    DiagnosticSeverity, ProtocolDiagnosticCode, ProtocolDiagnosticMarker,
};
pub use details::{
    AdapterConfigSourceDetails, AdapterReasonDetails, BoundaryDetails, DetailFieldRule,
    DetailFieldType, DiagnosticDetails, DiagnosticDetailsError, DiagnosticDetailsPayload,
    DiagnosticDetailsRule, FieldReasonDetails, FormatAmbiguousDetails, FormatCandidateDetails,
    FormatUnknownDetails, InternalDetails, PathDetails, PathEncodingDetails, PathReasonDetails,
    RefCandidateCountDetails, RefDetails, RefReasonDetails,
};
pub use record::{
    DiagnosticRecord, DiagnosticRecordDraft, DiagnosticRecordError, DiagnosticSource,
};
pub use stack::{DiagnosticId, DiagnosticMark, DiagnosticStack};

#[cfg(test)]
mod tests;
