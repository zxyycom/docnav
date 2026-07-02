use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

use docnav_diagnostics::{
    typed_codes, AdapterReasonDetails, CapabilityAdapterDetails, DiagnosticCode, DiagnosticRecord,
    DiagnosticRecordDraft, DiagnosticSource, FieldReasonDetails, FormatAmbiguousDetails,
    FormatCandidateDetails, FormatUnknownDetails, InternalDetails, PathDetails,
    PathEncodingDetails, PathReasonDetails, ProtocolDiagnosticCode, ProtocolDiagnosticMarker,
    RefCandidateCountDetails, RefDetails, RefReasonDetails,
};

use crate::{ErrorDetails, Operation};

mod details;
mod metadata;
mod protocol_diagnostic_code_serde;

use details::{
    error_details_from_payload, expected_from_details, location_from_details,
    payload_from_error_details, received_from_details,
};
use metadata::{default_owner_for_code, owner_from_source};

pub use metadata::{
    protocol_error_category, protocol_error_default_guidance, protocol_error_default_message,
};

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
    owner: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    location: Option<Box<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    received: Option<Box<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expected: Option<Box<Value>>,
    details: Box<ErrorDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    guidance: Option<Vec<String>>,
}

impl ProtocolError {
    pub fn new<C>(message: impl Into<String>, details: C::Details) -> Self
    where
        C: ProtocolDiagnosticMarker,
    {
        let details = error_details_from_payload(&details);
        let location = location_from_details(&details);
        let received = received_from_details(&details);
        let expected = expected_from_details(&details);
        Self {
            code: C::PROTOCOL_CODE,
            message: message.into(),
            owner: default_owner_for_code(C::PROTOCOL_CODE).to_owned(),
            location: location.map(Box::new),
            received: received.map(Box::new),
            expected: expected.map(Box::new),
            details: Box::new(details),
            guidance: Some(vec![
                protocol_error_default_guidance(C::PROTOCOL_CODE).to_owned()
            ]),
        }
    }

    pub fn with_guidance(mut self, guidance: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.guidance = Some(guidance.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = owner.into();
        self
    }

    pub const fn code(&self) -> ProtocolDiagnosticCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn location(&self) -> Option<&Value> {
        self.location.as_deref()
    }

    pub fn received(&self) -> Option<&Value> {
        self.received.as_deref()
    }

    pub fn expected(&self) -> Option<&Value> {
        self.expected.as_deref()
    }

    pub fn details(&self) -> &ErrorDetails {
        self.details.as_ref()
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
        candidates: Vec<FormatCandidateDetails>,
    ) -> Self {
        Self::with_default_message::<typed_codes::protocol::FormatUnknown>(
            FormatUnknownDetails::new(path, reason, candidates),
        )
    }

    pub fn format_ambiguous(
        path: impl Into<String>,
        candidates: Vec<FormatCandidateDetails>,
    ) -> Self {
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
            .validate_value(&Value::Object(
                self.details.as_ref().clone().into_iter().collect(),
            ))
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
        let details = payload_from_error_details::<C::Details>(self.code, self.details.as_ref())?;
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
        let details = details.into_iter().collect();
        Some(Self {
            code,
            message: record.summary().to_owned(),
            owner: owner_from_source(record.source()),
            location: location_from_details(&details).map(Box::new),
            received: received_from_details(&details).map(Box::new),
            expected: expected_from_details(&details).map(Box::new),
            details: Box::new(details),
            guidance: Some(
                record
                    .guidance()
                    .map(|guidance| guidance.to_vec())
                    .unwrap_or_else(|| vec![protocol_error_default_guidance(code).to_owned()]),
            ),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolErrorCategory {
    Request,
    Document,
    AdapterBoundary,
    Internal,
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
