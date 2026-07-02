mod candidate;
pub mod evidence;
mod state;

use crate::error::{AppError, AppResult};
use crate::project_paths::NormalizedDocumentPath;
use crate::registry::{AdapterRecord, AdapterRegistry};

pub use evidence::CandidateEvidence;

use candidate::{evaluate_candidate, evaluate_preselected, CandidateResult};
use state::SelectionState;

#[derive(Clone, Debug)]
pub struct AdapterSelection {
    pub record: AdapterRecord,
    pub evidence: Vec<CandidateEvidence>,
}

#[derive(Clone, Copy)]
pub struct AdapterSelectionRequest<'a> {
    pub registry: &'a AdapterRegistry,
    pub document: &'a NormalizedDocumentPath,
    pub preselected_adapter_id: Option<&'a str>,
    pub preselected_adapter_source: &'a str,
}

pub fn select_adapter(request: AdapterSelectionRequest<'_>) -> AppResult<AdapterSelection> {
    let AdapterSelectionRequest {
        registry,
        document,
        preselected_adapter_id,
        preselected_adapter_source,
    } = request;
    let mut state = SelectionState::default();

    if let Some(adapter_id) = preselected_adapter_id {
        state.mark_attempted(adapter_id);
        match evaluate_preselected(registry, adapter_id, document) {
            CandidateResult::Selected(selected) => {
                return Ok(state.into_selection(*selected));
            }
            CandidateResult::Continue(candidate) => {
                let error = explicit_adapter_error(&candidate, preselected_adapter_source);
                return Err(error);
            }
        }
    }

    for record in registry.adapters {
        if state.has_attempted(record.id()) {
            continue;
        }
        match evaluate_candidate(record, document) {
            CandidateResult::Selected(selected) => {
                return Ok(state.into_selection(*selected));
            }
            CandidateResult::Continue(candidate) => {
                state.record_failure(candidate);
            }
        }
    }

    Err(state.format_unknown(document))
}

fn explicit_adapter_error(candidate: &CandidateEvidence, selection_source: &str) -> AppError {
    AppError::adapter_unavailable_with_selection_context(
        candidate.adapter_id.clone(),
        candidate.reason.clone(),
        selection_source,
        candidate.stage.as_str(),
    )
}

#[cfg(test)]
mod tests;
