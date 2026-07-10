use std::collections::BTreeMap;

use crate::diagnostics::{DiagnosticReason, MergeConflictReason, ResolutionDiagnostic};
use crate::field::{FieldContract, FieldIdentity, FieldSet, ValidationReason};
use crate::source::{
    CandidateState, SourceCandidate, SourceCollection, SourceId, SourceKind, SourceLocator,
};
use crate::value::{ReceivedValueKind, Value, ValueMap};

pub type ResolvedValueMap = BTreeMap<FieldIdentity, Value>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MergeStrategy {
    ScalarReplace,
    ListAppend,
    ListReplace,
    MapMerge,
    MapReplace,
    DenyConflict,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolutionResult {
    fields: BTreeMap<FieldIdentity, FieldResolution>,
    diagnostics: Vec<ResolutionDiagnostic>,
}

impl ResolutionResult {
    pub fn fields(&self) -> &BTreeMap<FieldIdentity, FieldResolution> {
        &self.fields
    }

    pub fn diagnostics(&self) -> &[ResolutionDiagnostic] {
        &self.diagnostics
    }

    pub fn materialize(&self) -> Result<ResolvedValueMap, MaterializationError> {
        if !self.diagnostics.is_empty() {
            return Err(MaterializationError {
                diagnostics: self.diagnostics.clone(),
            });
        }
        Ok(self
            .fields
            .iter()
            .filter_map(|(identity, resolved)| {
                resolved
                    .value()
                    .cloned()
                    .map(|value| (identity.clone(), value))
            })
            .collect())
    }

    pub fn try_materialize_with<F, T>(&self, mapper: F) -> Result<T, MaterializationError>
    where
        F: FnOnce(&ResolvedValueMap) -> T,
    {
        let values = self.materialize()?;
        Ok(mapper(&values))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MaterializationError {
    diagnostics: Vec<ResolutionDiagnostic>,
}

impl MaterializationError {
    pub fn diagnostics(&self) -> &[ResolutionDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldResolution {
    value: Option<Value>,
    trace: FieldTrace,
}

impl FieldResolution {
    pub fn value(&self) -> Option<&Value> {
        self.value.as_ref()
    }

    pub fn trace(&self) -> &FieldTrace {
        &self.trace
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldTrace {
    pub field: FieldIdentity,
    pub selected: Option<CandidateTrace>,
    pub overridden: Vec<CandidateTrace>,
    pub merge_contributors: Vec<CandidateTrace>,
    pub default_fallback: Option<CandidateTrace>,
    pub invalid_candidates: Vec<CandidateTrace>,
    pub missing_required: bool,
}

impl FieldTrace {
    fn new(field: FieldIdentity) -> Self {
        Self {
            field,
            selected: None,
            overridden: Vec::new(),
            merge_contributors: Vec::new(),
            default_fallback: None,
            invalid_candidates: Vec::new(),
            missing_required: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CandidateTrace {
    pub source_id: SourceId,
    pub source_kind: SourceKind,
    pub locator: SourceLocator,
    pub value: Option<Value>,
    pub state: CandidateTraceState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CandidateTraceState {
    Present,
    Invalid,
    ExplicitAbsent,
    DefaultFallback,
}

pub struct Resolver;

impl Resolver {
    pub fn resolve(
        fields: &FieldSet,
        sources: &SourceCollection,
        candidates: Vec<SourceCandidate>,
    ) -> ResolutionResult {
        let mut diagnostics = Vec::new();
        let mut by_field = BTreeMap::<FieldIdentity, Vec<RankedCandidate>>::new();
        for candidate in candidates {
            if let Some((source_order, source)) = sources
                .ordered_applicable()
                .into_iter()
                .find(|(_, spec)| spec.id() == candidate.source_id())
            {
                by_field
                    .entry(candidate.field().clone())
                    .or_default()
                    .push(RankedCandidate {
                        priority: source.priority(),
                        source_order,
                        candidate,
                    });
            }
        }

        let mut results = BTreeMap::new();
        for field in fields.fields() {
            let mut field_candidates = by_field.remove(field.identity()).unwrap_or_default();
            field_candidates.sort_by(|left, right| {
                right
                    .priority
                    .cmp(&left.priority)
                    .then_with(|| left.source_order.cmp(&right.source_order))
            });
            let resolution = resolve_field(field, field_candidates, &mut diagnostics);
            results.insert(field.identity().clone(), resolution);
        }
        ResolutionResult {
            fields: results,
            diagnostics,
        }
    }
}

#[derive(Clone, Debug)]
struct RankedCandidate {
    priority: i32,
    source_order: usize,
    candidate: SourceCandidate,
}

#[derive(Clone, Debug)]
struct ValidCandidate {
    priority: i32,
    source_order: usize,
    candidate: SourceCandidate,
    value: Value,
}

fn resolve_field(
    field: &FieldContract,
    candidates: Vec<RankedCandidate>,
    diagnostics: &mut Vec<ResolutionDiagnostic>,
) -> FieldResolution {
    let mut trace = FieldTrace::new(field.identity().clone());
    let mut valid_explicit = Vec::<ValidCandidate>::new();
    let mut default_candidates = Vec::<RankedCandidate>::new();
    let mut explicit_absent = Vec::<RankedCandidate>::new();

    for ranked in candidates {
        match ranked.candidate.state().clone() {
            CandidateState::Missing => {}
            CandidateState::ExplicitAbsent => explicit_absent.push(ranked),
            CandidateState::Invalid { received, reason } => {
                trace.invalid_candidates.push(trace_for_candidate(
                    &ranked.candidate,
                    received.clone(),
                    CandidateTraceState::Invalid,
                ));
                if ranked.candidate.source_kind() == &SourceKind::Default {
                    default_candidates.push(ranked);
                } else {
                    diagnostics.push(diagnostic_for_candidate(
                        field.identity(),
                        &ranked.candidate,
                        received.as_ref().map(Value::received_kind),
                        DiagnosticReason::SourceInvalid {
                            reason: reason.clone(),
                        },
                    ));
                }
            }
            CandidateState::Present(value) => {
                if let Err(failure) = field.validate_value(&value) {
                    diagnostics.push(diagnostic_for_candidate(
                        field.identity(),
                        &ranked.candidate,
                        Some(value.received_kind()),
                        DiagnosticReason::ValidationFailed(failure.reason),
                    ));
                    trace.invalid_candidates.push(trace_for_candidate(
                        &ranked.candidate,
                        Some(value.clone()),
                        CandidateTraceState::Invalid,
                    ));
                } else {
                    valid_explicit.push(ValidCandidate {
                        priority: ranked.priority,
                        source_order: ranked.source_order,
                        candidate: ranked.candidate,
                        value: value.clone(),
                    });
                }
            }
            CandidateState::DefaultFallback { value, .. } => {
                if field.validate_value(&value).is_err() {
                    trace.invalid_candidates.push(trace_for_candidate(
                        &ranked.candidate,
                        Some(value),
                        CandidateTraceState::Invalid,
                    ));
                }
                default_candidates.push(ranked);
            }
        }
    }

    valid_explicit.sort_by(source_order);
    default_candidates.sort_by(source_order_ranked);
    explicit_absent.sort_by(source_order_ranked);
    trace.default_fallback = default_candidates
        .first()
        .and_then(default_trace_for_ranked);

    if explicit_absent_wins(&explicit_absent, &valid_explicit) {
        let winner = explicit_absent.remove(0);
        trace.selected = Some(trace_for_candidate(
            &winner.candidate,
            None,
            CandidateTraceState::ExplicitAbsent,
        ));
        if field.constraints().required {
            trace.missing_required = true;
            diagnostics.push(diagnostic_for_candidate(
                field.identity(),
                &winner.candidate,
                None,
                DiagnosticReason::MissingRequired,
            ));
        }
        return FieldResolution { value: None, trace };
    }

    let (value, mut merge_trace) = if valid_explicit.is_empty() {
        resolve_default_or_missing(field, &default_candidates, &mut trace, diagnostics)
    } else {
        match field.merge_strategy() {
            MergeStrategy::ScalarReplace
            | MergeStrategy::ListReplace
            | MergeStrategy::MapReplace => resolve_replace(field, &valid_explicit, diagnostics),
            MergeStrategy::ListAppend => resolve_list_append(&valid_explicit),
            MergeStrategy::MapMerge => resolve_map_merge(&valid_explicit),
            MergeStrategy::DenyConflict => {
                resolve_deny_conflict(field, &valid_explicit, diagnostics)
            }
        }
    };
    apply_merge_trace(&mut trace, &mut merge_trace);
    if value.is_none() && field.constraints().required && !trace.missing_required {
        trace.missing_required = true;
        diagnostics.push(ResolutionDiagnostic {
            field: field.identity().clone(),
            source_id: None,
            locator: None,
            received_kind: None,
            reason: DiagnosticReason::ValidationFailed(ValidationReason::MissingRequired),
        });
    }
    FieldResolution { value, trace }
}

fn resolve_default_or_missing(
    field: &FieldContract,
    default_candidates: &[RankedCandidate],
    trace: &mut FieldTrace,
    diagnostics: &mut Vec<ResolutionDiagnostic>,
) -> (Option<Value>, MergeTrace) {
    if let Some(default) = default_candidates.first() {
        match default.candidate.state() {
            CandidateState::DefaultFallback { value, .. } => {
                if let Err(failure) = field.validate_value(value) {
                    diagnostics.push(diagnostic_for_candidate(
                        field.identity(),
                        &default.candidate,
                        Some(value.received_kind()),
                        DiagnosticReason::ValidationFailed(failure.reason),
                    ));
                    return (None, MergeTrace::default());
                }
                let candidate_trace = trace_for_candidate(
                    &default.candidate,
                    Some(value.clone()),
                    CandidateTraceState::DefaultFallback,
                );
                trace.selected = Some(candidate_trace.clone());
                return (Some(value.clone()), MergeTrace::selected(candidate_trace));
            }
            CandidateState::Invalid { received, reason } => {
                diagnostics.push(diagnostic_for_candidate(
                    field.identity(),
                    &default.candidate,
                    received.as_ref().map(Value::received_kind),
                    DiagnosticReason::SourceInvalid {
                        reason: reason.clone(),
                    },
                ));
                return (None, MergeTrace::default());
            }
            CandidateState::ExplicitAbsent => {
                trace.selected = Some(trace_for_candidate(
                    &default.candidate,
                    None,
                    CandidateTraceState::ExplicitAbsent,
                ));
                return (None, MergeTrace::default());
            }
            CandidateState::Missing | CandidateState::Present(_) => {}
        }
    }
    (None, MergeTrace::default())
}

fn resolve_replace(
    field: &FieldContract,
    valid: &[ValidCandidate],
    diagnostics: &mut Vec<ResolutionDiagnostic>,
) -> (Option<Value>, MergeTrace) {
    let Some(winner) = valid.first() else {
        return (None, MergeTrace::default());
    };
    let same_priority = valid
        .iter()
        .take_while(|candidate| candidate.priority == winner.priority)
        .collect::<Vec<_>>();
    if same_priority.len() > 1
        && same_priority
            .iter()
            .any(|candidate| candidate.value != winner.value)
    {
        for candidate in &same_priority {
            diagnostics.push(diagnostic_for_candidate(
                field.identity(),
                &candidate.candidate,
                Some(candidate.value.received_kind()),
                DiagnosticReason::MergeConflict(MergeConflictReason::SamePriorityReplace),
            ));
            diagnostics.push(diagnostic_for_candidate(
                field.identity(),
                &candidate.candidate,
                Some(candidate.value.received_kind()),
                DiagnosticReason::AmbiguousPriority {
                    priority: winner.priority,
                },
            ));
        }
        return (
            None,
            MergeTrace::invalid(
                same_priority
                    .iter()
                    .map(|candidate| trace_for_valid(candidate, CandidateTraceState::Invalid))
                    .collect(),
            ),
        );
    }
    let selected = trace_for_valid(winner, CandidateTraceState::Present);
    let overridden = valid
        .iter()
        .skip(1)
        .map(|candidate| trace_for_valid(candidate, CandidateTraceState::Present))
        .collect();
    (
        Some(winner.value.clone()),
        MergeTrace {
            selected: Some(selected),
            overridden,
            merge_contributors: Vec::new(),
            invalid_candidates: Vec::new(),
        },
    )
}

fn resolve_list_append(valid: &[ValidCandidate]) -> (Option<Value>, MergeTrace) {
    let merge_len = valid
        .iter()
        .position(|candidate| candidate.value == Value::Null)
        .unwrap_or(valid.len());
    if merge_len == 0 {
        let selected = trace_for_valid(&valid[0], CandidateTraceState::Present);
        let overridden = valid
            .iter()
            .skip(1)
            .map(|candidate| trace_for_valid(candidate, CandidateTraceState::Present))
            .collect();
        return (
            Some(Value::Null),
            MergeTrace {
                selected: Some(selected),
                overridden,
                merge_contributors: Vec::new(),
                invalid_candidates: Vec::new(),
            },
        );
    }

    let mut items = Vec::new();
    for candidate in &valid[..merge_len] {
        let Value::List(values) = &candidate.value else {
            unreachable!("list merge strategy received a non-list candidate");
        };
        items.extend(values.clone());
    }
    let contributors = valid[..merge_len]
        .iter()
        .map(|candidate| trace_for_valid(candidate, CandidateTraceState::Present))
        .collect::<Vec<_>>();
    let selected = contributors.first().cloned();
    let overridden = valid[merge_len..]
        .iter()
        .map(|candidate| trace_for_valid(candidate, CandidateTraceState::Present))
        .collect();
    (
        Some(Value::List(items)),
        MergeTrace {
            selected,
            overridden,
            merge_contributors: contributors,
            invalid_candidates: Vec::new(),
        },
    )
}

fn resolve_map_merge(valid: &[ValidCandidate]) -> (Option<Value>, MergeTrace) {
    let merge_len = valid
        .iter()
        .position(|candidate| candidate.value == Value::Null)
        .unwrap_or(valid.len());
    if merge_len == 0 {
        let selected = trace_for_valid(&valid[0], CandidateTraceState::Present);
        let overridden = valid
            .iter()
            .skip(1)
            .map(|candidate| trace_for_valid(candidate, CandidateTraceState::Present))
            .collect();
        return (
            Some(Value::Null),
            MergeTrace {
                selected: Some(selected),
                overridden,
                merge_contributors: Vec::new(),
                invalid_candidates: Vec::new(),
            },
        );
    }

    let mut merged = ValueMap::new();
    for candidate in valid[..merge_len].iter().rev() {
        let Value::Map(map) = &candidate.value else {
            unreachable!("map merge strategy received a non-map candidate");
        };
        merged.extend(map.clone());
    }
    let contributors = valid[..merge_len]
        .iter()
        .map(|candidate| trace_for_valid(candidate, CandidateTraceState::Present))
        .collect::<Vec<_>>();
    let selected = contributors.first().cloned();
    let overridden = valid[merge_len..]
        .iter()
        .map(|candidate| trace_for_valid(candidate, CandidateTraceState::Present))
        .collect();
    (
        Some(Value::Map(merged)),
        MergeTrace {
            selected,
            overridden,
            merge_contributors: contributors,
            invalid_candidates: Vec::new(),
        },
    )
}

fn resolve_deny_conflict(
    field: &FieldContract,
    valid: &[ValidCandidate],
    diagnostics: &mut Vec<ResolutionDiagnostic>,
) -> (Option<Value>, MergeTrace) {
    let Some(winner) = valid.first() else {
        return (None, MergeTrace::default());
    };
    if valid
        .iter()
        .any(|candidate| candidate.value != winner.value)
    {
        for candidate in valid {
            diagnostics.push(diagnostic_for_candidate(
                field.identity(),
                &candidate.candidate,
                Some(candidate.value.received_kind()),
                DiagnosticReason::MergeConflict(MergeConflictReason::DenyConflict),
            ));
        }
        return (
            None,
            MergeTrace::invalid(
                valid
                    .iter()
                    .map(|candidate| trace_for_valid(candidate, CandidateTraceState::Invalid))
                    .collect(),
            ),
        );
    }
    let selected = trace_for_valid(winner, CandidateTraceState::Present);
    let overridden = valid
        .iter()
        .skip(1)
        .map(|candidate| trace_for_valid(candidate, CandidateTraceState::Present))
        .collect();
    (
        Some(winner.value.clone()),
        MergeTrace {
            selected: Some(selected),
            overridden,
            merge_contributors: Vec::new(),
            invalid_candidates: Vec::new(),
        },
    )
}

#[derive(Default)]
struct MergeTrace {
    selected: Option<CandidateTrace>,
    overridden: Vec<CandidateTrace>,
    merge_contributors: Vec<CandidateTrace>,
    invalid_candidates: Vec<CandidateTrace>,
}

impl MergeTrace {
    fn selected(selected: CandidateTrace) -> Self {
        Self {
            selected: Some(selected),
            overridden: Vec::new(),
            merge_contributors: Vec::new(),
            invalid_candidates: Vec::new(),
        }
    }

    fn invalid(invalid_candidates: Vec<CandidateTrace>) -> Self {
        Self {
            selected: None,
            overridden: Vec::new(),
            merge_contributors: Vec::new(),
            invalid_candidates,
        }
    }
}

fn apply_merge_trace(trace: &mut FieldTrace, merge_trace: &mut MergeTrace) {
    if trace.selected.is_none() {
        trace.selected = merge_trace.selected.take();
    }
    trace.overridden.append(&mut merge_trace.overridden);
    trace
        .merge_contributors
        .append(&mut merge_trace.merge_contributors);
    trace
        .invalid_candidates
        .append(&mut merge_trace.invalid_candidates);
}

fn explicit_absent_wins(absent: &[RankedCandidate], valid: &[ValidCandidate]) -> bool {
    match (absent.first(), valid.first()) {
        (Some(absent), Some(valid)) => {
            absent.priority > valid.priority
                || (absent.priority == valid.priority && absent.source_order <= valid.source_order)
        }
        (Some(_), None) => true,
        _ => false,
    }
}

fn trace_for_valid(candidate: &ValidCandidate, state: CandidateTraceState) -> CandidateTrace {
    trace_for_candidate(&candidate.candidate, Some(candidate.value.clone()), state)
}

fn default_trace_for_ranked(candidate: &RankedCandidate) -> Option<CandidateTrace> {
    match candidate.candidate.state() {
        CandidateState::DefaultFallback { value, .. } => Some(trace_for_candidate(
            &candidate.candidate,
            Some(value.clone()),
            CandidateTraceState::DefaultFallback,
        )),
        CandidateState::Invalid { received, .. } => Some(trace_for_candidate(
            &candidate.candidate,
            received.clone(),
            CandidateTraceState::Invalid,
        )),
        CandidateState::ExplicitAbsent => Some(trace_for_candidate(
            &candidate.candidate,
            None,
            CandidateTraceState::ExplicitAbsent,
        )),
        CandidateState::Missing | CandidateState::Present(_) => None,
    }
}

fn trace_for_candidate(
    candidate: &SourceCandidate,
    value: Option<Value>,
    state: CandidateTraceState,
) -> CandidateTrace {
    CandidateTrace {
        source_id: candidate.source_id().clone(),
        source_kind: candidate.source_kind().clone(),
        locator: candidate.locator().clone(),
        value,
        state,
    }
}

fn diagnostic_for_candidate(
    field: &FieldIdentity,
    candidate: &SourceCandidate,
    received_kind: Option<ReceivedValueKind>,
    reason: DiagnosticReason,
) -> ResolutionDiagnostic {
    ResolutionDiagnostic {
        field: field.clone(),
        source_id: Some(candidate.source_id().clone()),
        locator: Some(candidate.locator().clone()),
        received_kind,
        reason,
    }
}

fn source_order(left: &ValidCandidate, right: &ValidCandidate) -> std::cmp::Ordering {
    right
        .priority
        .cmp(&left.priority)
        .then_with(|| left.source_order.cmp(&right.source_order))
}

fn source_order_ranked(left: &RankedCandidate, right: &RankedCandidate) -> std::cmp::Ordering {
    right
        .priority
        .cmp(&left.priority)
        .then_with(|| left.source_order.cmp(&right.source_order))
}
