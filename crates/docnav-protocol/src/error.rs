use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

use docnav_diagnostics::{
    DiagnosticCategory, DiagnosticCode, DiagnosticDetails, DiagnosticRecord, DiagnosticRecordDraft,
    DiagnosticSource, ProtocolDiagnosticCode,
};

use crate::constants::{error_detail_fields as details_fields, stable_error_messages};
use crate::{ErrorDetails, Operation};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StableError {
    pub code: StableErrorCode,
    pub message: String,
    pub details: ErrorDetails,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guidance: Option<Vec<String>>,
}

impl StableError {
    pub fn new(code: StableErrorCode, message: impl Into<String>, details: ErrorDetails) -> Self {
        Self {
            code,
            message: message.into(),
            details,
            guidance: None,
        }
    }

    pub fn with_guidance(mut self, guidance: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.guidance = Some(guidance.into_iter().map(Into::into).collect());
        self
    }

    pub fn invalid_request(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::InvalidRequest,
            stable_error_messages::INVALID_PROTOCOL_REQUEST,
            details([
                (details_fields::FIELD, field.into()),
                (details_fields::REASON, reason.into()),
            ]),
        )
    }

    pub fn document_not_found(path: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::DocumentNotFound,
            stable_error_messages::DOCUMENT_NOT_FOUND,
            details([(details_fields::PATH, path.into())]),
        )
    }

    pub fn document_path_invalid(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::DocumentPathInvalid,
            stable_error_messages::DOCUMENT_PATH_INVALID,
            details([
                (details_fields::PATH, path.into()),
                (details_fields::REASON, reason.into()),
            ]),
        )
    }

    pub fn document_encoding_unsupported(
        path: impl Into<String>,
        encoding: impl Into<String>,
    ) -> Self {
        Self::new(
            StableErrorCode::DocumentEncodingUnsupported,
            stable_error_messages::DOCUMENT_ENCODING_UNSUPPORTED,
            details([
                (details_fields::PATH, path.into()),
                (details_fields::ENCODING, encoding.into()),
            ]),
        )
    }

    pub fn format_unknown(
        path: impl Into<String>,
        reason: impl Into<String>,
        candidates: Value,
    ) -> Self {
        let mut details = details([
            (details_fields::PATH, path.into()),
            (details_fields::REASON, reason.into()),
        ]);
        details.insert(details_fields::CANDIDATES.to_owned(), candidates);
        Self::new(
            StableErrorCode::FormatUnknown,
            stable_error_messages::DOCUMENT_FORMAT_UNKNOWN,
            details,
        )
    }

    pub fn format_ambiguous(path: impl Into<String>, candidates: Value) -> Self {
        let mut details = details([(details_fields::PATH, path.into())]);
        details.insert(details_fields::CANDIDATES.to_owned(), candidates);
        Self::new(
            StableErrorCode::FormatAmbiguous,
            stable_error_messages::DOCUMENT_FORMAT_AMBIGUOUS,
            details,
        )
    }

    pub fn capability_unsupported(capability: Operation, adapter_id: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::CapabilityUnsupported,
            stable_error_messages::CAPABILITY_UNSUPPORTED,
            details([
                (details_fields::CAPABILITY, capability.to_string()),
                (details_fields::ADAPTER_ID, adapter_id.into()),
            ]),
        )
    }

    pub fn ref_not_found(ref_id: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::RefNotFound,
            stable_error_messages::REF_NOT_FOUND,
            details([(details_fields::REF, ref_id.into())]),
        )
    }

    pub fn ref_ambiguous(ref_id: impl Into<String>, candidate_count: u32) -> Self {
        let mut details = details([(details_fields::REF, ref_id.into())]);
        details.insert(
            details_fields::CANDIDATE_COUNT.to_owned(),
            Value::from(candidate_count),
        );
        Self::new(
            StableErrorCode::RefAmbiguous,
            stable_error_messages::REF_AMBIGUOUS,
            details,
        )
    }

    pub fn ref_invalid(ref_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::RefInvalid,
            stable_error_messages::REF_INVALID,
            details([
                (details_fields::REF, ref_id.into()),
                (details_fields::REASON, reason.into()),
            ]),
        )
    }

    pub fn adapter_unavailable(adapter_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::AdapterUnavailable,
            stable_error_messages::ADAPTER_UNAVAILABLE,
            details([
                (details_fields::ADAPTER_ID, adapter_id.into()),
                (details_fields::REASON, reason.into()),
            ]),
        )
    }

    pub fn adapter_invoke_failed(adapter_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::AdapterInvokeFailed,
            stable_error_messages::ADAPTER_INVOKE_FAILED,
            details([
                (details_fields::ADAPTER_ID, adapter_id.into()),
                (details_fields::REASON, reason.into()),
            ]),
        )
    }

    pub fn internal_error(error_id: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::InternalError,
            stable_error_messages::INTERNAL_ERROR,
            details([(details_fields::ERROR_ID, error_id.into())]),
        )
    }

    pub fn validate_required_details(&self) -> Result<(), MissingErrorDetail> {
        for &field in self.code.required_details() {
            if !self.details.contains_key(field) {
                return Err(MissingErrorDetail {
                    code: self.code,
                    field,
                });
            }
        }
        Ok(())
    }

    pub fn to_record_draft(&self, source: DiagnosticSource) -> DiagnosticRecordDraft {
        let details = self
            .details
            .clone()
            .into_iter()
            .collect::<serde_json::Map<String, Value>>();
        let draft = DiagnosticRecordDraft::new(
            self.code.diagnostic_code(),
            self.message.clone(),
            DiagnosticDetails::Object(details),
            source,
        );
        match &self.guidance {
            Some(guidance) => draft.with_guidance(guidance.clone()),
            None => draft,
        }
    }

    pub fn from_diagnostic_record(record: &DiagnosticRecord) -> Option<Self> {
        let DiagnosticCode::Protocol(code) = record.code else {
            return None;
        };
        let Value::Object(details) = record.details.to_value() else {
            return None;
        };
        Some(Self {
            code: code.into(),
            message: record.summary.clone(),
            details: details.into_iter().collect(),
            guidance: record.guidance.clone(),
        })
    }
}

