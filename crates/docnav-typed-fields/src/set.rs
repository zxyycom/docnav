use std::collections::BTreeMap;
use std::fmt;

use serde_json::Value;

use crate::extraction::{ExtractionInputKind, ExtractionStrategyId};
use crate::field::{FieldDef, FieldDefBuilder};
use crate::metadata::{
    BuildError, FieldDuplicateIdentityError, FieldPath, SchemaMetadataView, TypedValue,
    ValidationFailure, ValueKind,
};
use crate::value::FieldValue;

mod errors;

pub use errors::{
    ExpectedFieldShape, FieldDefBuildFailure, FieldDefSetBuildError, FieldExtractionError,
};

#[derive(Debug)]
pub struct FieldDefSet {
    fields: Vec<FieldDef>,
    extraction_input_kinds: BTreeMap<ExtractionStrategyId, ExtractionInputKind>,
}

impl FieldDefSet {
    #[doc(hidden)]
    pub fn builder() -> FieldDefSetBuilder {
        FieldDefSetBuilder::new()
    }

    pub fn schema_metadata(&self) -> Vec<SchemaMetadataView> {
        self.fields.iter().map(FieldDef::schema_metadata).collect()
    }

    pub fn value_kinds(&self) -> BTreeMap<String, ValueKind> {
        self.schema_metadata()
            .into_iter()
            .map(|metadata| (metadata.identity.as_str().to_string(), metadata.value_kind))
            .collect()
    }

    #[doc(hidden)]
    pub fn __static_default_values(&self) -> FieldValues {
        let values = self
            .fields
            .iter()
            .map(FieldDef::static_default_value)
            .collect();
        FieldValues { values }
    }

    #[doc(hidden)]
    pub fn __extract_json_strategy_values(
        &self,
        strategy_id: ExtractionStrategyId,
        root: &Value,
    ) -> Result<FieldValues, FieldExtractionError> {
        self.require_json_strategy(&strategy_id)?;
        self.extract_json_strategy_values(&strategy_id, root, JsonExtractionDefaults::Absent)
    }

    #[doc(hidden)]
    pub fn __extract_json_strategy_values_with_static_defaults(
        &self,
        strategy_id: ExtractionStrategyId,
        root: &Value,
    ) -> Result<FieldValues, FieldExtractionError> {
        self.require_json_strategy(&strategy_id)?;
        self.extract_json_strategy_values(&strategy_id, root, JsonExtractionDefaults::Static)
    }

    fn require_json_strategy(
        &self,
        strategy_id: &ExtractionStrategyId,
    ) -> Result<(), FieldExtractionError> {
        let Some(expected) = self.extraction_input_kinds.get(strategy_id) else {
            return Err(FieldExtractionError::UnknownStrategy {
                strategy_id: strategy_id.clone(),
            });
        };
        if *expected == ExtractionInputKind::JsonValue {
            Ok(())
        } else {
            Err(FieldExtractionError::InputKindMismatch {
                strategy_id: strategy_id.clone(),
                expected: *expected,
                actual: ExtractionInputKind::JsonValue,
            })
        }
    }

    fn extract_json_strategy_values(
        &self,
        strategy_id: &ExtractionStrategyId,
        root: &Value,
        defaults: JsonExtractionDefaults,
    ) -> Result<FieldValues, FieldExtractionError> {
        let mut values = Vec::with_capacity(self.fields.len());
        let mut errors = Vec::new();
        for definition in &self.fields {
            let decoded = match defaults {
                JsonExtractionDefaults::Absent => definition.decode_strategy(strategy_id, root),
                JsonExtractionDefaults::Static => {
                    definition.decode_strategy_with_static_default(strategy_id, root)
                }
            };
            match decoded {
                Ok(value) => values.push(value),
                Err(error) => errors.push(error),
            }
        }
        if errors.is_empty() {
            Ok(FieldValues { values })
        } else {
            Err(FieldExtractionError::Validation(
                FieldValidationErrors::new(errors),
            ))
        }
    }
}

#[derive(Clone, Copy)]
enum JsonExtractionDefaults {
    Absent,
    Static,
}

#[doc(hidden)]
pub struct FieldDefSetBuilder {
    entries: Vec<FieldDefSetBuilderEntry>,
}

struct FieldDefSetBuilderEntry {
    declaration_path: Option<Vec<String>>,
    expected: ExpectedFieldShape,
    builder: Box<dyn ErasedFieldDefBuilder>,
}

