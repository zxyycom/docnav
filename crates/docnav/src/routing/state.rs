use std::collections::HashSet;

use super::candidate::SelectedCandidate;
use super::{AdapterSelection, CandidateEvidence};
use crate::error::AppError;
use crate::project_paths::NormalizedDocumentPath;

const FORMAT_UNKNOWN_REASON: &str = "NO_SUPPORTED_ADAPTER";

#[derive(Default)]
pub(super) struct SelectionState {
    evidence: Vec<CandidateEvidence>,
    attempted: HashSet<String>,
}

impl SelectionState {
    pub(super) fn record_failure(&mut self, candidate: CandidateEvidence) {
        self.evidence.push(candidate);
    }

    pub(super) fn mark_attempted(&mut self, adapter_id: &str) {
        self.attempted.insert(adapter_id.to_owned());
    }

    pub(super) fn has_attempted(&self, adapter_id: &str) -> bool {
        self.attempted.contains(adapter_id)
    }

    pub(super) fn into_selection(self, selected: SelectedCandidate) -> AdapterSelection {
        AdapterSelection {
            record: selected.record,
            evidence: self.evidence,
        }
    }

    pub(super) fn format_unknown(self, document: &NormalizedDocumentPath) -> AppError {
        let candidates = self
            .evidence
            .into_iter()
            .map(CandidateEvidence::into_format_candidate_details)
            .collect();
        AppError::format_unknown(
            document.adapter_path.clone(),
            FORMAT_UNKNOWN_REASON,
            candidates,
        )
    }
}
