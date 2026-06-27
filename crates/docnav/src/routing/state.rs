use std::collections::HashSet;

use serde_json::Value;

use super::candidate::SelectedCandidate;
use super::{AdapterSelection, AdapterSelectionWarning, CandidateEvidence};
use crate::error::AppError;
use crate::project_paths::NormalizedDocumentPath;

#[derive(Default)]
pub(super) struct SelectionState {
    evidence: Vec<CandidateEvidence>,
    warnings: Vec<AdapterSelectionWarning>,
    attempted: HashSet<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PreselectedSource {
    Explicit,
    Other,
}

impl SelectionState {
    pub(super) fn record_inference_failures(&mut self, candidates: Vec<CandidateEvidence>) {
        for candidate in candidates {
            self.mark_attempted(&candidate.adapter_id);
            self.record_failure(candidate, false);
        }
    }

    pub(super) fn record_failure(&mut self, candidate: CandidateEvidence, preselected: bool) {
        self.warnings
            .push(AdapterSelectionWarning::candidate_failure(
                &candidate,
                preselected,
            ));
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
            manifest: selected.manifest,
            probe: selected.probe,
            evidence: self.evidence,
            warnings: self.warnings,
        }
    }

    pub(super) fn format_unknown(self, document: &NormalizedDocumentPath) -> AppError {
        AppError::format_unknown(
            document.adapter_path.clone(),
            "no registered adapter supports the document and operation",
            serde_json::to_value(&self.evidence).unwrap_or_else(|_| Value::Array(Vec::new())),
        )
    }
}

impl PreselectedSource {
    pub(super) fn from_raw(source: &str) -> Self {
        if source == "explicit" {
            Self::Explicit
        } else {
            Self::Other
        }
    }

    pub(super) const fn is_explicit(self) -> bool {
        matches!(self, Self::Explicit)
    }
}
