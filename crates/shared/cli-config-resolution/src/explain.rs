use crate::diagnostics::ResolutionDiagnostic;
use crate::field::FieldIdentity;
use crate::resolution::{CandidateTrace, CandidateTraceState, ResolutionResult};
use crate::source::{SourceId, SourceKind, SourceLocator};
use crate::value::Value;

impl ResolutionResult {
    pub fn explain(&self) -> ResolutionExplanation {
        let fields = self
            .fields()
            .iter()
            .map(|(field, resolution)| {
                let trace = resolution.trace();
                FieldExplanation {
                    field: field.clone(),
                    value: resolution.value().cloned(),
                    selected: trace.selected.as_ref().map(CandidateExplanation::from),
                    overridden: trace
                        .overridden
                        .iter()
                        .map(CandidateExplanation::from)
                        .collect(),
                    merge_contributors: trace
                        .merge_contributors
                        .iter()
                        .map(CandidateExplanation::from)
                        .collect(),
                    default_fallback: trace
                        .default_fallback
                        .as_ref()
                        .map(CandidateExplanation::from),
                    invalid_candidates: trace
                        .invalid_candidates
                        .iter()
                        .map(CandidateExplanation::from)
                        .collect(),
                    missing_required: trace.missing_required,
                }
            })
            .collect();
        ResolutionExplanation {
            fields,
            diagnostics: self.diagnostics().to_vec(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolutionExplanation {
    fields: Vec<FieldExplanation>,
    diagnostics: Vec<ResolutionDiagnostic>,
}

impl ResolutionExplanation {
    pub fn fields(&self) -> &[FieldExplanation] {
        &self.fields
    }

    pub fn diagnostics(&self) -> &[ResolutionDiagnostic] {
        &self.diagnostics
    }

    pub fn lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        for field in &self.fields {
            if let Some(selected) = field.selected() {
                lines.push(candidate_line(field.field(), "selected", selected));
            } else {
                lines.push(format!("field={} unresolved", field.field().as_str()));
            }
            if let Some(default_fallback) = field.default_fallback() {
                lines.push(candidate_line(
                    field.field(),
                    "default-fallback",
                    default_fallback,
                ));
            }
            for overridden in field.overridden() {
                lines.push(candidate_line(field.field(), "overridden", overridden));
            }
            for contributor in field.merge_contributors() {
                lines.push(candidate_line(
                    field.field(),
                    "merge-contributor",
                    contributor,
                ));
            }
            for invalid in field.invalid_candidates() {
                lines.push(candidate_line(field.field(), "invalid", invalid));
            }
            if field.missing_required() {
                lines.push(format!(
                    "field={} missing-required=true",
                    field.field().as_str()
                ));
            }
        }
        for diagnostic in &self.diagnostics {
            lines.push(format!(
                "diagnostic field={} source={} locator={} received={} reason={:?}",
                diagnostic.field.as_str(),
                diagnostic
                    .source_id
                    .as_ref()
                    .map(SourceId::as_str)
                    .unwrap_or("<none>"),
                diagnostic
                    .locator
                    .as_ref()
                    .map(SourceLocator::as_key)
                    .unwrap_or_else(|| "<none>".to_owned()),
                diagnostic
                    .received_kind
                    .map(|kind| format!("{kind:?}"))
                    .unwrap_or_else(|| "<none>".to_owned()),
                diagnostic.reason
            ));
        }
        lines
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldExplanation {
    field: FieldIdentity,
    value: Option<Value>,
    selected: Option<CandidateExplanation>,
    overridden: Vec<CandidateExplanation>,
    merge_contributors: Vec<CandidateExplanation>,
    default_fallback: Option<CandidateExplanation>,
    invalid_candidates: Vec<CandidateExplanation>,
    missing_required: bool,
}

impl FieldExplanation {
    pub fn field(&self) -> &FieldIdentity {
        &self.field
    }

    pub fn value(&self) -> Option<&Value> {
        self.value.as_ref()
    }

    pub fn selected(&self) -> Option<&CandidateExplanation> {
        self.selected.as_ref()
    }

    pub fn overridden(&self) -> &[CandidateExplanation] {
        &self.overridden
    }

    pub fn merge_contributors(&self) -> &[CandidateExplanation] {
        &self.merge_contributors
    }

    pub fn default_fallback(&self) -> Option<&CandidateExplanation> {
        self.default_fallback.as_ref()
    }

    pub fn invalid_candidates(&self) -> &[CandidateExplanation] {
        &self.invalid_candidates
    }

    pub fn missing_required(&self) -> bool {
        self.missing_required
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CandidateExplanation {
    source_id: SourceId,
    source_kind: SourceKind,
    locator: SourceLocator,
    value: Option<Value>,
    state: CandidateTraceState,
}

impl CandidateExplanation {
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    pub fn source_kind(&self) -> &SourceKind {
        &self.source_kind
    }

    pub fn locator(&self) -> &SourceLocator {
        &self.locator
    }

    pub fn value(&self) -> Option<&Value> {
        self.value.as_ref()
    }

    pub fn state(&self) -> &CandidateTraceState {
        &self.state
    }
}

impl From<&CandidateTrace> for CandidateExplanation {
    fn from(trace: &CandidateTrace) -> Self {
        Self {
            source_id: trace.source_id.clone(),
            source_kind: trace.source_kind.clone(),
            locator: trace.locator.clone(),
            value: trace.value.clone(),
            state: trace.state.clone(),
        }
    }
}

fn candidate_line(field: &FieldIdentity, role: &str, candidate: &CandidateExplanation) -> String {
    format!(
        "field={} {} source={} kind={} locator={} state={} value={}",
        field.as_str(),
        role,
        candidate.source_id().as_str(),
        source_kind_label(candidate.source_kind()),
        candidate.locator().as_key(),
        candidate_state_label(candidate.state()),
        candidate
            .value()
            .map(|value| format!("{value:?}"))
            .unwrap_or_else(|| "<none>".to_owned())
    )
}

fn source_kind_label(kind: &SourceKind) -> String {
    match kind {
        SourceKind::Cli => "cli".to_owned(),
        SourceKind::Env => "env".to_owned(),
        SourceKind::Config => "config".to_owned(),
        SourceKind::Default => "default".to_owned(),
        SourceKind::Custom(name) => format!("custom({name})"),
    }
}

fn candidate_state_label(state: &CandidateTraceState) -> &'static str {
    match state {
        CandidateTraceState::Present => "present",
        CandidateTraceState::Invalid => "invalid",
        CandidateTraceState::ExplicitAbsent => "explicit-absent",
        CandidateTraceState::DefaultFallback => "default-fallback",
    }
}
