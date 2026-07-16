use docnav_typed_fields::{DefaultMetadata, FieldDef, MergeStrategy, SchemaMetadataView};

use crate::diagnostics::{DiagnosticReason, ResolutionDiagnostic};
use crate::source::SourceKind;

use super::candidate::{
    high_precedence, trace_candidate, trace_merge_candidate, EffectiveCandidate,
};
use super::strategy::resolve_value;
use super::{FieldResolution, FieldTrace};

pub(super) fn resolve_field(
    field: &FieldDef,
    metadata: &SchemaMetadataView,
    candidates: Vec<EffectiveCandidate>,
    diagnostics: &mut Vec<ResolutionDiagnostic>,
) -> FieldResolution {
    let mut trace = FieldTrace::new(metadata.identity.clone());
    let (explicit, mut defaults) = partition_candidates(candidates);
    add_static_default(metadata, &mut defaults);
    defaults.sort_by(high_precedence);
    record_default_fallback(field, &defaults, &mut trace);

    let active = if explicit.is_empty() {
        defaults
    } else {
        explicit
    };
    if active.is_empty() {
        return resolve_missing(metadata, trace, diagnostics);
    }

    let value = resolve_value(field, active, &mut trace, diagnostics);
    FieldResolution { value, trace }
}

fn partition_candidates(
    candidates: Vec<EffectiveCandidate>,
) -> (Vec<EffectiveCandidate>, Vec<EffectiveCandidate>) {
    candidates
        .into_iter()
        .partition(|candidate| candidate.source_kind != SourceKind::Default)
}

fn add_static_default(metadata: &SchemaMetadataView, defaults: &mut Vec<EffectiveCandidate>) {
    if let DefaultMetadata::Static(value) = &metadata.default {
        defaults.push(EffectiveCandidate::static_default(
            &metadata.identity,
            value.clone(),
        ));
    }
}

fn record_default_fallback(
    field: &FieldDef,
    defaults: &[EffectiveCandidate],
    trace: &mut FieldTrace,
) {
    let Some(default) = defaults.first() else {
        return;
    };
    let default_trace = match field.merge_strategy() {
        MergeStrategy::Replace => trace_candidate(field, default),
        MergeStrategy::Append | MergeStrategy::MapMerge | MergeStrategy::DenyConflict => {
            trace_merge_candidate(default)
        }
    };
    if default_trace.invalid_reason.is_some() {
        trace.invalid_candidates.push(default_trace.clone());
    }
    trace.default_fallback = Some(default_trace);
}

fn resolve_missing(
    metadata: &SchemaMetadataView,
    mut trace: FieldTrace,
    diagnostics: &mut Vec<ResolutionDiagnostic>,
) -> FieldResolution {
    if let Err(failure) = metadata.validate_optional_value(None) {
        trace.missing_required = true;
        diagnostics.push(ResolutionDiagnostic {
            field: metadata.identity.clone(),
            source_id: None,
            source_kind: None,
            locator: None,
            raw: None,
            reason: DiagnosticReason::MissingRequired(failure),
        });
    }
    FieldResolution { value: None, trace }
}
