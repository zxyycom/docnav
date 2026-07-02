use docnav_protocol::{Manifest, Operation, ProbeResult};
use serde_json::json;

use super::CandidateEvidence;
use crate::project_paths::NormalizedDocumentPath;
use crate::registry::{AdapterRecord, AdapterRegistry};

pub(super) enum CandidateResult {
    Selected(Box<SelectedCandidate>),
    Continue(CandidateEvidence),
}

pub(super) struct ExtensionInference {
    pub(super) adapter_id: Option<String>,
    pub(super) evidence: Vec<CandidateEvidence>,
}

#[derive(Clone, Debug)]
pub(super) struct SelectedCandidate {
    pub(super) record: AdapterRecord,
}

pub(super) fn infer_adapter_by_extension(
    registry: &AdapterRegistry,
    document: &NormalizedDocumentPath,
    operation: Operation,
) -> ExtensionInference {
    let extension = document_extension(&document.adapter_path);
    let Some(extension) = extension else {
        return ExtensionInference {
            adapter_id: None,
            evidence: Vec::new(),
        };
    };
    let mut evidence = Vec::new();

    for record in registry.adapters {
        let manifest = record.manifest();
        if !manifest_matches_extension(&manifest, &extension) {
            continue;
        }
        if let Err(candidate) = capability_for_candidate(record, &manifest, operation) {
            evidence.push(candidate);
            continue;
        }
        return ExtensionInference {
            adapter_id: Some(record.id().to_owned()),
            evidence,
        };
    }

    ExtensionInference {
        adapter_id: None,
        evidence,
    }
}

fn manifest_matches_extension(manifest: &Manifest, extension: &str) -> bool {
    manifest.formats.iter().any(|format| {
        format
            .extensions
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(extension))
    })
}

pub(super) fn evaluate_preselected(
    registry: &AdapterRegistry,
    adapter_id: &str,
    document: &NormalizedDocumentPath,
    operation: Operation,
) -> CandidateResult {
    let Some(record) = registry.find(adapter_id) else {
        return CandidateResult::Continue(CandidateEvidence::resolve(
            adapter_id,
            "ADAPTER_NOT_FOUND",
            "adapter id is not present in the core release static registry",
            json!({}),
        ));
    };

    evaluate_candidate(record, document, operation)
}

pub(super) fn evaluate_candidate(
    record: &AdapterRecord,
    document: &NormalizedDocumentPath,
    operation: Operation,
) -> CandidateResult {
    let manifest = record.manifest();

    if let Err(candidate) = capability_for_candidate(record, &manifest, operation) {
        return CandidateResult::Continue(candidate);
    }

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

fn capability_for_candidate(
    record: &AdapterRecord,
    manifest: &Manifest,
    operation: Operation,
) -> Result<(), CandidateEvidence> {
    if manifest.capabilities.contains(&operation) {
        return Ok(());
    }

    Err({
        let reason = format!("adapter does not declare capability {operation}");
        CandidateEvidence::resolve(
            record.id(),
            "CAPABILITY_UNSUPPORTED",
            reason,
            json!({ "capability": operation, "adapter_id": record.id() }),
        )
    })
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

fn document_extension(path: &str) -> Option<String> {
    let basename = path.rsplit('/').next().unwrap_or(path);
    let dot = basename.rfind('.')?;
    if dot == 0 {
        return None;
    }
    Some(basename[dot..].to_owned())
}
