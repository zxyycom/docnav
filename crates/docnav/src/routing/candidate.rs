use docnav_protocol::{Manifest, Operation, ProbeResult};
use serde_json::json;

use super::CandidateEvidence;
use crate::adapter_output_contract::{
    ensure_capability, manifest_from_output, probe_from_output, process_error_details,
};
use crate::adapter_process::{run_manifest, run_probe};
use crate::project_context::ProjectContext;
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
    pub(super) manifest: Manifest,
    pub(super) probe: ProbeResult,
}

pub(super) fn infer_adapter_by_extension(
    project: &ProjectContext,
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

    for record in &registry.adapters {
        let manifest = match manifest_for_candidate(project, record) {
            Ok(manifest) => manifest,
            Err(candidate) => {
                evidence.push(candidate);
                continue;
            }
        };
        if !manifest_matches_extension(&manifest, &extension) {
            continue;
        }
        if let Err(candidate) = capability_for_candidate(record, &manifest, operation) {
            evidence.push(candidate);
            continue;
        }
        return ExtensionInference {
            adapter_id: Some(record.id.clone()),
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
    project: &ProjectContext,
    registry: &AdapterRegistry,
    adapter_id: &str,
    document: &NormalizedDocumentPath,
    operation: Operation,
) -> CandidateResult {
    let Some(record) = registry.find(adapter_id) else {
        return CandidateResult::Continue(CandidateEvidence::resolve(
            adapter_id,
            "ADAPTER_NOT_FOUND",
            "adapter id is not present in the temporary registry",
            json!({}),
        ));
    };

    evaluate_candidate(project, record, document, operation)
}

pub(super) fn evaluate_candidate(
    project: &ProjectContext,
    record: &AdapterRecord,
    document: &NormalizedDocumentPath,
    operation: Operation,
) -> CandidateResult {
    let manifest = match manifest_for_candidate(project, record) {
        Ok(manifest) => manifest,
        Err(candidate) => return CandidateResult::Continue(candidate),
    };

    if let Err(candidate) = capability_for_candidate(record, &manifest, operation) {
        return CandidateResult::Continue(candidate);
    }

    let probe = match probe_for_candidate(project, record, document) {
        Ok(probe) => probe,
        Err(candidate) => return CandidateResult::Continue(candidate),
    };

    if !probe.supported {
        return CandidateResult::Continue(CandidateEvidence::probe(
            &record.id,
            "PROBE_UNSUPPORTED",
            "adapter probe returned supported=false",
            json!({ "probe": probe }),
        ));
    }

    CandidateResult::Selected(Box::new(SelectedCandidate {
        record: record.clone(),
        manifest,
        probe,
    }))
}

fn manifest_for_candidate(
    project: &ProjectContext,
    record: &AdapterRecord,
) -> Result<Manifest, CandidateEvidence> {
    match run_manifest(&project.project_root, record) {
        Ok(output) => manifest_from_output(&record.id, output).map_err(|reason| {
            CandidateEvidence::resolve(&record.id, "MANIFEST_INVALID", reason, json!({}))
        }),
        Err(error) => Err(CandidateEvidence::resolve(
            &record.id,
            "ADAPTER_UNAVAILABLE",
            error.reason.clone(),
            process_error_details(&error),
        )),
    }
}

fn capability_for_candidate(
    record: &AdapterRecord,
    manifest: &Manifest,
    operation: Operation,
) -> Result<(), CandidateEvidence> {
    ensure_capability(manifest, operation).map_err(|reason| {
        CandidateEvidence::resolve(
            &record.id,
            "CAPABILITY_UNSUPPORTED",
            reason,
            json!({ "capability": operation, "adapter_id": record.id }),
        )
    })
}

fn probe_for_candidate(
    project: &ProjectContext,
    record: &AdapterRecord,
    document: &NormalizedDocumentPath,
) -> Result<ProbeResult, CandidateEvidence> {
    match run_probe(&project.project_root, record, &document.adapter_path) {
        Ok(output) => match probe_from_output(&record.id, &document.adapter_path, output) {
            Ok(probe) => Ok(probe),
            Err(reason) => Err(CandidateEvidence::probe(
                &record.id,
                "PROBE_INVALID",
                reason,
                json!({}),
            )),
        },
        Err(error) => Err(CandidateEvidence::probe(
            &record.id,
            "ADAPTER_UNAVAILABLE",
            error.reason.clone(),
            process_error_details(&error),
        )),
    }
}

fn document_extension(path: &str) -> Option<String> {
    let basename = path.rsplit('/').next().unwrap_or(path);
    let dot = basename.rfind('.')?;
    if dot == 0 {
        return None;
    }
    Some(basename[dot..].to_owned())
}
