use std::fmt;

use crate::field::FieldDefBuilder;

use super::declaration::{FieldDefDeclaration, FieldDefSetBuilderEntry};
use super::{ExpectedFieldShape, FieldDefSet, FieldDefSetBuildError};

mod registry;

use registry::DefinitionRegistry;

pub struct FieldDefSetBuilder {
    entries: Vec<FieldDefSetBuilderEntry>,
}

impl FieldDefSetBuilder {
    pub(super) fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn field<T>(mut self, builder: FieldDefBuilder<T>, expected: ExpectedFieldShape) -> Self
    where
        T: 'static,
    {
        self = self.field_declaration(FieldDefDeclaration::new(builder, expected));
        self
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
        let mut definitions = DefinitionRegistry::default();
        for entry in self.entries {
            definitions.register(entry.build()?)?;
        }
        definitions.finish()
    }
}

impl fmt::Debug for FieldDefSetBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FieldDefSetBuilder")
            .field("entries", &self.entries.len())
            .finish()
    }
}
