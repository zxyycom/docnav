mod code;
mod details;
mod record;

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
    DiagnosticId, DiagnosticRecord, DiagnosticRecordDraft, DiagnosticRecordError, DiagnosticSource,
};

#[cfg(test)]
mod tests;
