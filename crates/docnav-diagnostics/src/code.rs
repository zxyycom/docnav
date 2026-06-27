use serde::Serialize;

use crate::details::{DetailFieldRule, DetailFieldType, DiagnosticDetailsRule};

const FIELD_REASON_FIELDS: &[DetailFieldRule] = &[
    DetailFieldRule::required("field", DetailFieldType::String),
    DetailFieldRule::required("reason", DetailFieldType::String),
    DetailFieldRule::optional("path", DetailFieldType::String),
    DetailFieldRule::optional("received", DetailFieldType::String),
    DetailFieldRule::optional("accepted", DetailFieldType::StringArray),
];
const FIELD_REASON_NAMES: &[&str] = &["field", "reason"];
const PATH_FIELDS: &[DetailFieldRule] =
    &[DetailFieldRule::required("path", DetailFieldType::String)];
const PATH_NAMES: &[&str] = &["path"];
const PATH_REASON_FIELDS: &[DetailFieldRule] = &[
    DetailFieldRule::required("path", DetailFieldType::String),
    DetailFieldRule::required("reason", DetailFieldType::String),
];
const PATH_REASON_NAMES: &[&str] = &["path", "reason"];
const PATH_ENCODING_FIELDS: &[DetailFieldRule] = &[
    DetailFieldRule::required("path", DetailFieldType::String),
    DetailFieldRule::required("encoding", DetailFieldType::String),
];
const PATH_ENCODING_NAMES: &[&str] = &["path", "encoding"];
const FORMAT_UNKNOWN_FIELDS: &[DetailFieldRule] = &[
    DetailFieldRule::required("path", DetailFieldType::String),
    DetailFieldRule::required("reason", DetailFieldType::String),
    DetailFieldRule::required("candidates", DetailFieldType::Any),
];
const FORMAT_UNKNOWN_NAMES: &[&str] = &["path", "reason", "candidates"];
const FORMAT_AMBIGUOUS_FIELDS: &[DetailFieldRule] = &[
    DetailFieldRule::required("path", DetailFieldType::String),
    DetailFieldRule::required("candidates", DetailFieldType::Any),
];
const FORMAT_AMBIGUOUS_NAMES: &[&str] = &["path", "candidates"];
const CAPABILITY_ADAPTER_FIELDS: &[DetailFieldRule] = &[
    DetailFieldRule::required("capability", DetailFieldType::String),
    DetailFieldRule::required("adapter_id", DetailFieldType::String),
];
const CAPABILITY_ADAPTER_NAMES: &[&str] = &["capability", "adapter_id"];
const REF_FIELDS: &[DetailFieldRule] = &[DetailFieldRule::required("ref", DetailFieldType::String)];
const REF_NAMES: &[&str] = &["ref"];
const REF_CANDIDATE_FIELDS: &[DetailFieldRule] = &[
    DetailFieldRule::required("ref", DetailFieldType::String),
    DetailFieldRule::required("candidate_count", DetailFieldType::U32),
];
const REF_CANDIDATE_NAMES: &[&str] = &["ref", "candidate_count"];
const REF_REASON_FIELDS: &[DetailFieldRule] = &[
    DetailFieldRule::required("ref", DetailFieldType::String),
    DetailFieldRule::required("reason", DetailFieldType::String),
];
const REF_REASON_NAMES: &[&str] = &["ref", "reason"];
const ADAPTER_REASON_FIELDS: &[DetailFieldRule] = &[
    DetailFieldRule::required("adapter_id", DetailFieldType::String),
    DetailFieldRule::required("reason", DetailFieldType::String),
    DetailFieldRule::optional("exit_code", DetailFieldType::I32),
    DetailFieldRule::optional("stderr", DetailFieldType::String),
];
const ADAPTER_REASON_NAMES: &[&str] = &["adapter_id", "reason"];
const INTERNAL_FIELDS: &[DetailFieldRule] = &[DetailFieldRule::required(
    "error_id",
    DetailFieldType::String,
)];
const INTERNAL_NAMES: &[&str] = &["error_id"];
const CLI_ARGV_FIELDS: &[DetailFieldRule] = &[DetailFieldRule::required(
    "tokens",
    DetailFieldType::StringArray,
)];
const ADAPTER_CANDIDATE_FIELDS: &[DetailFieldRule] = &[
    DetailFieldRule::required("adapter_id", DetailFieldType::String),
    DetailFieldRule::required("stage", DetailFieldType::String),
    DetailFieldRule::required("code", DetailFieldType::String),
    DetailFieldRule::optional("preselected", DetailFieldType::Boolean),
];
const ADAPTER_CONFIG_FIELDS: &[DetailFieldRule] = &[
    DetailFieldRule::required("source_level", DetailFieldType::String),
    DetailFieldRule::required("path_origin", DetailFieldType::String),
    DetailFieldRule::required("path", DetailFieldType::String),
    DetailFieldRule::required("reason_code", DetailFieldType::String),
];
const BOUNDARY_FIELDS: &[DetailFieldRule] = &[
    DetailFieldRule::required("reason", DetailFieldType::String),
    DetailFieldRule::optional("label", DetailFieldType::String),
];

