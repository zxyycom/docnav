use docnav_protocol::{Manifest, Operation, ProbeResult};

mod candidate;
pub mod evidence;
mod state;

use crate::error::{AppError, AppResult};
use crate::project_context::ProjectContext;
use crate::project_paths::NormalizedDocumentPath;
use crate::registry::{AdapterRecord, AdapterRegistry};

pub use evidence::CandidateEvidence;

use candidate::{
    evaluate_candidate, evaluate_preselected, infer_adapter_by_extension, CandidateResult,
};
use state::SelectionState;

#[derive(Clone, Debug)]
pub struct AdapterSelection {
    pub record: AdapterRecord,
    pub manifest: Manifest,
    pub probe: ProbeResult,
    pub evidence: Vec<CandidateEvidence>,
}

#[derive(Clone, Copy)]
pub struct AdapterSelectionRequest<'a> {
    pub project: &'a ProjectContext,
    pub registry: &'a AdapterRegistry,
    pub document: &'a NormalizedDocumentPath,
    pub operation: Operation,
    pub preselected_adapter_id: Option<&'a str>,
    pub preselected_adapter_source: &'a str,
}

pub fn select_adapter(request: AdapterSelectionRequest<'_>) -> AppResult<AdapterSelection> {
    let AdapterSelectionRequest {
        project,
        registry,
        document,
        operation,
        preselected_adapter_id,
        preselected_adapter_source,
    } = request;
    let mut state = SelectionState::default();

    if let Some(adapter_id) = preselected_adapter_id {
        state.mark_attempted(adapter_id);
        match evaluate_preselected(project, registry, adapter_id, document, operation) {
            CandidateResult::Selected(selected) => {
                return Ok(state.into_selection(*selected));
            }
            CandidateResult::Continue(candidate) => {
                let error = explicit_adapter_error(&candidate, preselected_adapter_source);
                return Err(error);
            }
        }
    }

    let inference = infer_adapter_by_extension(project, registry, document, operation);
    state.record_inference_failures(inference.evidence);
    if let Some(adapter_id) = inference.adapter_id {
        state.mark_attempted(&adapter_id);
        match evaluate_preselected(project, registry, &adapter_id, document, operation) {
            CandidateResult::Selected(selected) => {
                return Ok(state.into_selection(*selected));
            }
            CandidateResult::Continue(candidate) => {
                state.record_failure(candidate);
            }
        }
    }

    for record in &registry.adapters {
        if state.has_attempted(&record.id) {
            continue;
        }
        match evaluate_candidate(project, record, document, operation) {
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
