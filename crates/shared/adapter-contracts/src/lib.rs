//! Linked adapter strategy contract for Docnav navigation.
//!
//! [`AdapterDefinition`] combines manifest identity, declared capabilities, and one strategy.
//! Strategies receive closed operation-specific input and retain probe, navigation, and optional
//! unstructured full-read hooks. Parameter declaration and source resolution remain outside this
//! crate.

use docnav_diagnostics::{
    typed_codes, DiagnosticRecordDraft, DiagnosticSource, FieldReasonDetails, InternalDetails,
    PathDetails, PathEncodingDetails, PathReasonDetails, RefDetails, RefReasonDetails,
};
use docnav_protocol::{
    normalize_protocol_diagnostic, protocol_error_record_draft,
    protocol_error_record_draft_with_summary, Cost, FindResult, InfoResult, OutlineResult,
    ProbeResult, ProtocolDiagnosticFallback, ProtocolError, ReadResult, RequestEnvelope,
};
mod definition;
mod native_option;
mod operation_input;

pub use definition::{AdapterDefinition, AdapterDefinitionError};
pub use native_option::NativeOptionIssue;
pub use operation_input::{
    FindInput, InfoInput, OutlineInput, ReadInput, StandardInputBinding, StandardOperationInput,
};

pub type AdapterResult<T> = Result<T, AdapterError>;

pub trait Adapter: Sync {
    fn probe(&self, path: &str) -> ProbeResult;

    fn outline(&self, input: &OutlineInput) -> AdapterResult<OutlineResult>;

    fn read(&self, input: &ReadInput) -> AdapterResult<ReadResult>;

    fn find(&self, input: &FindInput) -> AdapterResult<FindResult>;

    fn info(&self, input: &InfoInput) -> AdapterResult<InfoResult>;

    fn unstructured_full_read(
        &self,
        _request: &RequestEnvelope,
    ) -> AdapterResult<UnstructuredFullRead> {
        Err(AdapterError::internal(
            "unstructured-full-read-hook-unavailable",
        ))
    }

    fn measure_unstructured_full_read_cost(
        &self,
        _request: &RequestEnvelope,
        _requested_units: &[String],
    ) -> AdapterResult<Cost> {
        Ok(Cost {
            measurements: Vec::new(),
        })
    }

    fn unstructured_full_read_facts(
        &self,
        _request: &RequestEnvelope,
    ) -> AdapterResult<UnstructuredFullReadFacts> {
        Ok(UnstructuredFullReadFacts::default())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UnstructuredFullReadCapabilities {
    pub content_hook: bool,
    pub cost_measurement_units: Vec<String>,
    pub result_facts_hook: bool,
}

impl UnstructuredFullReadCapabilities {
    pub fn has_cost_measurement_unit(&self, unit: &str) -> bool {
        self.cost_measurement_units
            .iter()
            .any(|declared| declared == unit)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnstructuredFullRead {
    pub content: String,
    pub content_type: String,
    pub facts: UnstructuredFullReadFacts,
}

impl UnstructuredFullRead {
    pub fn new(content: impl Into<String>, content_type: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            content_type: content_type.into(),
            facts: UnstructuredFullReadFacts::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UnstructuredFullReadFacts {
    pub cost: Option<Cost>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AdapterError {
    diagnostic: Box<DiagnosticRecordDraft>,
}

impl AdapterError {
    pub fn new(diagnostic: DiagnosticRecordDraft) -> Self {
        let diagnostic = normalize_protocol_diagnostic(
            diagnostic,
            ProtocolDiagnosticFallback::new(
                DiagnosticSource::with_stage("docnav-adapter-contracts", "adapter-error"),
                "adapter-error-diagnostic-invalid",
                "adapter-error-diagnostic-not-protocol",
            ),
        );
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
        details.config_issues = issue
            .config_source
            .clone()
            .map(|config_source| vec![config_source]);
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
    let Ok(record) = diagnostic.into_record() else {
        return diagnostic_projection_failed();
    };
    ProtocolError::from_diagnostic_record(&record).unwrap_or_else(diagnostic_projection_failed)
}

fn diagnostic_projection_failed() -> ProtocolError {
    ProtocolError::internal_error("adapter-diagnostic-projection-failed")
}

#[cfg(test)]
mod tests;
