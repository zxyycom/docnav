use std::fmt;

use docnav_protocol::Operation;
use docnav_typed_fields::{FieldDefSetBuildError, FieldIdentity, ValueKind};

use super::DocumentParameterBinding;

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