fn details(fields: impl IntoIterator<Item = (&'static str, String)>) -> ErrorDetails {
    fields
        .into_iter()
        .map(|(key, value)| (key.to_owned(), Value::String(value)))
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StableErrorCode {
    InvalidRequest,
    DocumentNotFound,
    DocumentPathInvalid,
    DocumentEncodingUnsupported,
    FormatUnknown,
    FormatAmbiguous,
    CapabilityUnsupported,
    RefNotFound,
    RefAmbiguous,
    RefInvalid,
    AdapterUnavailable,
    AdapterInvokeFailed,
    InternalError,
}

impl StableErrorCode {
    pub const fn diagnostic_code(self) -> ProtocolDiagnosticCode {
        match self {
            Self::InvalidRequest => ProtocolDiagnosticCode::InvalidRequest,
            Self::DocumentNotFound => ProtocolDiagnosticCode::DocumentNotFound,
            Self::DocumentPathInvalid => ProtocolDiagnosticCode::DocumentPathInvalid,
            Self::DocumentEncodingUnsupported => {
                ProtocolDiagnosticCode::DocumentEncodingUnsupported
            }
            Self::FormatUnknown => ProtocolDiagnosticCode::FormatUnknown,
            Self::FormatAmbiguous => ProtocolDiagnosticCode::FormatAmbiguous,
            Self::CapabilityUnsupported => ProtocolDiagnosticCode::CapabilityUnsupported,
            Self::RefNotFound => ProtocolDiagnosticCode::RefNotFound,
            Self::RefAmbiguous => ProtocolDiagnosticCode::RefAmbiguous,
            Self::RefInvalid => ProtocolDiagnosticCode::RefInvalid,
            Self::AdapterUnavailable => ProtocolDiagnosticCode::AdapterUnavailable,
            Self::AdapterInvokeFailed => ProtocolDiagnosticCode::AdapterInvokeFailed,
            Self::InternalError => ProtocolDiagnosticCode::InternalError,
        }
    }

    pub const fn diagnostic(self) -> DiagnosticCode {
        DiagnosticCode::Protocol(self.diagnostic_code())
    }

    pub const fn required_details(self) -> &'static [&'static str] {
        self.diagnostic_code().required_detail_names()
    }

    pub const fn category(self) -> StableErrorCategory {
        match self.diagnostic().category() {
            DiagnosticCategory::Request => StableErrorCategory::Request,
            DiagnosticCategory::Document => StableErrorCategory::Document,
            DiagnosticCategory::AdapterBoundary => StableErrorCategory::AdapterBoundary,
            DiagnosticCategory::Internal | DiagnosticCategory::Compatibility => {
                StableErrorCategory::Internal
            }
        }
    }
}

impl From<ProtocolDiagnosticCode> for StableErrorCode {
    fn from(code: ProtocolDiagnosticCode) -> Self {
        match code {
            ProtocolDiagnosticCode::InvalidRequest => Self::InvalidRequest,
            ProtocolDiagnosticCode::DocumentNotFound => Self::DocumentNotFound,
            ProtocolDiagnosticCode::DocumentPathInvalid => Self::DocumentPathInvalid,
            ProtocolDiagnosticCode::DocumentEncodingUnsupported => {
                Self::DocumentEncodingUnsupported
            }
            ProtocolDiagnosticCode::FormatUnknown => Self::FormatUnknown,
            ProtocolDiagnosticCode::FormatAmbiguous => Self::FormatAmbiguous,
            ProtocolDiagnosticCode::CapabilityUnsupported => Self::CapabilityUnsupported,
            ProtocolDiagnosticCode::RefNotFound => Self::RefNotFound,
            ProtocolDiagnosticCode::RefAmbiguous => Self::RefAmbiguous,
            ProtocolDiagnosticCode::RefInvalid => Self::RefInvalid,
            ProtocolDiagnosticCode::AdapterUnavailable => Self::AdapterUnavailable,
            ProtocolDiagnosticCode::AdapterInvokeFailed => Self::AdapterInvokeFailed,
            ProtocolDiagnosticCode::InternalError => Self::InternalError,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StableErrorCategory {
    Request,
    Document,
    AdapterBoundary,
    Internal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissingErrorDetail {
    pub code: StableErrorCode,
    pub field: &'static str,
}

impl fmt::Display for MissingErrorDetail {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "error {:?} is missing required details.{}",
            self.code, self.field
        )
    }
}

impl std::error::Error for MissingErrorDetail {}
