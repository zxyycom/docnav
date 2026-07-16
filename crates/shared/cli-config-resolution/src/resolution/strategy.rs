use docnav_typed_fields::{FieldDef, JsonValue, MergeStrategy, TypedValue, ValueKind};

use crate::diagnostics::{CandidateInvalidReason, DiagnosticReason, ResolutionDiagnostic};

use super::candidate::{
    high_precedence, low_precedence, trace_candidate, trace_merge_candidate, EffectiveCandidate,
};
use super::FieldTrace;

pub(super) fn resolve_value(
    field: &FieldDef,
    candidates: Vec<EffectiveCandidate>,
    trace: &mut FieldTrace,
    diagnostics: &mut Vec<ResolutionDiagnostic>,
) -> Option<TypedValue> {
    match field.merge_strategy() {
        MergeStrategy::Replace => resolve_replace(field, candidates, trace, diagnostics),
        MergeStrategy::Append => resolve_append(field, candidates, trace, diagnostics),
        MergeStrategy::MapMerge => resolve_map_merge(field, candidates, trace, diagnostics),
        MergeStrategy::DenyConflict => resolve_deny_conflict(field, candidates, trace, diagnostics),
    }
}

fn resolve_replace(
    field: &FieldDef,
    mut candidates: Vec<EffectiveCandidate>,
    trace: &mut FieldTrace,
    diagnostics: &mut Vec<ResolutionDiagnostic>,
) -> Option<TypedValue> {
    candidates.sort_by(high_precedence);
    let winner = candidates.first()?;
    let selected = trace_candidate(field, winner);
    trace.selected = Some(selected.clone());
    for overridden in candidates.iter().skip(1) {
        let overridden = trace_candidate(field, overridden);
        if overridden.invalid_reason.is_some() {
            trace.invalid_candidates.push(overridden.clone());
        }
        trace.overridden.push(overridden);
    }
    if let Some(reason) = selected.invalid_reason.clone() {
        trace.invalid_candidates.push(selected);
        diagnostics.push(invalid_diagnostic(field, winner, reason));
        return None;
    }
    final_validate(field, winner.raw(), winner, diagnostics)
}

fn resolve_append(
    field: &FieldDef,
    mut candidates: Vec<EffectiveCandidate>,
    trace: &mut FieldTrace,
    diagnostics: &mut Vec<ResolutionDiagnostic>,
) -> Option<TypedValue> {
    candidates.sort_by(low_precedence);
    let mut merged = Vec::new();
    let mut blocked = false;
    for candidate in &candidates {
        let mut contributor = trace_merge_candidate(candidate);
        if let Some(reason) = contributor.invalid_reason.clone() {
            blocked = true;
            trace.invalid_candidates.push(contributor.clone());
            diagnostics.push(invalid_diagnostic(field, candidate, reason));
        } else if let Some(values) = candidate.raw().as_array() {
            merged.extend(values.clone());
        } else {
            let reason = CandidateInvalidReason::Shape {
                expected: ValueKind::Array,
            };
            blocked = true;
            contributor.invalid_reason = Some(reason.clone());
            trace.invalid_candidates.push(contributor.clone());
            diagnostics.push(invalid_diagnostic(field, candidate, reason));
        }
        trace.contributors.push(contributor);
    }
    finish_collection_merge(
        field,
        &candidates,
        trace,
        diagnostics,
        CollectionMerge {
            blocked,
            value: JsonValue::Array(merged),
        },
    )
}

