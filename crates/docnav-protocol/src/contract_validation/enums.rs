use docnav_typed_fields::FieldStringEnum;

use crate::{Operation, ProbeReasonCode, PROTOCOL_VERSION};

#[derive(Clone, Copy)]
pub(super) enum ContractVersion {
    V01,
}

impl FieldStringEnum for ContractVersion {
    fn variants() -> &'static [Self] {
        const VARIANTS: &[ContractVersion] = &[ContractVersion::V01];
        VARIANTS
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::V01 => PROTOCOL_VERSION,
        }
    }
}

impl FieldStringEnum for Operation {
    fn variants() -> &'static [Self] {
        const VARIANTS: &[Operation] = &[
            Operation::Outline,
            Operation::Read,
            Operation::Find,
            Operation::Info,
        ];
        VARIANTS
    }

    fn as_str(&self) -> &'static str {
        Operation::as_str(*self)
    }
}

impl FieldStringEnum for ProbeReasonCode {
    fn variants() -> &'static [Self] {
        const VARIANTS: &[ProbeReasonCode] = &[
            ProbeReasonCode::ExtensionMatch,
            ProbeReasonCode::ContentMatch,
            ProbeReasonCode::ContentConflict,
            ProbeReasonCode::ReadError,
        ];
        VARIANTS
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::ExtensionMatch => "EXTENSION_MATCH",
            Self::ContentMatch => "CONTENT_MATCH",
            Self::ContentConflict => "CONTENT_CONFLICT",
            Self::ReadError => "READ_ERROR",
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum ProtocolErrorCode {
    InvalidRequest,
    DocumentNotFound,
    DocumentPathInvalid,
    DocumentEncodingUnsupported,
    FormatUnknown,
    FormatAmbiguous,
    RefNotFound,
    RefAmbiguous,
    RefInvalid,
    AdapterUnavailable,
    InternalError,
}

impl FieldStringEnum for ProtocolErrorCode {
    fn variants() -> &'static [Self] {
        const VARIANTS: &[ProtocolErrorCode] = &[
            ProtocolErrorCode::InvalidRequest,
            ProtocolErrorCode::DocumentNotFound,
            ProtocolErrorCode::DocumentPathInvalid,
            ProtocolErrorCode::DocumentEncodingUnsupported,
            ProtocolErrorCode::FormatUnknown,
            ProtocolErrorCode::FormatAmbiguous,
            ProtocolErrorCode::RefNotFound,
            ProtocolErrorCode::RefAmbiguous,
            ProtocolErrorCode::RefInvalid,
            ProtocolErrorCode::AdapterUnavailable,
            ProtocolErrorCode::InternalError,
        ];
        VARIANTS
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidRequest => "INVALID_REQUEST",
            Self::DocumentNotFound => "DOCUMENT_NOT_FOUND",
            Self::DocumentPathInvalid => "DOCUMENT_PATH_INVALID",
            Self::DocumentEncodingUnsupported => "DOCUMENT_ENCODING_UNSUPPORTED",
            Self::FormatUnknown => "FORMAT_UNKNOWN",
            Self::FormatAmbiguous => "FORMAT_AMBIGUOUS",
            Self::RefNotFound => "REF_NOT_FOUND",
            Self::RefAmbiguous => "REF_AMBIGUOUS",
            Self::RefInvalid => "REF_INVALID",
            Self::AdapterUnavailable => "ADAPTER_UNAVAILABLE",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }
}
