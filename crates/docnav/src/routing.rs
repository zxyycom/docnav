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

#[derive(Clone, Copy)]
pub struct AdapterSelectionRequest<'a> {
    pub project: &'a ProjectContext,
    pub registry: &'a AdapterRegistry,
    pub document: &'a NormalizedDocumentPath,
    pub operation: Operation,
    pub preselected_adapter_id: Option<&'a str>,
    pub preselected_source: &'a str,
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

#[derive(Default)]
struct SelectionState {
    evidence: Vec<CandidateEvidence>,
    warnings: Vec<AdapterSelectionWarning>,
    attempted: HashSet<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PreselectedSource {
    Explicit,
    Other,
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

fn evaluate_candidate(
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

impl SelectionState {
    fn record_inference_failures(&mut self, candidates: Vec<CandidateEvidence>) {
        for candidate in candidates {
            self.mark_attempted(&candidate.adapter_id);
            self.record_failure(candidate, false);
        }
    }

    fn record_failure(&mut self, candidate: CandidateEvidence, preselected: bool) {
        self.warnings
            .push(AdapterSelectionWarning::candidate_failure(
                &candidate,
                preselected,
            ));
        self.evidence.push(candidate);
    }

    fn mark_attempted(&mut self, adapter_id: &str) {
        self.attempted.insert(adapter_id.to_owned());
    }

    fn has_attempted(&self, adapter_id: &str) -> bool {
        self.attempted.contains(adapter_id)
    }

    fn into_selection(self, selected: SelectedCandidate) -> AdapterSelection {
        AdapterSelection {
            record: selected.record,
            manifest: selected.manifest,
            probe: selected.probe,
            evidence: self.evidence,
            warnings: self.warnings,
        }
    }

    fn format_unknown(self, document: &NormalizedDocumentPath) -> StableError {
        StableError::format_unknown(
            document.adapter_path.clone(),
            "no registered adapter supports the document and operation",
            serde_json::to_value(&self.evidence).unwrap_or_else(|_| Value::Array(Vec::new())),
        )
    }
}

impl PreselectedSource {
    fn from_raw(source: &str) -> Self {
        if source == "explicit" {
            Self::Explicit
        } else {
            Self::Other
        }
    }

    const fn is_explicit(self) -> bool {
        matches!(self, Self::Explicit)
    }
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
