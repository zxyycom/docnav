use std::fmt;

use crate::field::{FieldDef, FieldDefBuilder};
use crate::metadata::{BuildError, ProcessingMetadataView, SchemaMetadataView};
use crate::processing::ProcessingId;

use super::{ExpectedFieldShape, FieldDefBuildFailure, FieldDefSetBuildError};

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

#[derive(Clone)]
pub struct FieldDefDeclaration {
    declaration_path: Option<Vec<String>>,
    expected: ExpectedFieldShape,
    builder: Box<dyn ErasedFieldDefBuilder>,
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
    pub fn new<T>(builder: FieldDefBuilder<T>, expected: ExpectedFieldShape) -> Self
    where
        T: 'static,
    {
        Self {
            declaration_path: None,
            expected,
            builder: Box::new(builder),
        }
    }

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

    pub(super) fn into_entry(self) -> FieldDefSetBuilderEntry {
        FieldDefSetBuilderEntry {
            declaration_path: self.declaration_path,
            expected: self.expected,
            builder: self.builder,
        }
    }
}

pub(super) struct BuiltFieldDeclaration {
    pub(super) declaration_path: Option<Vec<String>>,
    pub(super) definition: FieldDef,
}

pub(super) struct FieldDefSetBuilderEntry {
    declaration_path: Option<Vec<String>>,
    expected: ExpectedFieldShape,
    builder: Box<dyn ErasedFieldDefBuilder>,
}

impl FieldDefSetBuilderEntry {
    pub(super) fn build(self) -> Result<BuiltFieldDeclaration, FieldDefSetBuildError> {
        let mut definition = self.builder.build().map_err(|error| {
            FieldDefSetBuildError::Field(FieldDefBuildFailure {
                declaration_path: self.declaration_path.clone(),
                error,
            })
        })?;
        definition.apply_declaration_presence(self.expected.required, self.expected.nullable);
        Ok(BuiltFieldDeclaration {
            declaration_path: self.declaration_path,
            definition,
        })
    }
}
