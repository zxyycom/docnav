use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use docnav_typed_fields::{
    FieldDefSet, FieldIdentity, FieldValidationErrors, FieldValueMap, TypedValue, ValidationFailure,
};

use crate::diagnostics::ResolutionDiagnostic;
use crate::source::{Source, SourceId, SourceKind, SourceLocator};

mod candidate;
mod field;
mod strategy;

use candidate::EffectiveCandidate;
use field::resolve_field;

#[derive(Clone, Debug, PartialEq)]
pub struct ResolutionResult {
    fields: BTreeMap<FieldIdentity, FieldResolution>,
    diagnostics: Vec<ResolutionDiagnostic>,
}

impl ResolutionResult {
    pub fn fields(&self) -> &BTreeMap<FieldIdentity, FieldResolution> {
        &self.fields
    }

    pub fn trace(&self, identity: &FieldIdentity) -> Option<&FieldTrace> {
        self.fields.get(identity).map(FieldResolution::trace)
    }

    pub fn diagnostics(&self) -> &[ResolutionDiagnostic] {
        &self.diagnostics
    }

    pub fn materialize(&self) -> Result<FieldValueMap, MaterializationError> {
        if !self.diagnostics.is_empty() {
            return Err(MaterializationError {
                diagnostics: self.diagnostics.clone(),
                validation_failures: Vec::new(),
            });
        }
        Ok(self
            .fields
            .iter()
            .filter_map(|(identity, resolution)| {
                resolution
                    .value
                    .clone()
                    .map(|value| (identity.clone(), value))
            })
            .collect())
    }

    pub fn materialize_with<T, F>(&self, materialize: F) -> Result<T, MaterializationError>
    where
        F: FnOnce(&FieldValueMap) -> Result<T, FieldValidationErrors>,
    {
        let values = self.materialize()?;
        materialize(&values).map_err(|errors| MaterializationError {
            diagnostics: Vec::new(),
            validation_failures: errors.into_failures(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MaterializationError {
    diagnostics: Vec<ResolutionDiagnostic>,
    validation_failures: Vec<ValidationFailure>,
}

impl MaterializationError {
    pub fn diagnostics(&self) -> &[ResolutionDiagnostic] {
        &self.diagnostics
    }

    pub fn validation_failures(&self) -> &[ValidationFailure] {
        &self.validation_failures
    }
}

impl fmt::Display for MaterializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "resolution has {} diagnostic(s) and {} materialization validation failure(s)",
            self.diagnostics.len(),
            self.validation_failures.len()
        )
    }
}

impl std::error::Error for MaterializationError {}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldResolution {
    value: Option<TypedValue>,
    trace: FieldTrace,
}

impl FieldResolution {
    pub fn value(&self) -> Option<&TypedValue> {
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
    pub contributors: Vec<CandidateTrace>,
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
            contributors: Vec::new(),
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
    pub raw: docnav_typed_fields::JsonValue,
    pub invalid_reason: Option<crate::diagnostics::CandidateInvalidReason>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResolutionInputError {
    DuplicateSourceId(SourceId),
    UnknownField {
        source_id: SourceId,
        field: FieldIdentity,
    },
}

impl fmt::Display for ResolutionInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateSourceId(source_id) => write!(
                formatter,
                "source id {} is registered more than once",
                source_id.as_str()
            ),
            Self::UnknownField { source_id, field } => write!(
                formatter,
                "source {} contains candidate for unknown field {}",
                source_id.as_str(),
                field.as_str()
            ),
        }
    }
}

impl std::error::Error for ResolutionInputError {}

pub fn resolve(
    fields: &FieldDefSet,
    sources: &[Source],
) -> Result<ResolutionResult, ResolutionInputError> {
    let mut source_ids = BTreeSet::new();
    let mut by_field = BTreeMap::<FieldIdentity, Vec<EffectiveCandidate>>::new();
    for (source_order, source) in sources.iter().enumerate() {
        if !source_ids.insert(source.id().clone()) {
            return Err(ResolutionInputError::DuplicateSourceId(source.id().clone()));
        }
        for candidate in source.candidates() {
            if fields.field(candidate.field()).is_none() {
                return Err(ResolutionInputError::UnknownField {
                    source_id: source.id().clone(),
                    field: candidate.field().clone(),
                });
            }
            by_field.entry(candidate.field().clone()).or_default().push(
                EffectiveCandidate::from_source(source, source_order, candidate),
            );
        }
    }

    let mut diagnostics = Vec::new();
    let mut resolved = BTreeMap::new();
    for metadata in fields.schema_metadata() {
        let field = metadata.field();
        let identity = field.identity().clone();
        let candidates = by_field.remove(&identity).unwrap_or_default();
        let resolution = resolve_field(field, &metadata, candidates, &mut diagnostics);
        resolved.insert(identity, resolution);
    }
    Ok(ResolutionResult {
        fields: resolved,
        diagnostics,
    })
}
