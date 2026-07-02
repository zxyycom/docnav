use docnav_protocol::Operation;

mod candidate;
pub mod evidence;
mod state;

use crate::error::{AppError, AppResult};
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
    pub evidence: Vec<CandidateEvidence>,
}

#[derive(Clone, Copy)]
pub struct AdapterSelectionRequest<'a> {
    pub registry: &'a AdapterRegistry,
    pub document: &'a NormalizedDocumentPath,
    pub operation: Operation,
    pub preselected_adapter_id: Option<&'a str>,
    pub preselected_adapter_source: &'a str,
}

pub fn select_adapter(request: AdapterSelectionRequest<'_>) -> AppResult<AdapterSelection> {
    let AdapterSelectionRequest {
        registry,
        document,
        operation,
        preselected_adapter_id,
        preselected_adapter_source,
    } = request;
    let mut state = SelectionState::default();

    if let Some(adapter_id) = preselected_adapter_id {
        state.mark_attempted(adapter_id);
        match evaluate_preselected(registry, adapter_id, document, operation) {
            CandidateResult::Selected(selected) => {
                return Ok(state.into_selection(*selected));
            }
            CandidateResult::Continue(candidate) => {
                let error = explicit_adapter_error(&candidate, preselected_adapter_source);
                return Err(error);
            }
        }
    }

    let inference = infer_adapter_by_extension(registry, document, operation);
    state.record_inference_failures(inference.evidence);
    if let Some(adapter_id) = inference.adapter_id {
        state.mark_attempted(&adapter_id);
        match evaluate_preselected(registry, &adapter_id, document, operation) {
            CandidateResult::Selected(selected) => {
                return Ok(state.into_selection(*selected));
            }
            CandidateResult::Continue(candidate) => {
                state.record_failure(candidate);
            }
        }
    }

    for record in registry.adapters {
        if state.has_attempted(record.id()) {
            continue;
        }
        match evaluate_candidate(record, document, operation) {
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
mod tests {
    use std::path::PathBuf;

    use docnav_diagnostics::DiagnosticStack;
    use docnav_protocol::{Operation, ProtocolDiagnosticCode, ProtocolError};

    use super::*;

    // @case WB-CORE-ADAPTER-SOURCE-001
    #[test]
    fn explicit_missing_adapter_guidance_stays_on_static_registry() {
        let registry = AdapterRegistry { adapters: &[] };
        let document = NormalizedDocumentPath {
            adapter_path: "docs/guide.md".to_owned(),
            absolute_path: PathBuf::from("docs/guide.md"),
        };

        let error = select_adapter(AdapterSelectionRequest {
            registry: &registry,
            document: &document,
            operation: Operation::Outline,
            preselected_adapter_id: Some("custom-local-adapter"),
            preselected_adapter_source: "cli",
        })
        .expect_err("missing explicit adapter should fail");

        let mut diagnostics = DiagnosticStack::new();
        let id = diagnostics
            .push(error.diagnostic().clone())
            .expect("routing diagnostic should be valid");
        let record = diagnostics.get(id).expect("diagnostic record");
        let protocol_error =
            ProtocolError::from_diagnostic_record(record).expect("protocol projection");

        assert_eq!(
            protocol_error.code(),
            ProtocolDiagnosticCode::AdapterUnavailable
        );
        assert_eq!(protocol_error.owner(), "adapter_selection");
        let guidance = protocol_error
            .guidance()
            .and_then(|items| items.first())
            .expect("adapter selection guidance");
        assert!(guidance.contains("current core release static registry"));
        for removed_term in ["install", "register", "executable", "artifact"] {
            assert!(
                !guidance.contains(removed_term),
                "guidance should not mention {removed_term}: {guidance}"
            );
        }
    }
}
