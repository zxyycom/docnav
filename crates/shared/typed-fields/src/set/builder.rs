use std::fmt;

use crate::field::{FieldDef, FieldDefBuilder};

use super::declaration::FieldDefSetBuilderEntry;
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
        self.entries
            .push(FieldDefSetBuilderEntry::new(None, builder, expected));
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
        self.entries.push(FieldDefSetBuilderEntry::new(
            Some(declaration_path.into_iter().map(Into::into).collect()),
            builder,
            expected,
        ));
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

pub(super) fn extend_with_built_fields<'a>(
    base: &FieldDefSet,
    fields: impl IntoIterator<Item = &'a FieldDef>,
) -> Result<FieldDefSet, FieldDefSetBuildError> {
    let mut definitions = DefinitionRegistry::default();
    for definition in &base.fields {
        definitions.register(super::declaration::BuiltFieldDeclaration {
            declaration_path: None,
            definition: definition.clone(),
        })?;
    }
    for definition in fields {
        definitions.register(super::declaration::BuiltFieldDeclaration {
            declaration_path: None,
            definition: definition.clone(),
        })?;
    }
    definitions.finish()
}

impl fmt::Debug for FieldDefSetBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FieldDefSetBuilder")
            .field("entries", &self.entries.len())
            .finish()
    }
}
