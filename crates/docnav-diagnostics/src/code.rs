use serde::Serialize;

use crate::details::DiagnosticDetailsRule;

use self::rules::{BoundaryDiagnosticRule, ProtocolDiagnosticRule, BOUNDARY_RULES, PROTOCOL_RULES};

mod details;
mod rules;
mod typed;

pub use typed::{
    typed_codes, BoundaryDiagnosticMarker, DiagnosticCodeMarker, ProtocolDiagnosticMarker,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCode {
    Protocol(ProtocolDiagnosticCode),
    Boundary(BoundaryDiagnosticCode),
}

impl DiagnosticCode {
    pub fn all() -> impl Iterator<Item = DiagnosticCode> {
        PROTOCOL_RULES
            .iter()
            .map(|rule| Self::Protocol(rule.code))
            .chain(BOUNDARY_RULES.iter().map(|rule| Self::Boundary(rule.code)))
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Protocol(code) => code.protocol_code(),
            Self::Boundary(code) => code.as_str(),
        }
    }

    pub const fn category(self) -> DiagnosticCategory {
        match self {
            Self::Protocol(code) => code.category(),
            Self::Boundary(code) => code.category(),
        }
    }

    pub const fn default_severity(self) -> DiagnosticSeverity {
        match self {
            Self::Protocol(code) => code.rule().severity,
            Self::Boundary(code) => code.rule().severity,
        }
    }

    pub const fn default_effect(self) -> DiagnosticEffect {
        match self {
            Self::Protocol(code) => code.rule().effect,
            Self::Boundary(code) => code.rule().effect,
        }
    }

    pub const fn details_rule(self) -> DiagnosticDetailsRule {
        match self {
            Self::Protocol(code) => code.details_rule(),
            Self::Boundary(code) => code.rule().details,
        }
    }

    pub const fn projection_rule(self) -> DiagnosticProjectionRule {
        DiagnosticProjectionRule {
            protocol_code: match self {
                Self::Protocol(code) => Some(code.protocol_code()),
                Self::Boundary(_) => None,
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
pub enum BoundaryDiagnosticCode {
    FailedToReadRequest,
    FailedToSerialize,
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
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Error,
    Fatal,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticEffect {
    InputRejected,
    DocumentFailed,
    AdapterBoundaryFailed,
    InternalFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticProjectionRule {
    pub protocol_code: Option<&'static str>,
    pub stderr: bool,
}
