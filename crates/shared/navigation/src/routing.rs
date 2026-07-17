use docnav_adapter_contracts::AdapterDefinition;
use docnav_diagnostics::{
    typed_codes, AdapterReasonDetails, DiagnosticSource, FormatCandidateDetails,
    FormatUnknownDetails,
};
use docnav_protocol::{protocol_error_record_draft, ProbeResult};
use serde_json::{json, Value};

use crate::NavigationError;

pub trait NavigationAdapterRegistry {
    fn adapters(&self) -> Vec<AdapterDefinition<'_>>;

    fn find_adapter(&self, adapter_id: &str) -> Option<AdapterDefinition<'_>> {
        self.adapters()
            .into_iter()
            .find(|adapter| adapter.id() == adapter_id)
    }
}

#[derive(Clone, Debug)]
pub struct AdapterSelection<'a> {
    pub adapter: AdapterDefinition<'a>,
    pub evidence: Vec<CandidateEvidence>,
}

#[derive(Clone, Copy)]
pub struct AdapterSelectionRequest<'a, R>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    pub registry: &'a R,
    pub document_path: &'a str,
    pub preselected_adapter_id: Option<&'a str>,
    pub preselected_adapter_source: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateEvidence {
    pub adapter_id: String,
    pub stage: CandidateStage,
    pub code: String,
    pub reason: String,
    pub details: Value,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateStage {
    Resolve,
    Probe,
}

pub fn select_adapter<'a, R>(
    request: AdapterSelectionRequest<'a, R>,
) -> Result<AdapterSelection<'a>, NavigationError>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    let AdapterSelectionRequest {
        registry,
        document_path,
        preselected_adapter_id,
        preselected_adapter_source,
    } = request;
    let mut state = SelectionState::default();

    if let Some(adapter_id) = preselected_adapter_id {
        state.mark_attempted(adapter_id);
        match evaluate_preselected(registry, adapter_id, document_path) {
            CandidateResult::Selected(selected) => {
                return Ok(state.into_selection(selected));
            }
            CandidateResult::Continue(candidate) => {
                return Err(explicit_adapter_error(
                    &candidate,
                    preselected_adapter_source,
                ));
            }
        }
    }

    for adapter in registry.adapters() {
        if state.has_attempted(adapter.id()) {
            continue;
        }
        match evaluate_candidate(adapter, document_path) {
            CandidateResult::Selected(selected) => {
                return Ok(state.into_selection(selected));
            }
            CandidateResult::Continue(candidate) => {
                state.record_failure(candidate);
            }
        }
    }

    Err(state.format_unknown(document_path))
}

enum CandidateResult<'a> {
    Selected(AdapterDefinition<'a>),
    Continue(CandidateEvidence),
}

fn evaluate_preselected<'a, R>(
    registry: &'a R,
    adapter_id: &str,
    document_path: &str,
) -> CandidateResult<'a>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    let Some(adapter) = registry.find_adapter(adapter_id) else {
        return CandidateResult::Continue(CandidateEvidence::resolve(
            adapter_id,
            "ADAPTER_NOT_FOUND",
            "adapter id is not present in the core release static registry",
            json!({}),
        ));
    };

    evaluate_candidate(adapter, document_path)
}

fn evaluate_candidate<'a>(
    adapter: AdapterDefinition<'a>,
    document_path: &str,
) -> CandidateResult<'a> {
    let probe = adapter.probe(document_path);
    if let Err(candidate) = probe_is_valid(&adapter, document_path, &probe) {
        return CandidateResult::Continue(candidate);
    }

    if !probe.supported {
        return CandidateResult::Continue(CandidateEvidence::probe(
            adapter.id(),
            "PROBE_UNSUPPORTED",
            "adapter probe returned supported=false",
            json!({ "probe": probe }),
        ));
    }

    CandidateResult::Selected(adapter)
}

fn probe_is_valid(
    adapter: &AdapterDefinition<'_>,
    document_path: &str,
    probe: &ProbeResult,
) -> Result<(), CandidateEvidence> {
    if probe.adapter_id != adapter.id() {
        return Err(CandidateEvidence::probe(
            adapter.id(),
            "PROBE_INVALID",
            format!(
                "probe adapter_id {:?} does not match registry id {:?}",
                probe.adapter_id,
                adapter.id()
            ),
            json!({}),
        ));
    }
    if probe.path != document_path {
        return Err(CandidateEvidence::probe(
            adapter.id(),
            "PROBE_INVALID",
            "probe path does not match requested document path",
            json!({}),
        ));
    }
    if let Err(error) = probe.validate_semantics() {
        return Err(CandidateEvidence::probe(
            adapter.id(),
            "PROBE_INVALID",
            error.to_string(),
            json!({}),
        ));
    }
    Ok(())
}

fn explicit_adapter_error(
    candidate: &CandidateEvidence,
    selection_source: &str,
) -> NavigationError {
    NavigationError::new(protocol_error_record_draft::<
        typed_codes::protocol::AdapterUnavailable,
    >(
        AdapterReasonDetails::new(candidate.adapter_id.clone(), candidate.reason.clone())
            .with_selection_context(selection_source, candidate.stage.as_str()),
        DiagnosticSource::with_stage("docnav-navigation", "routing"),
    ))
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

    fn into_format_candidate_details(self) -> FormatCandidateDetails {
        FormatCandidateDetails::new(self.adapter_id, self.stage.as_str(), self.code)
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

#[derive(Default)]
struct SelectionState {
    evidence: Vec<CandidateEvidence>,
    attempted: std::collections::HashSet<String>,
}

impl SelectionState {
    fn record_failure(&mut self, candidate: CandidateEvidence) {
        self.evidence.push(candidate);
    }

    fn mark_attempted(&mut self, adapter_id: &str) {
        self.attempted.insert(adapter_id.to_owned());
    }

    fn has_attempted(&self, adapter_id: &str) -> bool {
        self.attempted.contains(adapter_id)
    }

    fn into_selection<'a>(self, selected: AdapterDefinition<'a>) -> AdapterSelection<'a> {
        AdapterSelection {
            adapter: selected,
            evidence: self.evidence,
        }
    }

    fn format_unknown(self, document_path: &str) -> NavigationError {
        let candidates: Vec<FormatCandidateDetails> = self
            .evidence
            .into_iter()
            .map(CandidateEvidence::into_format_candidate_details)
            .collect();
        NavigationError::new(protocol_error_record_draft::<
            typed_codes::protocol::FormatUnknown,
        >(
            FormatUnknownDetails::new(
                document_path.to_owned(),
                "NO_SUPPORTED_ADAPTER",
                candidates.clone(),
            )
            .with_candidate_failures(candidates),
            DiagnosticSource::with_stage("docnav-navigation", "routing"),
        ))
    }
}
