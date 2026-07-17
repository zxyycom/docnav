use std::collections::BTreeMap;

use crate::field::FieldDef;
use crate::metadata::{FieldIdentity, ProcessingMetadataView, SchemaMetadataView, ValueKind};
use crate::process_strategy::ProcessingInputKind;
use crate::processing::ProcessingId;

mod builder;
mod declaration;
mod errors;
mod values;

pub use builder::FieldDefSetBuilder;
pub use errors::{
    ExpectedFieldShape, FieldDefBuildFailure, FieldDefSetBuildError,
    FieldDuplicateProcessingLocatorError, FieldDuplicateProcessingPathError, FieldExtractionError,
};
pub use values::{FieldValidationErrors, FieldValueMap};

#[derive(Debug)]
pub struct FieldDefSet {
    fields: Vec<FieldDef>,
    processing_input_kinds: BTreeMap<ProcessingId, ProcessingInputKind>,
}

impl FieldDefSet {
    pub fn builder() -> FieldDefSetBuilder {
        FieldDefSetBuilder::new()
    }

    pub fn schema_metadata(&self) -> Vec<SchemaMetadataView<'_>> {
        self.fields.iter().map(FieldDef::schema_metadata).collect()
    }

    pub fn processing_metadata(
        &self,
        processing_id: &ProcessingId,
    ) -> Vec<ProcessingMetadataView<'_>> {
        self.fields
            .iter()
            .filter_map(|field| field.processing_metadata(processing_id))
            .collect()
    }

    pub fn value_kinds(&self) -> BTreeMap<String, ValueKind> {
        self.schema_metadata()
            .into_iter()
            .map(|metadata| {
                (
                    metadata.identity().as_str().to_string(),
                    metadata.value_kind(),
                )
            })
            .collect()
    }

    pub fn field(&self, identity: &FieldIdentity) -> Option<&FieldDef> {
        self.fields
            .iter()
            .find(|field| field.identity() == identity)
    }

    /// Returns a validated set that keeps these definitions and adds borrowed canonical fields.
    pub fn with_fields<'a>(
        &self,
        fields: impl IntoIterator<Item = &'a FieldDef>,
    ) -> Result<Self, FieldDefSetBuildError> {
        builder::extend_with_built_fields(self, fields)
    }

    pub(crate) fn fields(&self) -> &[FieldDef] {
        &self.fields
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
