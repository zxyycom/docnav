use std::collections::BTreeMap;
use std::fmt;

use serde_json::Value;

use crate::field::{FieldDef, FieldDefBuilder};
use crate::metadata::{
    BuildError, FieldDuplicateIdentityError, FieldPath, ProcessingMetadataView, SchemaMetadataView,
    ValueKind,
};
use crate::process_strategy::ProcessingInputKind;
use crate::processing::{ProcessedExtraction, ProcessingBuild, ProcessingId};

mod errors;
mod json;
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
    #[doc(hidden)]
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

    pub fn validate_json(
        &self,
        processing_id: impl Into<ProcessingId>,
        root: &Value,
    ) -> Result<(), FieldExtractionError> {
        self.__extract_json_values(processing_id.into(), root)
            .map(|_| ())
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
    pub fn __extract_json_values(
        &self,
        processing_id: ProcessingId,
        root: &Value,
    ) -> Result<FieldValues, FieldExtractionError> {
        self.require_json_processing(&processing_id)?;
        self.extract_json_processing_values(&processing_id, root, JsonExtractionDefaults::Absent)
    }

    #[doc(hidden)]
    pub fn __extract_json_values_with_static_defaults(
        &self,
        processing_id: ProcessingId,
        root: &Value,
    ) -> Result<FieldValues, FieldExtractionError> {
        self.require_json_processing(&processing_id)?;
        self.extract_json_processing_values(&processing_id, root, JsonExtractionDefaults::Static)
    }

    #[doc(hidden)]
    pub fn __process_json_values<O>(
        &self,
        processing: &ProcessingBuild<'_, Value, O>,
        root: &Value,
    ) -> ProcessedExtraction<Result<FieldValues, FieldExtractionError>, O> {
        self.process_json_values(processing, root, JsonExtractionDefaults::Absent)
    }

    #[doc(hidden)]
    pub fn __process_json_values_with_static_defaults<O>(
        &self,
        processing: &ProcessingBuild<'_, Value, O>,
        root: &Value,
    ) -> ProcessedExtraction<Result<FieldValues, FieldExtractionError>, O> {
        self.process_json_values(processing, root, JsonExtractionDefaults::Static)
    }

    fn process_json_values<O>(
        &self,
        processing: &ProcessingBuild<'_, Value, O>,
        root: &Value,
        defaults: JsonExtractionDefaults,
    ) -> ProcessedExtraction<Result<FieldValues, FieldExtractionError>, O> {
        let values = self
            .require_json_processing(processing.id())
            .and_then(|()| self.extract_json_processing_values(processing.id(), root, defaults));
        ProcessedExtraction::new(values, processing.process(root.clone()))
    }

    fn require_json_processing(
        &self,
        processing_id: &ProcessingId,
    ) -> Result<(), FieldExtractionError> {
        let Some(expected) = self.processing_input_kinds.get(processing_id) else {
            return Err(FieldExtractionError::UnknownProcessing {
                processing_id: processing_id.clone(),
            });
        };
        if *expected == ProcessingInputKind::JsonValue {
            Ok(())
        } else {
            Err(FieldExtractionError::InputKindMismatch {
                processing_id: processing_id.clone(),
                expected: *expected,
                actual: ProcessingInputKind::JsonValue,
            })
        }
    }

    fn extract_json_processing_values(
        &self,
        processing_id: &ProcessingId,
        root: &Value,
        defaults: JsonExtractionDefaults,
    ) -> Result<FieldValues, FieldExtractionError> {
        let mut values = Vec::with_capacity(self.fields.len());
        let mut errors = Vec::new();
        for definition in &self.fields {
            let decoded = match defaults {
                JsonExtractionDefaults::Absent => definition.decode_process(processing_id, root),
                JsonExtractionDefaults::Static => {
                    definition.decode_process_with_static_default(processing_id, root)
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

impl AsRef<FieldDefSet> for FieldDefSet {
    fn as_ref(&self) -> &FieldDefSet {
        self
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
