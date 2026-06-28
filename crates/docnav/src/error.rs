use docnav_diagnostics::{
    typed_codes, AdapterReasonDetails, DiagnosticCategory, DiagnosticCode, DiagnosticRecordDraft,
    DiagnosticSource, FieldReasonDetails, FormatUnknownDetails, InternalDetails, PathDetails,
    PathReasonDetails,
};
use docnav_protocol::{protocol_error_record_draft, protocol_error_record_draft_with_summary};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocnavExitCode {
    Success = 0,
    InternalError = 1,
    InputError = 2,
    DocumentError = 3,
    AdapterOrProtocolError = 4,
}

impl DocnavExitCode {
    pub const fn code(self) -> i32 {
        self as i32
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppError {
    diagnostic: Box<DiagnosticRecordDraft>,
    related_diagnostics: Vec<DiagnosticRecordDraft>,
    exit_code: DocnavExitCode,
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn new(diagnostic: DiagnosticRecordDraft) -> Self {
        let diagnostic = normalize_protocol_diagnostic(diagnostic);
        let exit_code = exit_code_for_diagnostic(diagnostic.code());
        Self {
            diagnostic: Box::new(diagnostic),
            related_diagnostics: Vec::new(),
            exit_code,
        }
    }

    pub fn invalid_request(field: impl Into<String>, reason: impl Into<String>) -> Self {
        let reason = reason.into();
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::InvalidRequest,
        >(
            FieldReasonDetails::new(field, reason),
            DiagnosticSource::with_stage("docnav", "core"),
        ))
    }

    pub fn internal(error_id: impl Into<String>) -> Self {
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::InternalError,
        >(
            InternalDetails::new(error_id),
            DiagnosticSource::with_stage("docnav", "core"),
        ))
    }

    pub fn diagnostic(&self) -> &DiagnosticRecordDraft {
        self.diagnostic.as_ref()
    }

    pub fn related_diagnostics(&self) -> &[DiagnosticRecordDraft] {
        &self.related_diagnostics
    }

    pub fn with_related_diagnostics(
        mut self,
        diagnostics: impl IntoIterator<Item = DiagnosticRecordDraft>,
    ) -> Self {
        self.related_diagnostics.extend(diagnostics);
        self
    }

    pub const fn exit_code(&self) -> DocnavExitCode {
        self.exit_code
    }

    pub fn document_not_found(path: impl Into<String>) -> Self {
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::DocumentNotFound,
        >(
            PathDetails::new(path),
            DiagnosticSource::with_stage("docnav", "runtime"),
        ))
    }

    pub fn document_path_invalid(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::DocumentPathInvalid,
        >(
            PathReasonDetails::new(path, reason),
            DiagnosticSource::with_stage("docnav", "runtime"),
        ))
    }

    pub fn format_unknown(
        path: impl Into<String>,
        reason: impl Into<String>,
        candidates: Value,
    ) -> Self {
        let reason = reason.into();
        Self::new(protocol_error_record_draft_with_summary::<
            typed_codes::protocol::FormatUnknown,
        >(
            reason.clone(),
            FormatUnknownDetails::new(path, reason, candidates),
            DiagnosticSource::with_stage("docnav", "routing"),
        ))
    }

    pub fn adapter_invoke_failed(
        adapter_id: impl Into<String>,
        reason: impl Into<String>,
        exit_code: Option<i32>,
        stderr: impl Into<String>,
    ) -> Self {
        let reason = reason.into();
        let stderr = stderr.into();
        Self::new(protocol_error_record_draft_with_summary::<
            typed_codes::protocol::AdapterInvokeFailed,
        >(
            reason.clone(),
            AdapterReasonDetails {
                adapter_id: adapter_id.into(),
                reason,
                exit_code,
                stderr: (!stderr.trim().is_empty()).then_some(stderr),
            },
            DiagnosticSource::with_stage("docnav", "adapter-output"),
        ))
    }

    pub fn invalid_request_with_summary(
        field: impl Into<String>,
        reason: impl Into<String>,
        summary: impl Into<String>,
        source: DiagnosticSource,
    ) -> Self {
        let reason = reason.into();
        Self::new(protocol_error_record_draft_with_summary::<
            typed_codes::protocol::InvalidRequest,
        >(
            summary, FieldReasonDetails::new(field, reason), source
        ))
    }
}

pub fn exit_code_for_diagnostic(code: impl Into<DiagnosticCode>) -> DocnavExitCode {
    match code.into().category() {
        DiagnosticCategory::Request => DocnavExitCode::InputError,
        DiagnosticCategory::Document => DocnavExitCode::DocumentError,
        DiagnosticCategory::AdapterBoundary => DocnavExitCode::AdapterOrProtocolError,
        DiagnosticCategory::Internal | DiagnosticCategory::Compatibility => {
            DocnavExitCode::InternalError
        }
    }
}

fn normalize_protocol_diagnostic(diagnostic: DiagnosticRecordDraft) -> DiagnosticRecordDraft {
    if is_valid_protocol_diagnostic(&diagnostic) {
        return diagnostic;
    }

    let error_id = if matches!(diagnostic.code(), DiagnosticCode::Protocol(_)) {
        "app-error-diagnostic-invalid"
    } else {
        "app-error-diagnostic-not-protocol"
    };
    protocol_error_record_draft::<typed_codes::protocol::InternalError>(
        InternalDetails::new(error_id),
        DiagnosticSource::with_stage("docnav", "app-error"),
    )
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