struct FieldIdentityLocation {
    declaration_path: Option<Vec<String>>,
    path: FieldPath,
}

impl fmt::Debug for FieldDefSetBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FieldDefSetBuilder")
            .field("entries", &self.entries.len())
            .finish()
    }
}

trait ErasedFieldDefBuilder {
    fn build(self: Box<Self>) -> Result<FieldDef, BuildError>;
}

impl<T> ErasedFieldDefBuilder for FieldDefBuilder<T> {
    fn build(self: Box<Self>) -> Result<FieldDef, BuildError> {
        (*self).build()
    }
}

impl FieldDefSetBuilder {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    #[doc(hidden)]
    pub fn __field_with_declaration_path<T, I, S>(
        mut self,
        declaration_path: I,
        builder: FieldDefBuilder<T>,
        expected: ExpectedFieldShape,
    ) -> Self
    where
        T: 'static,
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.entries.push(FieldDefSetBuilderEntry {
            declaration_path: Some(declaration_path.into_iter().map(Into::into).collect()),
            expected,
            builder: Box::new(builder),
        });
        self
    }

    pub fn build(self) -> Result<FieldDefSet, FieldDefSetBuildError> {
        let mut identities = BTreeMap::new();
        let mut fields = Vec::new();
        for entry in self.entries {
            let FieldDefSetBuilderEntry {
                declaration_path,
                expected,
                builder,
            } = entry;
            let mut definition = builder.build().map_err(|error| {
                FieldDefSetBuildError::Field(FieldDefBuildFailure {
                    declaration_path: declaration_path.clone(),
                    error,
                })
            })?;
            definition.apply_declaration_presence(expected.required);
            if let Some(previous) = identities.insert(
                definition.identity().clone(),
                FieldIdentityLocation {
                    declaration_path: declaration_path.clone(),
                    path: definition.path.clone(),
                },
            ) {
                return Err(FieldDuplicateIdentityError {
                    field: definition.identity.clone(),
                    path: definition.path.clone(),
                    declaration_path,
                    previous_path: previous.path,
                    previous_declaration_path: previous.declaration_path,
                }
                .into());
            }
            fields.push(definition);
        }
        let extraction_input_kinds = extraction_input_kinds(&fields)?;
        Ok(FieldDefSet {
            fields,
            extraction_input_kinds,
        })
    }
}

fn extraction_input_kinds(
    fields: &[FieldDef],
) -> Result<BTreeMap<ExtractionStrategyId, ExtractionInputKind>, FieldDefSetBuildError> {
    let mut input_kinds = BTreeMap::new();
    for field in fields {
        for (strategy_id, input_kind) in field.extraction_input_kinds() {
            if let Some(previous) = input_kinds.insert(strategy_id.clone(), input_kind) {
                if previous != input_kind {
                    return Err(FieldDefSetBuildError::ExtractionInputKindConflict {
                        strategy_id: strategy_id.clone(),
                        previous,
                        current: input_kind,
                    });
                }
            }
        }
    }
    Ok(input_kinds)
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldValidationErrors {
    failures: Vec<ValidationFailure>,
}

impl FieldValidationErrors {
    pub fn new(failures: Vec<ValidationFailure>) -> Self {
        Self { failures }
    }

    pub fn failures(&self) -> &[ValidationFailure] {
        &self.failures
    }

    pub fn into_failures(self) -> Vec<ValidationFailure> {
        self.failures
    }
}

impl fmt::Display for FieldValidationErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} field validation error(s)",
            self.failures.len()
        )
    }
}

impl std::error::Error for FieldValidationErrors {}

#[derive(Debug, PartialEq)]
#[doc(hidden)]
pub struct FieldValues {
    values: Vec<Option<TypedValue>>,
}

impl FieldValues {
    #[doc(hidden)]
    pub fn __typed_optional_slot<T: FieldValue>(&self, slot: usize) -> Option<T> {
        let value = self
            .values
            .get(slot)
            .expect("generated field definition slot is present");
        value.clone().map(|value| {
            T::from_typed_value(value).expect("generated field definition type is consistent")
        })
    }

    #[doc(hidden)]
    pub fn __typed_required_slot<T: FieldValue>(&self, slot: usize) -> T {
        self.__typed_optional_slot(slot)
            .expect("required field was extracted during validation")
    }
}
