mod code;
mod details;
mod record;
mod stack;
mod warning;

pub use code::{
    typed_codes, BoundaryDiagnosticCode, BoundaryDiagnosticMarker, DiagnosticCategory,
    DiagnosticCode, DiagnosticCodeMarker, DiagnosticEffect, DiagnosticProjectionRule,
    DiagnosticSeverity, ProtocolDiagnosticCode, ProtocolDiagnosticMarker,
    ReadableWarningDiagnosticCode, ReadableWarningDiagnosticMarker,
};
pub use details::{
    AdapterCandidateDetails, AdapterConfigSourceDetails, AdapterReasonDetails, BoundaryDetails,
    CapabilityAdapterDetails, CliArgvDetails, DetailFieldRule, DetailFieldType, DiagnosticDetails,
    DiagnosticDetailsError, DiagnosticDetailsPayload, DiagnosticDetailsRule, FieldReasonDetails,
    FormatAmbiguousDetails, FormatUnknownDetails, InternalDetails, PathDetails,
    PathEncodingDetails, PathReasonDetails, RefCandidateCountDetails, RefDetails, RefReasonDetails,
};
pub use record::{
    DiagnosticRecord, DiagnosticRecordDraft, DiagnosticRecordError, DiagnosticSource,
};
pub use stack::{DiagnosticId, DiagnosticMark, DiagnosticStack};
pub use warning::{
    attach_warnings_to_value, warning_text_line, write_warning_text_lines, EmptyWarningReason,
    WarningProjection, ADAPTER_CANDIDATE_FAILURE, ADAPTER_CONFIG_SOURCE_SKIPPED, CLI_ARGV_IGNORED,
};

#[cfg(test)]
mod tests;
