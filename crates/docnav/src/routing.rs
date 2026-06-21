use docnav_protocol::{Manifest, Operation, ProbeResult};

mod candidate;
pub mod evidence;
mod state;

use crate::error::AppResult;
use crate::project_context::ProjectContext;
use crate::project_paths::NormalizedDocumentPath;
use crate::registry::{AdapterRecord, AdapterRegistry};

pub use evidence::{AdapterSelectionWarning, CandidateEvidence};

use candidate::{
    evaluate_candidate, evaluate_preselected, infer_adapter_by_extension, CandidateResult,
};
use state::{PreselectedSource, SelectionState};

#[derive(Clone, Debug)]
pub struct AdapterSelection {
    pub record: AdapterRecord,
    pub manifest: Manifest,
    pub probe: ProbeResult,
    pub evidence: Vec<CandidateEvidence>,
    pub warnings: Vec<AdapterSelectionWarning>,
}

#[derive(Clone, Copy)]
pub struct AdapterSelectionRequest<'a> {
    pub project: &'a ProjectContext,
    pub registry: &'a AdapterRegistry,
    pub document: &'a NormalizedDocumentPath,
    pub operation: Operation,
    pub preselected_adapter_id: Option<&'a str>,
    pub preselected_source: &'a str,
}

pub fn select_adapter(request: AdapterSelectionRequest<'_>) -> AppResult<AdapterSelection> {
    let AdapterSelectionRequest {
        project,
        registry,
        document,
        operation,
        preselected_adapter_id,
        preselected_source,
    } = request;
    let mut state = SelectionState::default();
    let preselected_source = PreselectedSource::from_raw(preselected_source);

    let preselected = match preselected_adapter_id {
        Some(adapter_id) => Some(adapter_id.to_owned()),
        None => {
            let inference = infer_adapter_by_extension(project, registry, document, operation);
            state.record_inference_failures(inference.evidence);
            inference.adapter_id
        }
    };

    if let Some(adapter_id) = preselected {
        state.mark_attempted(&adapter_id);
        match evaluate_preselected(project, registry, &adapter_id, document, operation) {
            CandidateResult::Selected(selected) => {
                return Ok(state.into_selection(*selected));
            }
            CandidateResult::Continue(candidate) => {
                state.record_failure(candidate, preselected_source.is_explicit());
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
                state.record_failure(candidate, false);
            }
        }
    }

    Err(state.format_unknown(document).into())
}
