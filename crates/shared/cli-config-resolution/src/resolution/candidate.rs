use std::cmp::Ordering;

use docnav_typed_fields::{FieldDef, FieldIdentity, JsonValue};

use crate::diagnostics::{CandidateInvalidReason, CandidateInvalidReason::Decode};
use crate::source::{CandidateInput, Source, SourceCandidate, SourceId, SourceKind, SourceLocator};

use super::CandidateTrace;

#[derive(Clone, Debug)]
pub(super) struct EffectiveCandidate {
    pub(super) source_id: SourceId,
    pub(super) source_kind: SourceKind,
    priority: i32,
    source_order: usize,
    pub(super) locator: SourceLocator,
    pub(super) input: CandidateInput,
    automatic_default: bool,
}

impl EffectiveCandidate {
    pub(super) fn from_source(
        source: &Source,
        source_order: usize,
        candidate: &SourceCandidate,
    ) -> Self {
        Self {
            source_id: source.id().clone(),
            source_kind: source.kind().clone(),
            priority: source.priority(),
            source_order,
            locator: candidate.locator().clone(),
            input: candidate.input().clone(),
            automatic_default: false,
        }
    }

    pub(super) fn static_default(field: &FieldIdentity, value: JsonValue) -> Self {
        Self {
            source_id: SourceId::static_default(),
            source_kind: SourceKind::Default,
            priority: i32::MIN,
            source_order: 0,
            locator: SourceLocator::Default(field.as_str().to_owned()),
            input: CandidateInput::Value(value),
            automatic_default: true,
        }
    }

    pub(super) fn raw(&self) -> &JsonValue {
        self.input.raw()
    }

    fn rank(&self) -> (i32, u8, usize) {
        (
            self.priority,
            u8::from(!self.automatic_default),
            self.source_order,
        )
    }
}

pub(super) fn trace_candidate(field: &FieldDef, candidate: &EffectiveCandidate) -> CandidateTrace {
    CandidateTrace {
        source_id: candidate.source_id.clone(),
        source_kind: candidate.source_kind.clone(),
        locator: candidate.locator.clone(),
        raw: candidate.raw().clone(),
        invalid_reason: candidate_invalid_reason(field, candidate),
    }
}

pub(super) fn trace_merge_candidate(candidate: &EffectiveCandidate) -> CandidateTrace {
    CandidateTrace {
        source_id: candidate.source_id.clone(),
        source_kind: candidate.source_kind.clone(),
        locator: candidate.locator.clone(),
        raw: candidate.raw().clone(),
        invalid_reason: match &candidate.input {
            CandidateInput::Invalid { reason, .. } => Some(Decode(reason.clone())),
            CandidateInput::Value(_) => None,
        },
    }
}

fn candidate_invalid_reason(
    field: &FieldDef,
    candidate: &EffectiveCandidate,
) -> Option<CandidateInvalidReason> {
    match &candidate.input {
        CandidateInput::Invalid { reason, .. } => Some(Decode(reason.clone())),
        CandidateInput::Value(value) => field
            .validate_value(value)
            .err()
            .map(CandidateInvalidReason::Validation),
    }
}

pub(super) fn high_precedence(left: &EffectiveCandidate, right: &EffectiveCandidate) -> Ordering {
    right.rank().cmp(&left.rank())
}

pub(super) fn low_precedence(left: &EffectiveCandidate, right: &EffectiveCandidate) -> Ordering {
    left.rank().cmp(&right.rank())
}
