use serde::Serialize;
use serde_json::Value;

use docnav_diagnostics::FormatCandidateDetails;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CandidateEvidence {
    pub adapter_id: String,
    pub stage: CandidateStage,
    pub code: String,
    pub reason: String,
    pub details: Value,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CandidateStage {
    Resolve,
    Probe,
}

impl CandidateEvidence {
    pub(super) fn resolve(
        adapter_id: &str,
        code: &str,
        reason: impl Into<String>,
        details: Value,
    ) -> Self {
        Self {
            adapter_id: adapter_id.to_owned(),
            stage: CandidateStage::Resolve,
            code: code.to_owned(),
            reason: reason.into(),
            details,
        }
    }

    pub(super) fn probe(
        adapter_id: &str,
        code: &str,
        reason: impl Into<String>,
        details: Value,
    ) -> Self {
        Self {
            adapter_id: adapter_id.to_owned(),
            stage: CandidateStage::Probe,
            code: code.to_owned(),
            reason: reason.into(),
            details,
        }
    }

    pub(super) fn into_format_candidate_details(self) -> FormatCandidateDetails {
        FormatCandidateDetails::new(self.adapter_id, self.stage.as_str(), self.code)
    }
}

impl CandidateStage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Resolve => "resolve",
            Self::Probe => "probe",
        }
    }
}
