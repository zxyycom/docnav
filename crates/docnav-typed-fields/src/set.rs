use std::collections::BTreeMap;
use std::fmt;

use crate::field::{FieldDef, FieldDefBuilder};
use crate::metadata::{
    BuildError, FieldDuplicateIdentityError, FieldPath, ProcessingMetadataView, SchemaMetadataView,
    ValueKind,
};
use crate::process_strategy::ProcessingInputKind;
use crate::processing::ProcessingId;

mod errors;
mod values;

pub use errors::{
    ExpectedFieldShape, FieldDefBuildFailure, FieldDefSetBuildError, FieldExtractionError,
};
pub use values::{FieldValidationErrors, FieldValues};

#[derive(Debug)]
pub struct FieldDefSet {
    fields: Vec<FieldDef>,
    processing_input_kinds: BTreeMap<ProcessingId, ProcessingInputKind>,
}

impl FieldDefSet {
    pub fn builder() -> FieldDefSetBuilder {
        FieldDefSetBuilder::new()
    }

    pub fn schema_metadata(&self) -> Vec<SchemaMetadataView> {
        self.fields.iter().map(FieldDef::schema_metadata).collect()
    }

    pub fn processing_metadata(&self, processing_id: &ProcessingId) -> Vec<ProcessingMetadataView> {
        self.fields
            .iter()
            .filter_map(|field| field.processing_metadata(processing_id))
            .collect()
    }

    pub fn value_kinds(&self) -> BTreeMap<String, ValueKind> {
        self.schema_metadata()
            .into_iter()
            .map(|metadata| (metadata.identity.as_str().to_string(), metadata.value_kind))
            .collect()
    }

    pub(crate) fn fields(&self) -> &[FieldDef] {
        &self.fields
    }

    pub(crate) fn static_default_values(&self) -> FieldValues {
        let values = self
            .fields
            .iter()
            .map(FieldDef::static_default_value)
            .collect();
        FieldValues { values }
    }

    pub(crate) fn require_processing_input_kind(
        &self,
        processing_id: &ProcessingId,
        actual: ProcessingInputKind,
    ) -> Result<(), FieldExtractionError> {
        let Some(expected) = self.processing_input_kinds.get(processing_id) else {
            return Err(FieldExtractionError::UnknownProcessing {
                processing_id: processing_id.clone(),
            });
        };
        if *expected == actual {
            Ok(())
        } else {
            Err(FieldExtractionError::InputKindMismatch {
                processing_id: processing_id.clone(),
                expected: *expected,
                actual,
            })
        }
    }
}

impl AsRef<FieldDefSet> for FieldDefSet {
    fn as_ref(&self) -> &FieldDefSet {
        self
    }
}

pub struct FieldDefSetBuilder {
    entries: Vec<FieldDefSetBuilderEntry>,
}

#[derive(Clone)]
pub struct FieldDefDeclaration {
    declaration_path: Option<Vec<String>>,
    expected: ExpectedFieldShape,
    builder: Box<dyn ErasedFieldDefBuilder>,
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
    fn clone_box(&self) -> Box<dyn ErasedFieldDefBuilder>;
}

impl<T: 'static> ErasedFieldDefBuilder for FieldDefBuilder<T> {
    fn build(self: Box<Self>) -> Result<FieldDef, BuildError> {
        (*self).build()
    }

    fn clone_box(&self) -> Box<dyn ErasedFieldDefBuilder> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn ErasedFieldDefBuilder> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl fmt::Debug for FieldDefDeclaration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FieldDefDeclaration")
            .field("declaration_path", &self.declaration_path)
            .field("expected", &self.expected)
            .finish_non_exhaustive()
    }
}

impl FieldDefDeclaration {
    pub fn with_declaration_path<T, I, S>(
        declaration_path: I,
        builder: FieldDefBuilder<T>,
        expected: ExpectedFieldShape,
    ) -> Self
    where
        T: 'static,
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            declaration_path: Some(declaration_path.into_iter().map(Into::into).collect()),
            expected,
            builder: Box::new(builder),
        }
    }

    pub fn declaration_path(&self) -> Option<&[String]> {
        self.declaration_path.as_deref()
    }

    pub fn schema_metadata(&self) -> Result<SchemaMetadataView, BuildError> {
        let mut definition = self.builder.clone().build()?;
        definition.apply_declaration_presence(self.expected.required, self.expected.nullable);
        Ok(definition.schema_metadata())
    }

    pub fn processing_metadata(
        &self,
        processing_id: &ProcessingId,
    ) -> Result<Option<ProcessingMetadataView>, BuildError> {
        let mut definition = self.builder.clone().build()?;
        definition.apply_declaration_presence(self.expected.required, self.expected.nullable);
        Ok(definition.processing_metadata(processing_id))
    }

    fn into_entry(self) -> FieldDefSetBuilderEntry {
        FieldDefSetBuilderEntry {
            declaration_path: self.declaration_path,
            expected: self.expected,
            builder: self.builder,
        }
    }
}

impl FieldDefSetBuilder {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn field_with_declaration_path<T, I, S>(
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
        self = self.field_declaration(FieldDefDeclaration::with_declaration_path(
            declaration_path,
            builder,
            expected,
        ));
        self
    }

    pub fn field_declaration(mut self, declaration: FieldDefDeclaration) -> Self {
        self.entries.push(declaration.into_entry());
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
            definition.apply_declaration_presence(expected.required, expected.nullable);
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
        let processing_input_kinds = processing_input_kinds(&fields)?;
        Ok(FieldDefSet {
            fields,
            processing_input_kinds,
        })
    }
}

fn processing_input_kinds(
    fields: &[FieldDef],
) -> Result<BTreeMap<ProcessingId, ProcessingInputKind>, FieldDefSetBuildError> {
    let mut input_kinds = BTreeMap::new();
    for field in fields {
        for (processing_id, input_kind) in field.processing_input_kinds() {
            if let Some(previous) = input_kinds.insert(processing_id.clone(), input_kind) {
                if previous != input_kind {
                    return Err(FieldDefSetBuildError::ProcessingInputKindConflict {
                        processing_id: processing_id.clone(),
                        previous,
                        current: input_kind,
                    });
                }
            }
        }
    }
    Ok(input_kinds)
}
