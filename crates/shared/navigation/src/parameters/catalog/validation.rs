use std::collections::BTreeSet;

use docnav_typed_fields::{FieldDefSet, FieldIdentity, ValueKind};

use super::{DocumentParameterBinding, DocumentParameterCatalogBuildError, DocumentParameterEntry};

type BoundTarget = (FieldIdentity, Option<String>, DocumentParameterBinding);

pub(super) fn validate_catalog_entries(
    fields: &FieldDefSet,
    known_adapter_ids: &BTreeSet<String>,
    entries: &[DocumentParameterEntry],
) -> Result<(), DocumentParameterCatalogBuildError> {
    let mut associated_fields = BTreeSet::new();
    let mut bound_targets = Vec::new();

    for entry in entries {
        validate_entry_association(entry, fields, known_adapter_ids, &mut associated_fields)?;
        validate_entry_targets(entry, &mut bound_targets)?;
    }

    validate_all_fields_are_associated(fields, &associated_fields)
}

fn validate_entry_association(
    entry: &DocumentParameterEntry,
    fields: &FieldDefSet,
    known_adapter_ids: &BTreeSet<String>,
    associated_fields: &mut BTreeSet<FieldIdentity>,
) -> Result<(), DocumentParameterCatalogBuildError> {
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
    validate_bindings(entry, field.value_kind())
}

fn validate_entry_targets(
    entry: &DocumentParameterEntry,
    bound_targets: &mut Vec<BoundTarget>,
) -> Result<(), DocumentParameterCatalogBuildError> {
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
    Ok(())
}

fn validate_all_fields_are_associated(
    fields: &FieldDefSet,
    associated_fields: &BTreeSet<FieldIdentity>,
) -> Result<(), DocumentParameterCatalogBuildError> {
    for metadata in fields.schema_metadata() {
        if !associated_fields.contains(metadata.identity()) {
            return Err(
                DocumentParameterCatalogBuildError::MissingEntryAssociation {
                    identity: metadata.identity().clone(),
                },
            );
        }
    }
    Ok(())
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
