use serde::Serialize;

use crate::details::DiagnosticDetailsRule;

use self::rules::{
    BoundaryDiagnosticRule, ProtocolDiagnosticRule, ReadableWarningDiagnosticRule, BOUNDARY_RULES,
    PROTOCOL_RULES, READABLE_WARNING_RULES,
};

mod details;
mod rules;
mod typed;

pub use typed::{
    typed_codes, BoundaryDiagnosticMarker, DiagnosticCodeMarker, ProtocolDiagnosticMarker,
    ReadableWarningDiagnosticMarker,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCode {
    Protocol(ProtocolDiagnosticCode),
    ReadableWarning(ReadableWarningDiagnosticCode),
    Boundary(BoundaryDiagnosticCode),
}

impl DiagnosticCode {
    pub fn all() -> impl Iterator<Item = DiagnosticCode> {
        PROTOCOL_RULES
            .iter()
            .map(|rule| Self::Protocol(rule.code))
            .chain(
                READABLE_WARNING_RULES
                    .iter()
                    .map(|rule| Self::ReadableWarning(rule.code)),
            )
            .chain(BOUNDARY_RULES.iter().map(|rule| Self::Boundary(rule.code)))
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
            Self::Protocol(code) => code.rule().severity,
            Self::ReadableWarning(_) => DiagnosticSeverity::Warning,
            Self::Boundary(code) => code.rule().severity,
        }
    }

    pub const fn default_effect(self) -> DiagnosticEffect {
        match self {
            Self::Protocol(code) => code.rule().effect,
            Self::ReadableWarning(code) => code.rule().effect,
            Self::Boundary(code) => code.rule().effect,
        }
    }

    pub const fn details_rule(self) -> DiagnosticDetailsRule {
        match self {
            Self::Protocol(code) => code.details_rule(),
            Self::ReadableWarning(code) => code.details_rule(),
            Self::Boundary(code) => code.rule().details,
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
    const fn rule(self) -> ProtocolDiagnosticRule {
        PROTOCOL_RULES[self as usize]
    }

    pub const fn protocol_code(self) -> &'static str {
        self.rule().protocol_code
    }

    pub fn from_protocol_code(value: &str) -> Option<Self> {
        PROTOCOL_RULES
            .iter()
            .find_map(|rule| (rule.protocol_code == value).then_some(rule.code))
    }

    pub const fn category(self) -> DiagnosticCategory {
        self.rule().category
    }

    pub const fn details_rule(self) -> DiagnosticDetailsRule {
        self.rule().details
    }

    pub fn required_detail_names(self) -> impl Iterator<Item = &'static str> {
        self.details_rule().required_field_names()
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
    const fn rule(self) -> ReadableWarningDiagnosticRule {
        READABLE_WARNING_RULES[self as usize]
    }

    pub fn from_warning_id(value: &str) -> Option<Self> {
        READABLE_WARNING_RULES
            .iter()
            .find_map(|rule| (rule.warning_id == value).then_some(rule.code))
    }

    pub const fn warning_id(self) -> &'static str {
        self.rule().warning_id
    }

    pub const fn details_rule(self) -> DiagnosticDetailsRule {
        self.rule().details
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
    const fn rule(self) -> BoundaryDiagnosticRule {
        BOUNDARY_RULES[self as usize]
    }

    pub const fn as_str(self) -> &'static str {
        self.rule().id
    }

    pub const fn category(self) -> DiagnosticCategory {
        self.rule().category
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