const PROTOCOL_CODES: &[DiagnosticCode] = &[
    DiagnosticCode::Protocol(ProtocolDiagnosticCode::InvalidRequest),
    DiagnosticCode::Protocol(ProtocolDiagnosticCode::DocumentNotFound),
    DiagnosticCode::Protocol(ProtocolDiagnosticCode::DocumentPathInvalid),
    DiagnosticCode::Protocol(ProtocolDiagnosticCode::DocumentEncodingUnsupported),
    DiagnosticCode::Protocol(ProtocolDiagnosticCode::FormatUnknown),
    DiagnosticCode::Protocol(ProtocolDiagnosticCode::FormatAmbiguous),
    DiagnosticCode::Protocol(ProtocolDiagnosticCode::CapabilityUnsupported),
    DiagnosticCode::Protocol(ProtocolDiagnosticCode::RefNotFound),
    DiagnosticCode::Protocol(ProtocolDiagnosticCode::RefAmbiguous),
    DiagnosticCode::Protocol(ProtocolDiagnosticCode::RefInvalid),
    DiagnosticCode::Protocol(ProtocolDiagnosticCode::AdapterUnavailable),
    DiagnosticCode::Protocol(ProtocolDiagnosticCode::AdapterInvokeFailed),
    DiagnosticCode::Protocol(ProtocolDiagnosticCode::InternalError),
];

const READABLE_WARNING_CODES: &[DiagnosticCode] = &[
    DiagnosticCode::ReadableWarning(ReadableWarningDiagnosticCode::CliArgvIgnored),
    DiagnosticCode::ReadableWarning(ReadableWarningDiagnosticCode::AdapterCandidateFailure),
    DiagnosticCode::ReadableWarning(ReadableWarningDiagnosticCode::AdapterConfigSourceSkipped),
];

const BOUNDARY_CODES: &[DiagnosticCode] = &[
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::AdapterErrorExitCodeCannotBe),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::FailedToReadRequest),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::FailedToSerialize),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::FailedToWriteCliWarning),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::FailedToWriteJson),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::FailedToWriteProtocolResponse),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::FailedToWriteReadableView),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::InvalidRequestJson),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::ManifestAdapterIdMismatch),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::ManifestSchemaValidationFailed),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::ManifestSemanticValidationFailed),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::ProbeResultAdapterIdMismatch),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::ProbeResultSchemaValidationFailed),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::ProbeResultSemanticValidationFailed),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::ProtocolResponseSchemaValidationFailed),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::ProtocolResponseSemanticValidationFailed),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::ReadableViewRenderFailed),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::RequestDeserializationFailed),
    DiagnosticCode::Boundary(BoundaryDiagnosticCode::RequestSchemaValidationFailed),
];

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCode {
    Protocol(ProtocolDiagnosticCode),
    ReadableWarning(ReadableWarningDiagnosticCode),
    Boundary(BoundaryDiagnosticCode),
}