fn resolve_map_merge(
    field: &FieldDef,
    mut candidates: Vec<EffectiveCandidate>,
    trace: &mut FieldTrace,
    diagnostics: &mut Vec<ResolutionDiagnostic>,
) -> Option<TypedValue> {
    candidates.sort_by(low_precedence);
    let mut merged = serde_json::Map::new();
    let mut blocked = false;
    for candidate in &candidates {
        let mut contributor = trace_merge_candidate(candidate);
        if let Some(reason) = contributor.invalid_reason.clone() {
            blocked = true;
            trace.invalid_candidates.push(contributor.clone());
            diagnostics.push(invalid_diagnostic(field, candidate, reason));
        } else if let Some(values) = candidate.raw().as_object() {
            merged.extend(values.clone());
        } else {
            let reason = CandidateInvalidReason::Shape {
                expected: ValueKind::Object,
            };
            blocked = true;
            contributor.invalid_reason = Some(reason.clone());
            trace.invalid_candidates.push(contributor.clone());
            diagnostics.push(invalid_diagnostic(field, candidate, reason));
        }
        trace.contributors.push(contributor);
    }
    finish_collection_merge(
        field,
        &candidates,
        trace,
        diagnostics,
        CollectionMerge {
            blocked,
            value: JsonValue::Object(merged),
        },
    )
}

struct CollectionMerge {
    blocked: bool,
    value: JsonValue,
}

fn finish_collection_merge(
    field: &FieldDef,
    candidates: &[EffectiveCandidate],
    trace: &mut FieldTrace,
    diagnostics: &mut Vec<ResolutionDiagnostic>,
    merge: CollectionMerge,
) -> Option<TypedValue> {
    trace.selected = trace.contributors.last().cloned();
    if merge.blocked {
        None
    } else {
        final_validate(field, &merge.value, candidates.last()?, diagnostics)
    }
}

fn resolve_deny_conflict(
    field: &FieldDef,
    mut candidates: Vec<EffectiveCandidate>,
    trace: &mut FieldTrace,
    diagnostics: &mut Vec<ResolutionDiagnostic>,
) -> Option<TypedValue> {
    candidates.sort_by(low_precedence);
    let mut blocked = false;
    for candidate in &candidates {
        let contributor = trace_merge_candidate(candidate);
        if let Some(reason) = contributor.invalid_reason.clone() {
            blocked = true;
            trace.invalid_candidates.push(contributor.clone());
            diagnostics.push(invalid_diagnostic(field, candidate, reason));
        }
        trace.contributors.push(contributor);
    }
    trace.selected = trace.contributors.last().cloned();
    if blocked {
        return None;
    }
    let first = candidates.first()?.raw();
    if candidates
        .iter()
        .skip(1)
        .any(|candidate| candidate.raw() != first)
    {
        let conflict = candidates.last().expect("non-empty candidates");
        diagnostics.push(ResolutionDiagnostic {
            field: field.identity().clone(),
            source_id: Some(conflict.source_id.clone()),
            source_kind: Some(conflict.source_kind.clone()),
            locator: Some(conflict.locator.clone()),
            raw: Some(conflict.raw().clone()),
            reason: DiagnosticReason::MergeConflict(
                candidates
                    .iter()
                    .map(|candidate| candidate.locator.clone())
                    .collect(),
            ),
        });
        return None;
    }
    let winner = candidates.last()?;
    final_validate(field, winner.raw(), winner, diagnostics)
}

fn final_validate(
    field: &FieldDef,
    value: &JsonValue,
    selected: &EffectiveCandidate,
    diagnostics: &mut Vec<ResolutionDiagnostic>,
) -> Option<TypedValue> {
    match field.validate_value(value) {
        Ok(value) => Some(value),
        Err(failure) => {
            diagnostics.push(ResolutionDiagnostic {
                field: field.identity().clone(),
                source_id: Some(selected.source_id.clone()),
                source_kind: Some(selected.source_kind.clone()),
                locator: Some(selected.locator.clone()),
                raw: Some(value.clone()),
                reason: DiagnosticReason::FinalValidation(failure),
            });
            None
        }
    }
}

fn invalid_diagnostic(
    field: &FieldDef,
    candidate: &EffectiveCandidate,
    reason: CandidateInvalidReason,
) -> ResolutionDiagnostic {
    ResolutionDiagnostic {
        field: field.identity().clone(),
        source_id: Some(candidate.source_id.clone()),
        source_kind: Some(candidate.source_kind.clone()),
        locator: Some(candidate.locator.clone()),
        raw: Some(candidate.raw().clone()),
        reason: DiagnosticReason::InvalidCandidate(reason),
    }
}
