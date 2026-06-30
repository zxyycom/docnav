use docnav_diagnostics::{
    typed_codes, CapabilityAdapterDetails, DiagnosticCategory, DiagnosticCode,
    DiagnosticRecordDraft, DiagnosticSource, DiagnosticStack, FieldReasonDetails, InternalDetails,
    PathDetails, PathEncodingDetails, PathReasonDetails, RefDetails, RefReasonDetails,
};
use docnav_protocol::{protocol_error_record_draft, Operation, ProtocolError};
use std::fmt;

use crate::constants::diagnostics;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterExitCode {
    Success = 0,
    InternalError = 1,
    ProtocolError = 2,
    HandlerError = 3,
    IoError = 4,
}

impl AdapterExitCode {
    pub const fn code(self) -> i32 {
        self as i32
    }
}

pub fn exit_code_for_diagnostic(code: impl Into<DiagnosticCode>) -> AdapterExitCode {
    match code.into().category() {
        DiagnosticCategory::Request => AdapterExitCode::ProtocolError,
        DiagnosticCategory::Document => AdapterExitCode::HandlerError,
        DiagnosticCategory::AdapterBoundary => AdapterExitCode::IoError,
        DiagnosticCategory::Internal => AdapterExitCode::InternalError,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AdapterError {
    diagnostic: Box<DiagnosticRecordDraft>,
    exit_code: AdapterExitCode,
}

impl AdapterError {
    pub fn new(diagnostic: DiagnosticRecordDraft) -> Self {
        let diagnostic = normalize_protocol_diagnostic(diagnostic);
        let exit_code = exit_code_for_diagnostic(diagnostic.code());
        Self {
            diagnostic: Box::new(diagnostic),
            exit_code,
        }
    }

    pub fn with_exit_code(
        diagnostic: DiagnosticRecordDraft,
        exit_code: AdapterExitCode,
    ) -> Result<Self, AdapterExitCodeError> {
        if exit_code == AdapterExitCode::Success {
            Err(AdapterExitCodeError { exit_code })
        } else {
            let diagnostic = normalize_protocol_diagnostic(diagnostic);
            Ok(Self {
                diagnostic: Box::new(diagnostic),
                exit_code,
            })
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

    pub const fn exit_code(&self) -> AdapterExitCode {
        self.exit_code
    }

    pub fn invalid_request(field: impl Into<String>, reason: impl Into<String>) -> Self {
        let reason = reason.into();
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::InvalidRequest,
        >(
            FieldReasonDetails::new(field, reason),
            DiagnosticSource::with_stage("docnav-adapter-sdk", "adapter"),
        ))
    }

    pub fn document_not_found(path: impl Into<String>) -> Self {
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::DocumentNotFound,
        >(
            PathDetails::new(path),
            DiagnosticSource::with_stage("docnav-adapter-sdk", "adapter"),
        ))
    }

    pub fn document_path_invalid(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::DocumentPathInvalid,
        >(
            PathReasonDetails::new(path, reason),
            DiagnosticSource::with_stage("docnav-adapter-sdk", "adapter"),
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
            DiagnosticSource::with_stage("docnav-adapter-sdk", "adapter"),
        ))
    }

    pub fn ref_not_found(ref_id: impl Into<String>) -> Self {
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::RefNotFound,
        >(
            RefDetails::new(ref_id),
            DiagnosticSource::with_stage("docnav-adapter-sdk", "adapter"),
        ))
    }

    pub fn ref_invalid(ref_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::RefInvalid,
        >(
            RefReasonDetails::new(ref_id, reason),
            DiagnosticSource::with_stage("docnav-adapter-sdk", "adapter"),
        ))
    }

    pub fn capability_unsupported(operation: Operation, adapter_id: impl Into<String>) -> Self {
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::CapabilityUnsupported,
        >(
            CapabilityAdapterDetails::new(operation.to_string(), adapter_id),
            DiagnosticSource::with_stage("docnav-adapter-sdk", "adapter"),
        ))
    }

    pub fn internal(error_id: impl Into<String>) -> Self {
        Self::new(protocol_error_record_draft::<
            typed_codes::protocol::InternalError,
        >(
            InternalDetails::new(error_id),
            DiagnosticSource::with_stage("docnav-adapter-sdk", "adapter"),
        ))
    }
}

impl From<DiagnosticRecordDraft> for AdapterError {
    fn from(diagnostic: DiagnosticRecordDraft) -> Self {
        Self::new(diagnostic)
    }
}

pub(crate) fn protocol_error_from_diagnostic(diagnostic: DiagnosticRecordDraft) -> ProtocolError {
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
        DiagnosticSource::with_stage("docnav-adapter-sdk", "adapter-error"),
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdapterExitCodeError {
    exit_code: AdapterExitCode,
}

impl AdapterExitCodeError {
    pub const fn exit_code(self) -> AdapterExitCode {
        self.exit_code
    }
}

impl fmt::Display for AdapterExitCodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} {:?}",
            diagnostics::ADAPTER_ERROR_EXIT_CODE_CANNOT_BE,
            self.exit_code
        )
    }
}

impl std::error::Error for AdapterExitCodeError {}
