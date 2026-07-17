use std::collections::BTreeSet;
use std::fmt;

use docnav_adapter_contracts::StandardInputBinding;
use docnav_protocol::{Operation, PagedOperation};
use docnav_typed_fields::{
    FieldDefSet, FieldDefSetBuildError, FieldDefSetBuilder, FieldIdentity, ValueKind,
};

/// Closed consumer targets for one core-authored document parameter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentParameterBinding {
    StandardInput(StandardInputBinding),
    PaginationEnabled(PagedOperation),
    OutputMode(Operation),
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
        }
    }

    const fn expected_value_kind(self) -> ValueKind {
        match self {
            Self::StandardInput(binding) => binding.expected_value_kind(),
            Self::PaginationEnabled(_) => ValueKind::Boolean,
            Self::OutputMode(_) => ValueKind::String,
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
        let mut associated_fields = BTreeSet::new();
        let mut bound_targets: Vec<(FieldIdentity, Option<String>, DocumentParameterBinding)> =
            Vec::new();

        for entry in &entries {
            if !associated_fields.insert(entry.identity.clone()) {
                return Err(
                    DocumentParameterCatalogBuildError::DuplicateEntryAssociation {
                        identity: entry.identity.clone(),
                    },
                );
            }
            let Some(field) = fields.field(&entry.identity) else {
                return Err(
                    DocumentParameterCatalogBuildError::UnknownFieldAssociation {
                        identity: entry.identity.clone(),
                    },
                );
            };
            if let Some(adapter_id) = &entry.adapter_id {
                if !known_adapter_ids.contains(adapter_id) {
                    return Err(DocumentParameterCatalogBuildError::UnknownAdapterId {
                        identity: entry.identity.clone(),
                        adapter_id: adapter_id.clone(),
                    });
                }
            }
            validate_bindings(entry, field.value_kind())?;
            for binding in &entry.bindings {
                if let Some((previous_identity, _, _)) =
                    bound_targets
                        .iter()
                        .find(|(_, adapter_id, previous_binding)| {
                            previous_binding == binding
                                && adapter_scopes_overlap(
                                    adapter_id.as_deref(),
                                    entry.adapter_id.as_deref(),
                                )
                        })
                {
                    return Err(
                        DocumentParameterCatalogBuildError::DuplicateOperationTarget {
                            previous_identity: previous_identity.clone(),
                            current_identity: entry.identity.clone(),
                            binding: *binding,
                        },
                    );
                }
                bound_targets.push((entry.identity.clone(), entry.adapter_id.clone(), *binding));
            }
        }

        for metadata in fields.schema_metadata() {
            if !associated_fields.contains(metadata.identity()) {
                return Err(
                    DocumentParameterCatalogBuildError::MissingEntryAssociation {
                        identity: metadata.identity().clone(),
                    },
                );
            }
        }

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

#[derive(Debug, PartialEq)]
pub enum DocumentParameterCatalogBuildError {
    FieldSet {
        source: Box<FieldDefSetBuildError>,
    },
    DuplicateEntryAssociation {
        identity: FieldIdentity,
    },
    UnknownFieldAssociation {
        identity: FieldIdentity,
    },
    MissingEntryAssociation {
        identity: FieldIdentity,
    },
    UnknownAdapterId {
        identity: FieldIdentity,
        adapter_id: String,
    },
    MissingBinding {
        identity: FieldIdentity,
    },
    DuplicateBinding {
        identity: FieldIdentity,
        binding: DocumentParameterBinding,
    },
    DuplicateOperationTarget {
        previous_identity: FieldIdentity,
        current_identity: FieldIdentity,
        binding: DocumentParameterBinding,
    },
    InvalidOperationTarget {
        identity: FieldIdentity,
        operation: Operation,
    },
    BindingValueKindMismatch {
        identity: FieldIdentity,
        binding: DocumentParameterBinding,
        expected: ValueKind,
        actual: ValueKind,
    },
}

impl fmt::Display for DocumentParameterCatalogBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldSet { source } => {
                write!(formatter, "parameter field set is invalid: {source}")
            }
            Self::DuplicateEntryAssociation { identity } => write!(
                formatter,
                "parameter field {} has more than one catalog entry",
                identity.as_str()
            ),
            Self::UnknownFieldAssociation { identity } => write!(
                formatter,
                "catalog entry {} has no parameter field definition",
                identity.as_str()
            ),
            Self::MissingEntryAssociation { identity } => write!(
                formatter,
                "parameter field {} has no catalog entry",
                identity.as_str()
            ),
            Self::UnknownAdapterId {
                identity,
                adapter_id,
            } => write!(
                formatter,
                "catalog entry {} references unknown adapter {adapter_id}",
                identity.as_str()
            ),
            Self::MissingBinding { identity } => write!(
                formatter,
                "catalog entry {} has no consumer binding",
                identity.as_str()
            ),
            Self::DuplicateBinding { identity, binding } => write!(
                formatter,
                "catalog entry {} repeats consumer binding {binding:?}",
                identity.as_str()
            ),
            Self::DuplicateOperationTarget {
                previous_identity,
                current_identity,
                binding,
            } => write!(
                formatter,
                "catalog fields {} and {} both target {binding:?} for overlapping adapter scopes",
                previous_identity.as_str(),
                current_identity.as_str()
            ),
            Self::InvalidOperationTarget {
                identity,
                operation,
            } => write!(
                formatter,
                "catalog entry {} has more than one target for operation {operation}",
                identity.as_str()
            ),
            Self::BindingValueKindMismatch {
                identity,
                binding,
                expected,
                actual,
            } => write!(
                formatter,
                "catalog entry {} binding {binding:?} expects {expected:?}, got {actual:?}",
                identity.as_str()
            ),
        }
    }
}

impl std::error::Error for DocumentParameterCatalogBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::FieldSet { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

fn validate_bindings(
    entry: &DocumentParameterEntry,
    actual: ValueKind,
) -> Result<(), DocumentParameterCatalogBuildError> {
    if entry.bindings.is_empty() {
        return Err(DocumentParameterCatalogBuildError::MissingBinding {
            identity: entry.identity.clone(),
        });
    }

    let mut bindings = Vec::with_capacity(entry.bindings.len());
    let mut operations = BTreeSet::new();
    for binding in &entry.bindings {
        if bindings.contains(binding) {
            return Err(DocumentParameterCatalogBuildError::DuplicateBinding {
                identity: entry.identity.clone(),
                binding: *binding,
            });
        }
        bindings.push(*binding);

        let operation = binding.operation();
        if !operations.insert(operation) {
            return Err(DocumentParameterCatalogBuildError::InvalidOperationTarget {
                identity: entry.identity.clone(),
                operation,
            });
        }

        let expected = binding.expected_value_kind();
        if actual != expected {
            return Err(
                DocumentParameterCatalogBuildError::BindingValueKindMismatch {
                    identity: entry.identity.clone(),
                    binding: *binding,
                    expected,
                    actual,
                },
            );
        }
    }
    Ok(())
}

fn adapter_scopes_overlap(previous: Option<&str>, current: Option<&str>) -> bool {
    previous.is_none() || current.is_none() || previous == current
}

#[cfg(test)]
mod tests;
