use docnav_diagnostics::{
    typed_codes, DiagnosticCategory, DiagnosticCode, DiagnosticRecordDraft, DiagnosticSource,
    FieldReasonDetails, InternalDetails, PathDetails, PathReasonDetails,
};
use docnav_protocol::{
    normalize_protocol_diagnostic, protocol_error_record_draft, ProtocolDiagnosticFallback,
};

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
    exit_code: DocnavExitCode,
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn new(diagnostic: DiagnosticRecordDraft) -> Self {
        let diagnostic = normalize_protocol_diagnostic(
            diagnostic,
            ProtocolDiagnosticFallback::new(
                DiagnosticSource::with_stage("docnav", "app-error"),
                "app-error-diagnostic-invalid",
                "app-error-diagnostic-not-protocol",
            ),
        );
        let exit_code = exit_code_for_diagnostic(diagnostic.code());
        Self {
            diagnostic: Box::new(diagnostic),
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

    pub fn invalid_request_with_input_context(
        field: impl Into<String>,
        reason: impl Into<String>,
        received: Option<impl Into<String>>,
        accepted: impl IntoIterator<Item = impl Into<String>>,
        guidance: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let accepted = accepted.into_iter().map(Into::into).collect::<Vec<_>>();
        let guidance = guidance.into_iter().map(Into::into).collect::<Vec<_>>();
        let mut details = FieldReasonDetails::new(field, reason);
        details.received = received.map(Into::into);
        if !accepted.is_empty() {
            details.accepted = Some(accepted);
        }

        let mut draft = protocol_error_record_draft::<typed_codes::protocol::InvalidRequest>(
            details,
            DiagnosticSource::with_stage("docnav", "core"),
        );
        if !guidance.is_empty() {
            draft = draft.with_guidance(guidance);
        }
        Self::new(draft)
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
}

pub fn exit_code_for_diagnostic(code: impl Into<DiagnosticCode>) -> DocnavExitCode {
    match code.into().category() {
        DiagnosticCategory::Request => DocnavExitCode::InputError,
        DiagnosticCategory::Document => DocnavExitCode::DocumentError,
        DiagnosticCategory::AdapterBoundary => DocnavExitCode::AdapterOrProtocolError,
        DiagnosticCategory::Internal => DocnavExitCode::InternalError,
    }
}
