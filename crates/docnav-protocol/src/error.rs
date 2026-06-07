use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

use crate::constants::{error_detail_fields as details_fields, stable_error_messages};
use crate::generated::error_rules;
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
    AdapterUnavailable,
    AdapterInvokeFailed,
    InternalError,
}

impl StableErrorCode {
    pub const fn required_details(self) -> &'static [&'static str] {
        error_rules::required_details(self)
    }
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
