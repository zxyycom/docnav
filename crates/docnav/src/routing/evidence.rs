use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterSelectionWarning {
    pub adapter_id: String,
    pub stage: CandidateStage,
    pub code: String,
    pub reason: String,
    pub preselected: bool,
}

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
}

impl AdapterSelectionWarning {
    pub(super) fn candidate_failure(candidate: &CandidateEvidence, preselected: bool) -> Self {
        Self {
            adapter_id: candidate.adapter_id.clone(),
            stage: candidate.stage,
            code: candidate.code.clone(),
            reason: candidate.reason.clone(),
            preselected,
        }
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
