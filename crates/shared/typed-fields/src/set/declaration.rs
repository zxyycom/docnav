use crate::field::{FieldDef, FieldDefBuilder};
use crate::metadata::BuildError;

use super::{ExpectedFieldShape, FieldDefBuildFailure, FieldDefSetBuildError};

trait ErasedFieldDefBuilder {
    fn build(self: Box<Self>) -> Result<FieldDef, BuildError>;
}

impl<T: 'static> ErasedFieldDefBuilder for FieldDefBuilder<T> {
    fn build(self: Box<Self>) -> Result<FieldDef, BuildError> {
        (*self).build()
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
    pub(super) fn new<T>(
        declaration_path: Option<Vec<String>>,
        builder: FieldDefBuilder<T>,
        expected: ExpectedFieldShape,
    ) -> Self
    where
        T: 'static,
    {
        Self {
            declaration_path,
            expected,
            builder: Box::new(builder),
        }
    }

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
