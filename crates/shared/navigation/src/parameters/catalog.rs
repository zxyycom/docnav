use std::collections::BTreeSet;

use docnav_adapter_contracts::StandardInputBinding;
use docnav_protocol::{Operation, PagedOperation};
use docnav_typed_fields::{FieldDefSet, FieldDefSetBuilder, FieldIdentity, ValueKind};

mod error;
mod validation;

pub use error::DocumentParameterCatalogBuildError;
use validation::validate_catalog_entries;

/// Closed consumer targets for one core-authored document parameter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentParameterBinding {
    StandardInput(StandardInputBinding),
    PaginationEnabled(PagedOperation),
    OutputMode(Operation),
    AutoReadMode(Operation),
}

impl DocumentParameterBinding {
    pub const fn operation(self) -> Operation {
        match self {
            Self::StandardInput(binding) => binding.operation(),
            Self::PaginationEnabled(operation) => match operation {
                PagedOperation::Outline => Operation::Outline,
                PagedOperation::Read => Operation::Read,
                PagedOperation::Find => Operation::Find,
            },
            Self::OutputMode(operation) => operation,
            Self::AutoReadMode(operation) => operation,
        }
    }

    const fn expected_value_kind(self) -> ValueKind {
        match self {
            Self::StandardInput(binding) => binding.expected_value_kind(),
            Self::PaginationEnabled(_) => ValueKind::Boolean,
            Self::OutputMode(_) | Self::AutoReadMode(_) => ValueKind::String,
        }
    }
}

/// Core-authored association between one canonical field and its consumer targets.
#[derive(Clone, Debug)]
pub struct DocumentParameterEntry {
    identity: FieldIdentity,
    adapter_id: Option<String>,
    bindings: Vec<DocumentParameterBinding>,
}

impl DocumentParameterEntry {
    pub fn new(
        identity: FieldIdentity,
        adapter_id: Option<String>,
        bindings: Vec<DocumentParameterBinding>,
    ) -> Self {
        Self {
            identity,
            adapter_id,
            bindings,
        }
    }

    pub fn identity(&self) -> &FieldIdentity {
        &self.identity
    }

    pub fn adapter_id(&self) -> Option<&str> {
        self.adapter_id.as_deref()
    }

    pub fn bindings(&self) -> &[DocumentParameterBinding] {
        &self.bindings
    }

    pub fn operations(&self) -> impl Iterator<Item = Operation> + '_ {
        self.bindings.iter().map(|binding| binding.operation())
    }

    pub fn applies_to(&self, operation: Operation) -> bool {
        self.operations()
            .any(|bound_operation| bound_operation == operation)
    }
}

/// Passive product-parameter facts validated against canonical typed-field definitions.
#[derive(Debug)]
pub struct DocumentParameterCatalog {
    known_adapter_ids: BTreeSet<String>,
    fields: FieldDefSet,
    entries: Vec<DocumentParameterEntry>,
}

impl DocumentParameterCatalog {
    pub fn new<I, S>(
        known_adapter_ids: I,
        fields: FieldDefSetBuilder,
        entries: Vec<DocumentParameterEntry>,
    ) -> Result<Self, DocumentParameterCatalogBuildError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let fields =
            fields
                .build()
                .map_err(|source| DocumentParameterCatalogBuildError::FieldSet {
                    source: Box::new(source),
                })?;
        let known_adapter_ids = known_adapter_ids
            .into_iter()
            .map(Into::into)
            .collect::<BTreeSet<_>>();
        validate_catalog_entries(&fields, &known_adapter_ids, &entries)?;

        Ok(Self {
            known_adapter_ids,
            fields,
            entries,
        })
    }

    pub fn is_known_adapter_id(&self, adapter_id: &str) -> bool {
        self.known_adapter_ids.contains(adapter_id)
    }

    pub fn fields(&self) -> &FieldDefSet {
        &self.fields
    }

    pub fn entries(&self) -> &[DocumentParameterEntry] {
        &self.entries
    }

    pub fn entry(&self, identity: &FieldIdentity) -> Option<&DocumentParameterEntry> {
        self.entries
            .iter()
            .find(|entry| entry.identity() == identity)
    }

    pub fn operation_fields(
        &self,
        operation: Operation,
    ) -> impl Iterator<Item = &docnav_typed_fields::FieldDef> {
        self.entries
            .iter()
            .filter(move |entry| entry.applies_to(operation))
            .map(|entry| {
                self.fields
                    .field(entry.identity())
                    .expect("validated catalog entries have canonical fields")
            })
    }

    pub fn selected_operation_fields<'a>(
        &'a self,
        selected_adapter_id: &'a str,
        operation: Operation,
    ) -> impl Iterator<Item = &'a docnav_typed_fields::FieldDef> + 'a {
        self.selected_operation_entries(selected_adapter_id, operation)
            .map(|entry| {
                self.fields
                    .field(entry.identity())
                    .expect("validated catalog entries have canonical fields")
            })
    }

    pub fn selected_operation_parameters<'a>(
        &'a self,
        selected_adapter_id: &'a str,
        operation: Operation,
    ) -> impl Iterator<
        Item = (
            &'a docnav_typed_fields::FieldDef,
            &'a DocumentParameterEntry,
            DocumentParameterBinding,
        ),
    > + 'a {
        self.selected_operation_entries(selected_adapter_id, operation)
            .flat_map(move |entry| {
                entry
                    .bindings()
                    .iter()
                    .copied()
                    .filter(move |binding| binding.operation() == operation)
                    .map(move |binding| {
                        (
                            self.fields
                                .field(entry.identity())
                                .expect("validated catalog entries have canonical fields"),
                            entry,
                            binding,
                        )
                    })
            })
    }

    fn selected_operation_entries<'a>(
        &'a self,
        selected_adapter_id: &'a str,
        operation: Operation,
    ) -> impl Iterator<Item = &'a DocumentParameterEntry> + 'a {
        self.entries.iter().filter(move |entry| {
            entry.applies_to(operation)
                && entry
                    .adapter_id()
                    .is_none_or(|adapter_id| adapter_id == selected_adapter_id)
        })
    }
}

#[cfg(test)]
mod tests;
