use std::collections::HashSet;

use docnav_protocol::{Manifest, Operation, ProbeResult, StableError};
use serde::Serialize;
use serde_json::{json, Value};

use crate::adapter_output_contract::{
    ensure_capability, manifest_from_output, probe_from_output, process_error_details,
};
use crate::adapter_process::{run_manifest, run_probe};
use crate::error::AppResult;
use crate::project_context::ProjectContext;
use crate::project_paths::NormalizedDocumentPath;
use crate::registry::{AdapterRecord, AdapterRegistry};

#[derive(Clone, Debug)]
pub struct AdapterSelection {
    pub record: AdapterRecord,
    pub manifest: Manifest,
    pub probe: ProbeResult,
    pub evidence: Vec<CandidateEvidence>,
    pub warnings: Vec<AdapterSelectionWarning>,
}

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

enum CandidateResult {
    Selected(Box<SelectedCandidate>),
    Continue(CandidateEvidence),
}

struct ExtensionInference {
    adapter_id: Option<String>,
    evidence: Vec<CandidateEvidence>,
}

#[derive(Clone, Debug)]
struct SelectedCandidate {
    record: AdapterRecord,
    manifest: Manifest,
    probe: ProbeResult,
}

pub fn select_adapter(
    project: &ProjectContext,
    registry: &AdapterRegistry,
    document: &NormalizedDocumentPath,
    operation: Operation,
    preselected_adapter_id: Option<&str>,
    preselected_source: &str,
) -> AppResult<AdapterSelection> {
    let mut evidence = Vec::new();
    let mut warnings = Vec::new();
    let mut attempted = HashSet::new();

    let preselected = match preselected_adapter_id {
        Some(adapter_id) => Some(adapter_id.to_owned()),
        None => {
            let inference = infer_adapter_by_extension(project, registry, document, operation);
            for candidate in inference.evidence {
                attempted.insert(candidate.adapter_id.clone());
                warnings.push(AdapterSelectionWarning::candidate_failure(
                    &candidate, false,
                ));
                evidence.push(candidate);
            }
            inference.adapter_id
        }
    };

    if let Some(adapter_id) = preselected {
        attempted.insert(adapter_id.clone());
        match evaluate_preselected(project, registry, &adapter_id, document, operation) {
            CandidateResult::Selected(selected) => {
                let selected = *selected;
                return Ok(AdapterSelection {
                    record: selected.record,
                    manifest: selected.manifest,
                    probe: selected.probe,
                    evidence,
                    warnings,
                });
            }
            CandidateResult::Continue(candidate) => {
                warnings.push(AdapterSelectionWarning::candidate_failure(
                    &candidate,
                    preselected_source == "explicit",
                ));
                evidence.push(candidate);
            }
        }
    }

    for record in &registry.adapters {
        if attempted.contains(&record.id) {
            continue;
        }
        match evaluate_registry_candidate(project, record, document, operation) {
            CandidateResult::Selected(selected) => {
                let selected = *selected;
                return Ok(AdapterSelection {
                    record: selected.record,
                    manifest: selected.manifest,
                    probe: selected.probe,
                    evidence,
                    warnings,
                });
            }
            CandidateResult::Continue(candidate) => {
                warnings.push(AdapterSelectionWarning::candidate_failure(
                    &candidate, false,
                ));
                evidence.push(candidate);
            }
        }
    }

    Err(StableError::format_unknown(
        document.adapter_path.clone(),
        "no registered adapter supports the document and operation",
        serde_json::to_value(&evidence).unwrap_or_else(|_| Value::Array(Vec::new())),
    )
    .into())
}

fn infer_adapter_by_extension(
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
        let manifest = match run_manifest(&project.project_root, record) {
            Ok(output) => match manifest_from_output(&record.id, output) {
                Ok(manifest) => manifest,
                Err(reason) => {
                    evidence.push(CandidateEvidence::resolve(
                        &record.id,
                        "MANIFEST_INVALID",
                        reason,
                        json!({}),
                    ));
                    continue;
                }
            },
            Err(error) => {
                evidence.push(CandidateEvidence::resolve(
                    &record.id,
                    "ADAPTER_UNAVAILABLE",
                    error.reason.clone(),
                    process_error_details(&error),
                ));
                continue;
            }
        };
        let matches_extension = manifest.formats.iter().any(|format| {
            format
                .extensions
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(&extension))
        });
        if !matches_extension {
            continue;
        }
        if let Err(reason) = ensure_capability(&manifest, operation) {
            evidence.push(CandidateEvidence::resolve(
                &record.id,
                "CAPABILITY_UNSUPPORTED",
                reason,
                json!({ "capability": operation, "adapter_id": record.id }),
            ));
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

fn evaluate_preselected(
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

fn evaluate_registry_candidate(
    project: &ProjectContext,
    record: &AdapterRecord,
    document: &NormalizedDocumentPath,
    operation: Operation,
) -> CandidateResult {
    evaluate_candidate(project, record, document, operation)
}

fn evaluate_candidate(
    project: &ProjectContext,
    record: &AdapterRecord,
    document: &NormalizedDocumentPath,
    operation: Operation,
) -> CandidateResult {
    let manifest = match run_manifest(&project.project_root, record) {
        Ok(output) => match manifest_from_output(&record.id, output) {
            Ok(manifest) => manifest,
            Err(reason) => {
                return CandidateResult::Continue(CandidateEvidence::resolve(
                    &record.id,
                    "MANIFEST_INVALID",
                    reason,
                    json!({}),
                ));
            }
        },
        Err(error) => {
            return CandidateResult::Continue(CandidateEvidence::resolve(
                &record.id,
                "ADAPTER_UNAVAILABLE",
                error.reason.clone(),
                process_error_details(&error),
            ));
        }
    };

    if let Err(reason) = ensure_capability(&manifest, operation) {
        return CandidateResult::Continue(CandidateEvidence::resolve(
            &record.id,
            "CAPABILITY_UNSUPPORTED",
            reason,
            json!({ "capability": operation, "adapter_id": record.id }),
        ));
    }

    let probe = match run_probe(&project.project_root, record, &document.adapter_path) {
        Ok(output) => match probe_from_output(&record.id, &document.adapter_path, output) {
            Ok(probe) => probe,
            Err(reason) => {
                return CandidateResult::Continue(CandidateEvidence::probe(
                    &record.id,
                    "PROBE_INVALID",
                    reason,
                    json!({}),
                ));
            }
        },
        Err(error) => {
            return CandidateResult::Continue(CandidateEvidence::probe(
                &record.id,
                "ADAPTER_UNAVAILABLE",
                error.reason.clone(),
                process_error_details(&error),
            ));
        }
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

fn document_extension(path: &str) -> Option<String> {
    let basename = path.rsplit('/').next().unwrap_or(path);
    let dot = basename.rfind('.')?;
    if dot == 0 {
        return None;
    }
    Some(basename[dot..].to_owned())
}

impl CandidateEvidence {
    fn resolve(adapter_id: &str, code: &str, reason: impl Into<String>, details: Value) -> Self {
        Self {
            adapter_id: adapter_id.to_owned(),
            stage: CandidateStage::Resolve,
            code: code.to_owned(),
            reason: reason.into(),
            details,
        }
    }

    fn probe(adapter_id: &str, code: &str, reason: impl Into<String>, details: Value) -> Self {
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
    fn candidate_failure(candidate: &CandidateEvidence, preselected: bool) -> Self {
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
