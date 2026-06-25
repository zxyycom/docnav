use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde_json::Value;

use crate::field::{FieldDef, FieldDefBuilder};
use crate::metadata::{
    BuildError, FieldDuplicateIdentityError, SchemaMetadataView, TypedValue, ValidationFailure,
    ValueKind,
};
use crate::value::FieldValue;

mod errors;

pub use errors::{
    ExpectedFieldShape, FieldDeclarationShapeError, FieldDefBuildFailure, FieldDefSetBuildError,
};

#[derive(Debug)]
pub struct FieldDefSet {
    fields: Vec<FieldDef>,
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
    pub fn __validate_values_without_default(
        &self,
        root: &Value,
    ) -> Result<(), FieldValidationErrors> {
        self.__extract_values_without_default(root).map(|_| ())
    }

    #[doc(hidden)]
    pub fn __validate_values_with_static_defaults(
        &self,
        root: &Value,
    ) -> Result<(), FieldValidationErrors> {
        self.__extract_values_with_static_defaults(root).map(|_| ())
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
    pub fn __extract_values_without_default(
        &self,
        root: &Value,
    ) -> Result<FieldValues, FieldValidationErrors> {
        self.extract_values_by(root, FieldDef::decode_without_default)
    }

    #[doc(hidden)]
    pub fn __extract_values_with_static_defaults(
        &self,
        root: &Value,
    ) -> Result<FieldValues, FieldValidationErrors> {
        self.extract_values_by(root, FieldDef::decode_with_static_default)
    }

    fn extract_values_by(
        &self,
        root: &Value,
        decode: fn(&FieldDef, &Value) -> Result<Option<TypedValue>, ValidationFailure>,
    ) -> Result<FieldValues, FieldValidationErrors> {
        let mut values = Vec::with_capacity(self.fields.len());
        let mut errors = Vec::new();
        for definition in &self.fields {
            match decode(definition, root) {
                Ok(value) => values.push(value),
                Err(error) => errors.push(error),
            }
        }
        if errors.is_empty() {
            Ok(FieldValues { values })
        } else {
            Err(FieldValidationErrors::new(errors))
        }
    }
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
        let mut identities = BTreeSet::new();
        let mut fields = Vec::new();
        for entry in self.entries {
            let FieldDefSetBuilderEntry {
                declaration_path,
                expected,
                builder,
            } = entry;
            let definition = builder.build().map_err(|error| {
                FieldDefSetBuildError::Field(FieldDefBuildFailure {
                    declaration_path: declaration_path.clone(),
                    error,
                })
            })?;
            validate_declared_shape(&declaration_path, expected, &definition)?;
            if !identities.insert(definition.identity().clone()) {
                return Err(FieldDuplicateIdentityError {
                    field: definition.identity.clone(),
                    path: definition.path.clone(),
                }
                .into());
            }
            fields.push(definition);
        }
        Ok(FieldDefSet { fields })
    }
}

fn validate_declared_shape(
    declaration_path: &Option<Vec<String>>,
    expected: ExpectedFieldShape,
    definition: &FieldDef,
) -> Result<(), FieldDefSetBuildError> {
    let actual = ExpectedFieldShape {
        required: definition.is_required(),
    };
    if expected == actual {
        return Ok(());
    }
    Err(FieldDefSetBuildError::DeclarationShape(
        FieldDeclarationShapeError {
            declaration_path: declaration_path.clone(),
            expected,
            actual,
        },
    ))
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