impl DiagnosticCode {
    pub fn all() -> impl Iterator<Item = DiagnosticCode> {
        PROTOCOL_CODES
            .iter()
            .chain(READABLE_WARNING_CODES.iter())
            .chain(BOUNDARY_CODES.iter())
            .copied()
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Protocol(code) => code.protocol_code(),
            Self::ReadableWarning(code) => code.warning_id(),
            Self::Boundary(code) => code.as_str(),
        }
    }

    pub const fn category(self) -> DiagnosticCategory {
        match self {
            Self::Protocol(code) => code.category(),
            Self::ReadableWarning(_) => DiagnosticCategory::Compatibility,
            Self::Boundary(code) => code.category(),
        }
    }

    pub const fn default_severity(self) -> DiagnosticSeverity {
        match self {
            Self::ReadableWarning(_) => DiagnosticSeverity::Warning,
            Self::Protocol(ProtocolDiagnosticCode::InternalError)
            | Self::Boundary(BoundaryDiagnosticCode::ReadableViewRenderFailed)
            | Self::Boundary(BoundaryDiagnosticCode::FailedToWriteCliWarning)
            | Self::Boundary(BoundaryDiagnosticCode::FailedToWriteJson)
            | Self::Boundary(BoundaryDiagnosticCode::FailedToWriteProtocolResponse)
            | Self::Boundary(BoundaryDiagnosticCode::FailedToWriteReadableView)
            | Self::Boundary(BoundaryDiagnosticCode::FailedToSerialize) => {
                DiagnosticSeverity::Fatal
            }
            Self::Protocol(_) | Self::Boundary(_) => DiagnosticSeverity::Error,
        }
    }

    pub const fn default_effect(self) -> DiagnosticEffect {
        match self {
            Self::ReadableWarning(ReadableWarningDiagnosticCode::CliArgvIgnored)
            | Self::ReadableWarning(ReadableWarningDiagnosticCode::AdapterConfigSourceSkipped) => {
                DiagnosticEffect::OperationContinued
            }
            Self::ReadableWarning(ReadableWarningDiagnosticCode::AdapterCandidateFailure) => {
                DiagnosticEffect::CandidateSkipped
            }
            Self::Protocol(ProtocolDiagnosticCode::InvalidRequest)
            | Self::Protocol(ProtocolDiagnosticCode::CapabilityUnsupported)
            | Self::Boundary(BoundaryDiagnosticCode::InvalidRequestJson)
            | Self::Boundary(BoundaryDiagnosticCode::RequestDeserializationFailed)
            | Self::Boundary(BoundaryDiagnosticCode::RequestSchemaValidationFailed) => {
                DiagnosticEffect::InputRejected
            }
            Self::Protocol(ProtocolDiagnosticCode::DocumentNotFound)
            | Self::Protocol(ProtocolDiagnosticCode::DocumentPathInvalid)
            | Self::Protocol(ProtocolDiagnosticCode::DocumentEncodingUnsupported)
            | Self::Protocol(ProtocolDiagnosticCode::FormatUnknown)
            | Self::Protocol(ProtocolDiagnosticCode::FormatAmbiguous)
            | Self::Protocol(ProtocolDiagnosticCode::RefNotFound)
            | Self::Protocol(ProtocolDiagnosticCode::RefAmbiguous)
            | Self::Protocol(ProtocolDiagnosticCode::RefInvalid) => {
                DiagnosticEffect::DocumentFailed
            }
            Self::Protocol(ProtocolDiagnosticCode::AdapterUnavailable)
            | Self::Protocol(ProtocolDiagnosticCode::AdapterInvokeFailed)
            | Self::Boundary(BoundaryDiagnosticCode::ManifestAdapterIdMismatch)
            | Self::Boundary(BoundaryDiagnosticCode::ManifestSchemaValidationFailed)
            | Self::Boundary(BoundaryDiagnosticCode::ManifestSemanticValidationFailed)
            | Self::Boundary(BoundaryDiagnosticCode::ProbeResultAdapterIdMismatch)
            | Self::Boundary(BoundaryDiagnosticCode::ProbeResultSchemaValidationFailed)
            | Self::Boundary(BoundaryDiagnosticCode::ProbeResultSemanticValidationFailed)
            | Self::Boundary(BoundaryDiagnosticCode::ProtocolResponseSchemaValidationFailed)
            | Self::Boundary(BoundaryDiagnosticCode::ProtocolResponseSemanticValidationFailed)
            | Self::Boundary(BoundaryDiagnosticCode::FailedToReadRequest) => {
                DiagnosticEffect::AdapterBoundaryFailed
            }
            Self::Protocol(ProtocolDiagnosticCode::InternalError)
            | Self::Boundary(BoundaryDiagnosticCode::AdapterErrorExitCodeCannotBe)
            | Self::Boundary(BoundaryDiagnosticCode::FailedToSerialize)
            | Self::Boundary(BoundaryDiagnosticCode::FailedToWriteCliWarning)
            | Self::Boundary(BoundaryDiagnosticCode::FailedToWriteJson)
            | Self::Boundary(BoundaryDiagnosticCode::FailedToWriteProtocolResponse)
            | Self::Boundary(BoundaryDiagnosticCode::FailedToWriteReadableView)
            | Self::Boundary(BoundaryDiagnosticCode::ReadableViewRenderFailed) => {
                DiagnosticEffect::InternalFailure
            }
        }
    }

    pub const fn details_rule(self) -> DiagnosticDetailsRule {
        match self {
            Self::Protocol(code) => code.details_rule(),
            Self::ReadableWarning(code) => code.details_rule(),
            Self::Boundary(_) => DiagnosticDetailsRule::exact(BOUNDARY_FIELDS),
        }
    }

    pub const fn projection_rule(self) -> DiagnosticProjectionRule {
        DiagnosticProjectionRule {
            protocol_code: match self {
                Self::Protocol(code) => Some(code.protocol_code()),
                Self::ReadableWarning(_) | Self::Boundary(_) => None,
            },
            readable_warning_id: match self {
                Self::ReadableWarning(code) => Some(code.warning_id()),
                Self::Protocol(_) | Self::Boundary(_) => None,
            },
            stderr: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolDiagnosticCode {
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

impl ProtocolDiagnosticCode {
    pub const fn protocol_code(self) -> &'static str {
        match self {
            Self::InvalidRequest => "INVALID_REQUEST",
            Self::DocumentNotFound => "DOCUMENT_NOT_FOUND",
            Self::DocumentPathInvalid => "DOCUMENT_PATH_INVALID",
            Self::DocumentEncodingUnsupported => "DOCUMENT_ENCODING_UNSUPPORTED",
            Self::FormatUnknown => "FORMAT_UNKNOWN",
            Self::FormatAmbiguous => "FORMAT_AMBIGUOUS",
            Self::CapabilityUnsupported => "CAPABILITY_UNSUPPORTED",
            Self::RefNotFound => "REF_NOT_FOUND",
            Self::RefAmbiguous => "REF_AMBIGUOUS",
            Self::RefInvalid => "REF_INVALID",
            Self::AdapterUnavailable => "ADAPTER_UNAVAILABLE",
            Self::AdapterInvokeFailed => "ADAPTER_INVOKE_FAILED",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }

    pub const fn category(self) -> DiagnosticCategory {
        match self {
            Self::InvalidRequest | Self::CapabilityUnsupported => DiagnosticCategory::Request,
            Self::DocumentNotFound
            | Self::DocumentPathInvalid
            | Self::DocumentEncodingUnsupported
            | Self::FormatUnknown
            | Self::FormatAmbiguous
            | Self::RefNotFound
            | Self::RefAmbiguous
            | Self::RefInvalid => DiagnosticCategory::Document,
            Self::AdapterUnavailable | Self::AdapterInvokeFailed => {
                DiagnosticCategory::AdapterBoundary
            }
            Self::InternalError => DiagnosticCategory::Internal,
        }
    }

    pub const fn details_rule(self) -> DiagnosticDetailsRule {
        match self {
            Self::InvalidRequest => DiagnosticDetailsRule::exact(FIELD_REASON_FIELDS),
            Self::DocumentNotFound => DiagnosticDetailsRule::exact(PATH_FIELDS),
            Self::DocumentPathInvalid => DiagnosticDetailsRule::exact(PATH_REASON_FIELDS),
            Self::DocumentEncodingUnsupported => DiagnosticDetailsRule::exact(PATH_ENCODING_FIELDS),
            Self::FormatUnknown => DiagnosticDetailsRule::exact(FORMAT_UNKNOWN_FIELDS),
            Self::FormatAmbiguous => DiagnosticDetailsRule::exact(FORMAT_AMBIGUOUS_FIELDS),
            Self::CapabilityUnsupported => DiagnosticDetailsRule::exact(CAPABILITY_ADAPTER_FIELDS),
            Self::RefNotFound => DiagnosticDetailsRule::exact(REF_FIELDS),
            Self::RefAmbiguous => DiagnosticDetailsRule::exact(REF_CANDIDATE_FIELDS),
            Self::RefInvalid => DiagnosticDetailsRule::exact(REF_REASON_FIELDS),
            Self::AdapterUnavailable | Self::AdapterInvokeFailed => {
                DiagnosticDetailsRule::exact(ADAPTER_REASON_FIELDS)
            }
            Self::InternalError => DiagnosticDetailsRule::exact(INTERNAL_FIELDS),
        }
    }

    pub const fn required_detail_names(self) -> &'static [&'static str] {
        match self {
            Self::InvalidRequest => FIELD_REASON_NAMES,
            Self::DocumentNotFound => PATH_NAMES,
            Self::DocumentPathInvalid => PATH_REASON_NAMES,
            Self::DocumentEncodingUnsupported => PATH_ENCODING_NAMES,
            Self::FormatUnknown => FORMAT_UNKNOWN_NAMES,
            Self::FormatAmbiguous => FORMAT_AMBIGUOUS_NAMES,
            Self::CapabilityUnsupported => CAPABILITY_ADAPTER_NAMES,
            Self::RefNotFound => REF_NAMES,
            Self::RefAmbiguous => REF_CANDIDATE_NAMES,
            Self::RefInvalid => REF_REASON_NAMES,
            Self::AdapterUnavailable | Self::AdapterInvokeFailed => ADAPTER_REASON_NAMES,
            Self::InternalError => INTERNAL_NAMES,
        }
    }
}

impl From<ProtocolDiagnosticCode> for DiagnosticCode {
    fn from(code: ProtocolDiagnosticCode) -> Self {
        Self::Protocol(code)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadableWarningDiagnosticCode {
    CliArgvIgnored,
    AdapterCandidateFailure,
    AdapterConfigSourceSkipped,
}

impl ReadableWarningDiagnosticCode {
    pub const fn warning_id(self) -> &'static str {
        match self {
            Self::CliArgvIgnored => "cli_argv_ignored",
            Self::AdapterCandidateFailure => "adapter_candidate_failure",
            Self::AdapterConfigSourceSkipped => "adapter_config_source_skipped",
        }
    }

    pub const fn details_rule(self) -> DiagnosticDetailsRule {
        match self {
            Self::CliArgvIgnored => DiagnosticDetailsRule::exact(CLI_ARGV_FIELDS),
            Self::AdapterCandidateFailure => DiagnosticDetailsRule::exact(ADAPTER_CANDIDATE_FIELDS),
            Self::AdapterConfigSourceSkipped => DiagnosticDetailsRule::exact(ADAPTER_CONFIG_FIELDS),
        }
    }
}

impl From<ReadableWarningDiagnosticCode> for DiagnosticCode {
    fn from(code: ReadableWarningDiagnosticCode) -> Self {
        Self::ReadableWarning(code)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryDiagnosticCode {
    AdapterErrorExitCodeCannotBe,
    FailedToReadRequest,
    FailedToSerialize,
    FailedToWriteCliWarning,
    FailedToWriteJson,
    FailedToWriteProtocolResponse,
    FailedToWriteReadableView,
    InvalidRequestJson,
    ManifestAdapterIdMismatch,
    ManifestSchemaValidationFailed,
    ManifestSemanticValidationFailed,
    ProbeResultAdapterIdMismatch,
    ProbeResultSchemaValidationFailed,
    ProbeResultSemanticValidationFailed,
    ProtocolResponseSchemaValidationFailed,
    ProtocolResponseSemanticValidationFailed,
    ReadableViewRenderFailed,
    RequestDeserializationFailed,
    RequestSchemaValidationFailed,
}

impl BoundaryDiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AdapterErrorExitCodeCannotBe => "adapter_error_exit_code_cannot_be",
            Self::FailedToReadRequest => "failed_to_read_request",
            Self::FailedToSerialize => "failed_to_serialize",
            Self::FailedToWriteCliWarning => "failed_to_write_cli_warning",
            Self::FailedToWriteJson => "failed_to_write_json",
            Self::FailedToWriteProtocolResponse => "failed_to_write_protocol_response",
            Self::FailedToWriteReadableView => "failed_to_write_readable_view",
            Self::InvalidRequestJson => "invalid_request_json",
            Self::ManifestAdapterIdMismatch => "manifest_adapter_id_mismatch",
            Self::ManifestSchemaValidationFailed => "manifest_schema_validation_failed",
            Self::ManifestSemanticValidationFailed => "manifest_semantic_validation_failed",
            Self::ProbeResultAdapterIdMismatch => "probe_result_adapter_id_mismatch",
            Self::ProbeResultSchemaValidationFailed => "probe_result_schema_validation_failed",
            Self::ProbeResultSemanticValidationFailed => "probe_result_semantic_validation_failed",
            Self::ProtocolResponseSchemaValidationFailed => {
                "protocol_response_schema_validation_failed"
            }
            Self::ProtocolResponseSemanticValidationFailed => {
                "protocol_response_semantic_validation_failed"
            }
            Self::ReadableViewRenderFailed => "readable_view_render_failed",
            Self::RequestDeserializationFailed => "request_deserialization_failed",
            Self::RequestSchemaValidationFailed => "request_schema_validation_failed",
        }
    }

    pub const fn category(self) -> DiagnosticCategory {
        match self {
            Self::InvalidRequestJson
            | Self::RequestDeserializationFailed
            | Self::RequestSchemaValidationFailed => DiagnosticCategory::Request,
            Self::FailedToReadRequest
            | Self::ManifestAdapterIdMismatch
            | Self::ManifestSchemaValidationFailed
            | Self::ManifestSemanticValidationFailed
            | Self::ProbeResultAdapterIdMismatch
            | Self::ProbeResultSchemaValidationFailed
            | Self::ProbeResultSemanticValidationFailed
            | Self::ProtocolResponseSchemaValidationFailed
            | Self::ProtocolResponseSemanticValidationFailed => DiagnosticCategory::AdapterBoundary,
            Self::AdapterErrorExitCodeCannotBe
            | Self::FailedToSerialize
            | Self::FailedToWriteCliWarning
            | Self::FailedToWriteJson
            | Self::FailedToWriteProtocolResponse
            | Self::FailedToWriteReadableView
            | Self::ReadableViewRenderFailed => DiagnosticCategory::Internal,
        }
    }
}

impl From<BoundaryDiagnosticCode> for DiagnosticCode {
    fn from(code: BoundaryDiagnosticCode) -> Self {
        Self::Boundary(code)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCategory {
    Request,
    Document,
    AdapterBoundary,
    Internal,
    Compatibility,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Warning,
    Error,
    Fatal,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticEffect {
    OperationContinued,
    CandidateSkipped,
    InputRejected,
    DocumentFailed,
    AdapterBoundaryFailed,
    InternalFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticProjectionRule {
    pub protocol_code: Option<&'static str>,
    pub readable_warning_id: Option<&'static str>,
    pub stderr: bool,
}
