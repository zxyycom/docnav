use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeResult {
    pub probe_version: String,
    pub adapter_id: String,
    pub path: String,
    pub supported: bool,
    pub format: Option<String>,
    pub confidence: f64,
    pub reasons: Vec<ProbeReason>,
}

impl ProbeResult {
    pub fn validate_semantics(&self) -> Result<(), ProbeValidationError> {
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(ProbeValidationError::ConfidenceOutOfRange(self.confidence));
        }
        if self.reasons.is_empty() {
            return Err(ProbeValidationError::MissingReasons);
        }
        if self.supported && self.format.is_none() {
            return Err(ProbeValidationError::SupportedWithoutFormat);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProbeValidationError {
    ConfidenceOutOfRange(f64),
    MissingReasons,
    SupportedWithoutFormat,
}

impl fmt::Display for ProbeValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfidenceOutOfRange(confidence) => {
                write!(formatter, "probe confidence {confidence} is outside 0..1")
            }
            Self::MissingReasons => formatter.write_str("probe reasons must not be empty"),
            Self::SupportedWithoutFormat => {
                formatter.write_str("probe supported=true requires a format")
            }
        }
    }
}

impl std::error::Error for ProbeValidationError {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeReason {
    pub code: ProbeReasonCode,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProbeReasonCode {
    ExtensionMatch,
    ContentMatch,
    ContentConflict,
    ReadError,
}
