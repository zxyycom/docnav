use std::fmt;

use serde::Serialize;

use crate::code::{DiagnosticCode, DiagnosticEffect, DiagnosticSeverity};
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
    pub id: DiagnosticId,
    pub summary: String,
    pub severity: DiagnosticSeverity,
    pub code: DiagnosticCode,
    pub effect: DiagnosticEffect,
    pub details: DiagnosticDetails,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidance: Option<Vec<String>>,
    pub source: DiagnosticSource,
    pub recoverable: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiagnosticRecordDraft {
    pub summary: String,
    pub severity: DiagnosticSeverity,
    pub code: DiagnosticCode,
    pub effect: DiagnosticEffect,
    pub details: DiagnosticDetails,
    pub guidance: Option<Vec<String>>,
    pub source: DiagnosticSource,
    pub recoverable: bool,
}

impl DiagnosticRecordDraft {
    pub fn new(
        code: impl Into<DiagnosticCode>,
        summary: impl Into<String>,
        details: DiagnosticDetails,
        source: DiagnosticSource,
    ) -> Self {
        let code = code.into();
        let severity = code.default_severity();
        let effect = code.default_effect();
        let recoverable = matches!(
            effect,
            DiagnosticEffect::OperationContinued | DiagnosticEffect::CandidateSkipped
        );
        Self {
            summary: summary.into(),
            severity,
            code,
            effect,
            details,
            guidance: None,
            source,
            recoverable,
        }
    }

    pub fn with_severity(mut self, severity: DiagnosticSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_effect(mut self, effect: DiagnosticEffect) -> Self {
        self.effect = effect;
        self.recoverable = matches!(
            effect,
            DiagnosticEffect::OperationContinued | DiagnosticEffect::CandidateSkipped
        );
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
