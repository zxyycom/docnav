use docnav_diagnostics::{
    typed_codes, CapabilityAdapterDetails, DiagnosticCode, DiagnosticRecordDraft, DiagnosticSource,
    DiagnosticStack, FieldReasonDetails, InternalDetails, PathDetails, PathEncodingDetails,
    PathReasonDetails, RefDetails, RefReasonDetails,
};
use docnav_protocol::{
    protocol_error_record_draft, protocol_error_record_draft_with_summary, FindArguments,
    FindResult, InfoArguments, InfoResult, Manifest, Operation, OutlineArguments, OutlineResult,
    ProbeResult, ProtocolError, ReadArguments, ReadResult, RequestEnvelope,
};

mod native_options;

pub use native_options::{
    NativeOptionIssue, NativeOptionSpec, NativeOptionValueKind, NativeOptionValueSpec,
};

pub type AdapterResult<T> = Result<T, AdapterError>;

pub trait Adapter: Sync {
    fn adapter_id(&self) -> &str;

    fn manifest(&self) -> Manifest;

    fn native_options(&self) -> &'static [NativeOptionSpec] {
        &[]
    }

    fn probe(&self, path: &str) -> ProbeResult;

    fn outline(
        &self,
        _request: &RequestEnvelope,
        _arguments: &OutlineArguments,
    ) -> AdapterResult<OutlineResult> {
        Err(self.unsupported(Operation::Outline))
    }

    fn read(
        &self,
        _request: &RequestEnvelope,
        _arguments: &ReadArguments,
    ) -> AdapterResult<ReadResult> {
        Err(self.unsupported(Operation::Read))
    }

    fn find(
        &self,
        _request: &RequestEnvelope,
        _arguments: &FindArguments,
    ) -> AdapterResult<FindResult> {
        Err(self.unsupported(Operation::Find))
    }

    fn info(
        &self,
        _request: &RequestEnvelope,
        _arguments: &InfoArguments,
    ) -> AdapterResult<InfoResult> {
        Err(self.unsupported(Operation::Info))
    }

    fn unsupported(&self, operation: Operation) -> AdapterError {
        AdapterError::capability_unsupported(operation, self.adapter_id())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AdapterError {
    diagnostic: Box<DiagnosticRecordDraft>,
}

impl AdapterError {
    pub fn new(diagnostic: DiagnosticRecordDraft) -> Self {
        let diagnostic = normalize_protocol_diagnostic(diagnostic);
        Self {
            diagnostic: Box::new(diagnostic),
        }
    }

    pub fn diagnostic(&self) -> &DiagnosticRecordDraft {
        self.diagnostic.as_ref()
    }

    pub fn into_diagnostic(self) -> DiagnosticRecordDraft {
        *self.diagnostic
    }

    pub fn protocol_error(&self) -> ProtocolError {
        protocol_error_from_diagnostic(self.diagnostic.as_ref().clone())
    }

    pub fn invalid_request(field: impl Into<String>, reason: impl Into<String>) -> Self {
        let reason = reason.into();
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::InvalidRequest,
        >(
            FieldReasonDetails::new(field, reason),
            DiagnosticSource::with_stage("docnav-adapter-contracts", "adapter"),
        ))
    }

    pub fn native_option_invalid(
        summary: impl Into<String>,
        issue: NativeOptionIssue,
        guidance: impl IntoIterator<Item = String>,
    ) -> Self {
        let mut details = FieldReasonDetails::new(issue.field.clone(), issue.reason_code.clone());
        details.received = issue.received.clone();
        details.accepted = issue.expected.clone().map(|value| vec![value]);
        details.option_issues = Some(vec![issue.into_json()]);
        Self::new(
            protocol_error_record_draft_with_summary::<typed_codes::protocol::InvalidRequest>(
                summary,
                details,
                DiagnosticSource::with_stage("adapter", "options"),
            )
            .with_guidance(guidance),
        )
    }

    pub fn document_not_found(path: impl Into<String>) -> Self {
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::DocumentNotFound,
        >(
            PathDetails::new(path),
            DiagnosticSource::with_stage("docnav-adapter-contracts", "adapter"),
        ))
    }

    pub fn document_path_invalid(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::DocumentPathInvalid,
        >(
            PathReasonDetails::new(path, reason),
            DiagnosticSource::with_stage("docnav-adapter-contracts", "adapter"),
        ))
    }

    pub fn document_encoding_unsupported(
        path: impl Into<String>,
        encoding: impl Into<String>,
    ) -> Self {
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::DocumentEncodingUnsupported,
        >(
            PathEncodingDetails::new(path, encoding),
            DiagnosticSource::with_stage("docnav-adapter-contracts", "adapter"),
        ))
    }

    pub fn ref_not_found(ref_id: impl Into<String>) -> Self {
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::RefNotFound,
        >(
            RefDetails::new(ref_id),
            DiagnosticSource::with_stage("docnav-adapter-contracts", "adapter"),
        ))
    }

    pub fn ref_invalid(ref_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::RefInvalid,
        >(
            RefReasonDetails::new(ref_id, reason),
            DiagnosticSource::with_stage("docnav-adapter-contracts", "adapter"),
        ))
    }

    pub fn capability_unsupported(operation: Operation, adapter_id: impl Into<String>) -> Self {
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::CapabilityUnsupported,
        >(
            CapabilityAdapterDetails::new(operation.to_string(), adapter_id),
            DiagnosticSource::with_stage("docnav-adapter-contracts", "adapter"),
        ))
    }

    pub fn internal(error_id: impl Into<String>) -> Self {
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::InternalError,
        >(
            InternalDetails::new(error_id),
            DiagnosticSource::with_stage("docnav-adapter-contracts", "adapter"),
        ))
    }
}

