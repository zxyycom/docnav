use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::fmt;

use docnav_diagnostics::{
    typed_codes, AdapterReasonDetails, CapabilityAdapterDetails, DiagnosticCategory,
    DiagnosticCode, DiagnosticDetailsPayload, DiagnosticRecord, DiagnosticRecordDraft,
    DiagnosticSource, FieldReasonDetails, FormatAmbiguousDetails, FormatUnknownDetails,
    InternalDetails, PathDetails, PathEncodingDetails, PathReasonDetails, ProtocolDiagnosticCode,
    ProtocolDiagnosticMarker, RefCandidateCountDetails, RefDetails, RefReasonDetails,
};

use crate::constants::protocol_error_messages;
use crate::{ErrorDetails, Operation};

pub fn protocol_error_record_draft<C>(
    details: C::Details,
    source: DiagnosticSource,
) -> DiagnosticRecordDraft
where
    C: ProtocolDiagnosticMarker,
{
    protocol_error_record_draft_with_summary::<C>(
        protocol_error_default_message(C::PROTOCOL_CODE),
        details,
        source,
    )
}

pub fn protocol_error_record_draft_with_summary<C>(
    summary: impl Into<String>,
    details: C::Details,
    source: DiagnosticSource,
) -> DiagnosticRecordDraft
where
    C: ProtocolDiagnosticMarker,
{
    DiagnosticRecordDraft::new::<C>(summary, details, source)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolError {
    #[serde(with = "protocol_diagnostic_code_serde")]
    code: ProtocolDiagnosticCode,
    message: String,
    details: ErrorDetails,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    guidance: Option<Vec<String>>,
}

impl ProtocolError {
    pub fn new<C>(message: impl Into<String>, details: C::Details) -> Self
    where
        C: ProtocolDiagnosticMarker,
    {
        Self {
            code: C::PROTOCOL_CODE,
            message: message.into(),
            details: error_details_from_payload(&details),
            guidance: None,
        }
    }

    pub fn with_guidance(mut self, guidance: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.guidance = Some(guidance.into_iter().map(Into::into).collect());
        self
    }

    pub const fn code(&self) -> ProtocolDiagnosticCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub const fn details(&self) -> &ErrorDetails {
        &self.details
    }

    pub fn guidance(&self) -> Option<&[String]> {
        self.guidance.as_deref()
    }

    fn with_default_message<C>(details: C::Details) -> Self
    where
        C: ProtocolDiagnosticMarker,
    {
        Self::new::<C>(protocol_error_default_message(C::PROTOCOL_CODE), details)
    }

    pub fn invalid_request(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::with_default_message::<typed_codes::protocol::InvalidRequest>(
            FieldReasonDetails::new(field, reason),
        )
    }

    pub fn document_not_found(path: impl Into<String>) -> Self {
        Self::with_default_message::<typed_codes::protocol::DocumentNotFound>(PathDetails::new(
            path,
        ))
    }

    pub fn document_path_invalid(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::with_default_message::<typed_codes::protocol::DocumentPathInvalid>(
            PathReasonDetails::new(path, reason),
        )
    }

    pub fn document_encoding_unsupported(
        path: impl Into<String>,
        encoding: impl Into<String>,
    ) -> Self {
        Self::with_default_message::<typed_codes::protocol::DocumentEncodingUnsupported>(
            PathEncodingDetails::new(path, encoding),
        )
    }

    pub fn format_unknown(
        path: impl Into<String>,
        reason: impl Into<String>,
        candidates: Value,
    ) -> Self {
        Self::with_default_message::<typed_codes::protocol::FormatUnknown>(
            FormatUnknownDetails::new(path, reason, candidates),
        )
    }

    pub fn format_ambiguous(path: impl Into<String>, candidates: Value) -> Self {
        Self::with_default_message::<typed_codes::protocol::FormatAmbiguous>(
            FormatAmbiguousDetails::new(path, candidates),
        )
    }

    pub fn capability_unsupported(capability: Operation, adapter_id: impl Into<String>) -> Self {
        Self::with_default_message::<typed_codes::protocol::CapabilityUnsupported>(
            CapabilityAdapterDetails::new(capability.to_string(), adapter_id),
        )
    }

    pub fn ref_not_found(ref_id: impl Into<String>) -> Self {
        Self::with_default_message::<typed_codes::protocol::RefNotFound>(RefDetails::new(ref_id))
    }

    pub fn ref_ambiguous(ref_id: impl Into<String>, candidate_count: u32) -> Self {
        Self::with_default_message::<typed_codes::protocol::RefAmbiguous>(
            RefCandidateCountDetails::new(ref_id, candidate_count),
        )
    }

    pub fn ref_invalid(ref_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::with_default_message::<typed_codes::protocol::RefInvalid>(RefReasonDetails::new(
            ref_id, reason,
        ))
    }

    pub fn adapter_unavailable(adapter_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::with_default_message::<typed_codes::protocol::AdapterUnavailable>(
            AdapterReasonDetails::new(adapter_id, reason),
        )
    }

    pub fn adapter_invoke_failed(adapter_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::with_default_message::<typed_codes::protocol::AdapterInvokeFailed>(
            AdapterReasonDetails::new(adapter_id, reason),
        )
    }

    pub fn internal_error(error_id: impl Into<String>) -> Self {
        Self::with_default_message::<typed_codes::protocol::InternalError>(InternalDetails::new(
            error_id,
        ))
    }

    pub fn validate_details(&self) -> Result<(), InvalidErrorDetail> {
        self.code
            .details_rule()
            .validate_value(&Value::Object(self.details.clone().into_iter().collect()))
            .map_err(|source| InvalidErrorDetail {
                code: self.code,
                reason: source.to_string(),
            })
    }

    pub fn to_record_draft(
        &self,
        source: DiagnosticSource,
    ) -> Result<DiagnosticRecordDraft, InvalidErrorDetail> {
        match self.code {
            ProtocolDiagnosticCode::InvalidRequest => {
                self.record_draft::<typed_codes::protocol::InvalidRequest>(source)
            }
            ProtocolDiagnosticCode::DocumentNotFound => {
                self.record_draft::<typed_codes::protocol::DocumentNotFound>(source)
            }
            ProtocolDiagnosticCode::DocumentPathInvalid => {
                self.record_draft::<typed_codes::protocol::DocumentPathInvalid>(source)
            }
            ProtocolDiagnosticCode::DocumentEncodingUnsupported => {
                self.record_draft::<typed_codes::protocol::DocumentEncodingUnsupported>(source)
            }
            ProtocolDiagnosticCode::FormatUnknown => {
                self.record_draft::<typed_codes::protocol::FormatUnknown>(source)
            }
            ProtocolDiagnosticCode::FormatAmbiguous => {
                self.record_draft::<typed_codes::protocol::FormatAmbiguous>(source)
            }
            ProtocolDiagnosticCode::CapabilityUnsupported => {
                self.record_draft::<typed_codes::protocol::CapabilityUnsupported>(source)
            }
            ProtocolDiagnosticCode::RefNotFound => {
                self.record_draft::<typed_codes::protocol::RefNotFound>(source)
            }
            ProtocolDiagnosticCode::RefAmbiguous => {
                self.record_draft::<typed_codes::protocol::RefAmbiguous>(source)
            }
            ProtocolDiagnosticCode::RefInvalid => {
                self.record_draft::<typed_codes::protocol::RefInvalid>(source)
            }
            ProtocolDiagnosticCode::AdapterUnavailable => {
                self.record_draft::<typed_codes::protocol::AdapterUnavailable>(source)
            }
            ProtocolDiagnosticCode::AdapterInvokeFailed => {
                self.record_draft::<typed_codes::protocol::AdapterInvokeFailed>(source)
            }
            ProtocolDiagnosticCode::InternalError => {
                self.record_draft::<typed_codes::protocol::InternalError>(source)
            }
        }
    }

    fn record_draft<C>(
        &self,
        source: DiagnosticSource,
    ) -> Result<DiagnosticRecordDraft, InvalidErrorDetail>
    where
        C: ProtocolDiagnosticMarker,
    {
        let details = payload_from_error_details::<C::Details>(self.code, &self.details)?;
        let draft = DiagnosticRecordDraft::new::<C>(self.message.clone(), details, source);
        Ok(match &self.guidance {
            Some(guidance) => draft.with_guidance(guidance.clone()),
            None => draft,
        })
    }

    pub fn from_diagnostic_record(record: &DiagnosticRecord) -> Option<Self> {
        let DiagnosticCode::Protocol(code) = record.code() else {
            return None;
        };
        let Value::Object(details) = record.details().to_value() else {
            return None;
        };
        Some(Self {
            code,
            message: record.summary().to_owned(),
            details: details.into_iter().collect(),
            guidance: record.guidance().map(|guidance| guidance.to_vec()),
        })
    }
}

fn error_details_from_payload<T>(details: &T) -> ErrorDetails
where
    T: DiagnosticDetailsPayload,
{
    let Value::Object(object) =
        serde_json::to_value(details).expect("diagnostic details payloads serialize to objects")
    else {
        unreachable!("diagnostic details payloads serialize to objects");
    };
    object.into_iter().collect()
}

fn payload_from_error_details<T>(
    code: ProtocolDiagnosticCode,
    details: &ErrorDetails,
) -> Result<T, InvalidErrorDetail>
where
    T: DiagnosticDetailsPayload,
{
    serde_json::from_value(Value::Object(details.clone().into_iter().collect())).map_err(|error| {
        InvalidErrorDetail {
            code,
            reason: error.to_string(),
        }
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolErrorCategory {
    Request,
    Document,
    AdapterBoundary,
    Internal,
}

pub const fn protocol_error_default_message(code: ProtocolDiagnosticCode) -> &'static str {
    match code {
        ProtocolDiagnosticCode::InvalidRequest => protocol_error_messages::INVALID_PROTOCOL_REQUEST,
        ProtocolDiagnosticCode::DocumentNotFound => protocol_error_messages::DOCUMENT_NOT_FOUND,
        ProtocolDiagnosticCode::DocumentPathInvalid => {
            protocol_error_messages::DOCUMENT_PATH_INVALID
        }
        ProtocolDiagnosticCode::DocumentEncodingUnsupported => {
            protocol_error_messages::DOCUMENT_ENCODING_UNSUPPORTED
        }
        ProtocolDiagnosticCode::FormatUnknown => protocol_error_messages::DOCUMENT_FORMAT_UNKNOWN,
        ProtocolDiagnosticCode::FormatAmbiguous => {
            protocol_error_messages::DOCUMENT_FORMAT_AMBIGUOUS
        }
        ProtocolDiagnosticCode::CapabilityUnsupported => {
            protocol_error_messages::CAPABILITY_UNSUPPORTED
        }
        ProtocolDiagnosticCode::RefNotFound => protocol_error_messages::REF_NOT_FOUND,
        ProtocolDiagnosticCode::RefAmbiguous => protocol_error_messages::REF_AMBIGUOUS,
        ProtocolDiagnosticCode::RefInvalid => protocol_error_messages::REF_INVALID,
        ProtocolDiagnosticCode::AdapterUnavailable => protocol_error_messages::ADAPTER_UNAVAILABLE,
        ProtocolDiagnosticCode::AdapterInvokeFailed => {
            protocol_error_messages::ADAPTER_INVOKE_FAILED
        }
        ProtocolDiagnosticCode::InternalError => protocol_error_messages::INTERNAL_ERROR,
    }
}

pub const fn protocol_error_category(code: ProtocolDiagnosticCode) -> ProtocolErrorCategory {
    match DiagnosticCode::Protocol(code).category() {
        DiagnosticCategory::Request => ProtocolErrorCategory::Request,
        DiagnosticCategory::Document => ProtocolErrorCategory::Document,
        DiagnosticCategory::AdapterBoundary => ProtocolErrorCategory::AdapterBoundary,
        DiagnosticCategory::Internal | DiagnosticCategory::Compatibility => {
            ProtocolErrorCategory::Internal
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidErrorDetail {
    pub code: ProtocolDiagnosticCode,
    pub reason: String,
}

impl fmt::Display for InvalidErrorDetail {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "error {:?} has invalid details: {}",
            self.code, self.reason
        )
    }
}

impl std::error::Error for InvalidErrorDetail {}

mod protocol_diagnostic_code_serde {
    use super::*;

    pub fn serialize<S>(code: &ProtocolDiagnosticCode, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(code.protocol_code())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ProtocolDiagnosticCode, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        ProtocolDiagnosticCode::from_protocol_code(&value).ok_or_else(|| {
            serde::de::Error::custom(format!("unknown protocol error code {value:?}"))
        })
    }
}
