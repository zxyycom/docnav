use std::fmt;

use serde::Serialize;

use crate::code::{DiagnosticCode, DiagnosticCodeMarker, DiagnosticEffect, DiagnosticSeverity};
use crate::details::{DiagnosticDetails, DiagnosticDetailsError};
use crate::stack::DiagnosticId;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DiagnosticSource {
    pub component: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
}

impl DiagnosticSource {
    pub fn new(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            stage: None,
        }
    }

    pub fn with_stage(component: impl Into<String>, stage: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            stage: Some(stage.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DiagnosticRecord {
    id: DiagnosticId,
    summary: String,
    severity: DiagnosticSeverity,
    code: DiagnosticCode,
    effect: DiagnosticEffect,
    details: DiagnosticDetails,
    #[serde(skip_serializing_if = "Option::is_none")]
    guidance: Option<Vec<String>>,
    source: DiagnosticSource,
    recoverable: bool,
}

macro_rules! diagnostic_record_accessors {
    () => {
        pub fn summary(&self) -> &str {
            &self.summary
        }

        pub const fn severity(&self) -> DiagnosticSeverity {
            self.severity
        }

        pub const fn code(&self) -> DiagnosticCode {
            self.code
        }

        pub const fn effect(&self) -> DiagnosticEffect {
            self.effect
        }

        pub const fn details(&self) -> &DiagnosticDetails {
            &self.details
        }

        pub fn guidance(&self) -> Option<&[String]> {
            self.guidance.as_deref()
        }

        pub const fn source(&self) -> &DiagnosticSource {
            &self.source
        }
    };
}

impl DiagnosticRecord {
    pub const fn id(&self) -> DiagnosticId {
        self.id
    }

    diagnostic_record_accessors!();

    pub const fn recoverable(&self) -> bool {
        self.recoverable
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiagnosticRecordDraft {
    summary: String,
    severity: DiagnosticSeverity,
    code: DiagnosticCode,
    effect: DiagnosticEffect,
    details: DiagnosticDetails,
    guidance: Option<Vec<String>>,
    source: DiagnosticSource,
    recoverable: bool,
}

impl DiagnosticRecordDraft {
    pub fn new<C>(summary: impl Into<String>, details: C::Details, source: DiagnosticSource) -> Self
    where
        C: DiagnosticCodeMarker,
    {
        Self::from_typed_parts(C::CODE, summary, details.into(), source)
    }

    fn from_typed_parts(
        code: DiagnosticCode,
        summary: impl Into<String>,
        details: DiagnosticDetails,
        source: DiagnosticSource,
    ) -> Self {
        let severity = code.default_severity();
        let effect = code.default_effect();
        Self {
            summary: summary.into(),
            severity,
            code,
            effect,
            details,
            guidance: None,
            source,
            recoverable: is_recoverable_effect(effect),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_erased_for_test(
        code: impl Into<DiagnosticCode>,
        summary: impl Into<String>,
        details: DiagnosticDetails,
        source: DiagnosticSource,
    ) -> Self {
        Self::from_typed_parts(code.into(), summary, details, source)
    }

    diagnostic_record_accessors!();

    pub const fn recoverable_status(&self) -> bool {
        self.recoverable
    }

    pub fn with_severity(mut self, severity: DiagnosticSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_effect(mut self, effect: DiagnosticEffect) -> Self {
        self.effect = effect;
        self.recoverable = is_recoverable_effect(effect);
        self
    }

    pub fn recoverable(mut self, recoverable: bool) -> Self {
        self.recoverable = recoverable;
        self
    }

    pub fn with_guidance(mut self, guidance: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.guidance = Some(guidance.into_iter().map(Into::into).collect());
        self
    }

    pub(crate) fn into_record(
        self,
        id: DiagnosticId,
    ) -> Result<DiagnosticRecord, DiagnosticRecordError> {
        if self.summary.is_empty() {
            return Err(DiagnosticRecordError::EmptySummary);
        }
        self.code
            .details_rule()
            .validate_value(&self.details.to_value())
            .map_err(DiagnosticRecordError::InvalidDetails)?;
        Ok(DiagnosticRecord {
            id,
            summary: self.summary,
            severity: self.severity,
            code: self.code,
            effect: self.effect,
            details: self.details,
            guidance: self.guidance,
            source: self.source,
            recoverable: self.recoverable,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DiagnosticRecordError {
    EmptySummary,
    InvalidDetails(DiagnosticDetailsError),
}

impl fmt::Display for DiagnosticRecordError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySummary => formatter.write_str("diagnostic summary must not be empty"),
            Self::InvalidDetails(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for DiagnosticRecordError {}

const fn is_recoverable_effect(effect: DiagnosticEffect) -> bool {
    matches!(
        effect,
        DiagnosticEffect::OperationContinued | DiagnosticEffect::CandidateSkipped
    )
}
