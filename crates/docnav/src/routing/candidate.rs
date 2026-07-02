use docnav_protocol::ProbeResult;
use serde_json::json;

use super::CandidateEvidence;
use crate::project_paths::NormalizedDocumentPath;
use crate::registry::{AdapterRecord, AdapterRegistry};

pub(super) enum CandidateResult {
    Selected(Box<SelectedCandidate>),
    Continue(CandidateEvidence),
}

#[derive(Clone, Debug)]
pub(super) struct SelectedCandidate {
    pub(super) record: AdapterRecord,
}

pub(super) fn evaluate_preselected(
    registry: &AdapterRegistry,
    adapter_id: &str,
    document: &NormalizedDocumentPath,
) -> CandidateResult {
    let Some(record) = registry.find(adapter_id) else {
        return CandidateResult::Continue(CandidateEvidence::resolve(
            adapter_id,
            "ADAPTER_NOT_FOUND",
            "adapter id is not present in the core release static registry",
            json!({}),
        ));
    };

    evaluate_candidate(record, document)
}

pub(super) fn evaluate_candidate(
    record: &AdapterRecord,
    document: &NormalizedDocumentPath,
) -> CandidateResult {
    let probe = probe_for_candidate(record, document);
    if let Err(candidate) = probe_is_valid(record, document, &probe) {
        return CandidateResult::Continue(candidate);
    }

    if !probe.supported {
        return CandidateResult::Continue(CandidateEvidence::probe(
            record.id(),
            "PROBE_UNSUPPORTED",
            "adapter probe returned supported=false",
            json!({ "probe": probe }),
        ));
    }

    CandidateResult::Selected(Box::new(SelectedCandidate { record: *record }))
}

fn probe_for_candidate(record: &AdapterRecord, document: &NormalizedDocumentPath) -> ProbeResult {
    record.probe(&document.adapter_path)
}

fn probe_is_valid(
    record: &AdapterRecord,
    document: &NormalizedDocumentPath,
    probe: &ProbeResult,
) -> Result<(), CandidateEvidence> {
    if probe.adapter_id != record.id() {
        return Err(CandidateEvidence::probe(
            record.id(),
            "PROBE_INVALID",
            format!(
                "probe adapter_id {:?} does not match registry id {:?}",
                probe.adapter_id,
                record.id()
            ),
            json!({}),
        ));
    }
    if probe.path != document.adapter_path {
        return Err(CandidateEvidence::probe(
            record.id(),
            "PROBE_INVALID",
            "probe path does not match requested document path",
            json!({}),
        ));
    }
    if let Err(error) = probe.validate_semantics() {
        return Err(CandidateEvidence::probe(
            record.id(),
            "PROBE_INVALID",
            error.to_string(),
            json!({}),
        ));
    }
    Ok(())
}
