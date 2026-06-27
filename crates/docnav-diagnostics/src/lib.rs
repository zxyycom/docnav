mod code;
mod details;
mod record;
mod stack;
mod warning;

pub use code::{
    BoundaryDiagnosticCode, DiagnosticCategory, DiagnosticCode, DiagnosticEffect,
    DiagnosticProjectionRule, DiagnosticSeverity, ProtocolDiagnosticCode,
    ReadableWarningDiagnosticCode,
};
pub use details::{
    DetailFieldRule, DetailFieldType, DiagnosticDetails, DiagnosticDetailsError,
    DiagnosticDetailsRule,
};
pub use record::{
    DiagnosticRecord, DiagnosticRecordDraft, DiagnosticRecordError, DiagnosticSource,
};
pub use stack::{DiagnosticId, DiagnosticMark, DiagnosticStack};
pub use warning::{
    attach_warnings_to_value, warning_text_line, write_warning_text_lines, EmptyWarningReason,
    InvalidWarningId, Warning, WarningDetails, WarningEffect, WarningId, ADAPTER_CANDIDATE_FAILURE,
    ADAPTER_CONFIG_SOURCE_SKIPPED, CLI_ARGV_IGNORED,
};

#[cfg(test)]
mod tests;