impl From<DiagnosticRecordDraft> for AdapterError {
    fn from(diagnostic: DiagnosticRecordDraft) -> Self {
        Self::new(diagnostic)
    }
}

pub fn protocol_error_from_diagnostic(diagnostic: DiagnosticRecordDraft) -> ProtocolError {
    let mut diagnostics = DiagnosticStack::new();
    let Some(id) = diagnostics.push(diagnostic).ok() else {
        return diagnostic_projection_failed();
    };
    let Some(record) = diagnostics.get(id) else {
        return diagnostic_projection_failed();
    };
    ProtocolError::from_diagnostic_record(record).unwrap_or_else(diagnostic_projection_failed)
}

fn normalize_protocol_diagnostic(diagnostic: DiagnosticRecordDraft) -> DiagnosticRecordDraft {
    if is_valid_protocol_diagnostic(&diagnostic) {
        return diagnostic;
    }

    let error_id = if matches!(diagnostic.code(), DiagnosticCode::Protocol(_)) {
        "adapter-error-diagnostic-invalid"
    } else {
        "adapter-error-diagnostic-not-protocol"
    };
    internal_diagnostic(error_id)
}

fn is_valid_protocol_diagnostic(diagnostic: &DiagnosticRecordDraft) -> bool {
    matches!(diagnostic.code(), DiagnosticCode::Protocol(_))
        && !diagnostic.summary().is_empty()
        && diagnostic
            .code()
            .details_rule()
            .validate_value(&diagnostic.details().to_value())
            .is_ok()
}

fn diagnostic_projection_failed() -> ProtocolError {
    ProtocolError::internal_error("adapter-diagnostic-projection-failed")
}

fn internal_diagnostic(error_id: impl Into<String>) -> DiagnosticRecordDraft {
    protocol_error_record_draft::<typed_codes::protocol::InternalError>(
        InternalDetails::new(error_id),
        DiagnosticSource::with_stage("docnav-adapter-contracts", "adapter-error"),
    )
}

#[cfg(test)]
// @case WB-CONTRACTS-ERROR-001
mod tests;
