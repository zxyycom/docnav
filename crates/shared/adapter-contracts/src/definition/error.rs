use std::fmt;

use docnav_protocol::Operation;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterDefinitionError {
    MissingAdapter {
        id: String,
    },
    MissingManifest {
        id: String,
    },
    ManifestIdMismatch {
        id: String,
        manifest_id: String,
    },
    MissingRequiredHandlers {
        id: String,
        operations: Vec<Operation>,
    },
    DuplicateOperationHandler {
        id: String,
        operation: Operation,
    },
    InvalidNativeOption {
        id: String,
        option: String,
        reason: String,
    },
    DuplicateNativeOptionDeclaration {
        id: String,
        option: String,
    },
    DuplicateNativeOptionPath {
        id: String,
        owner: String,
        namespace: String,
        key: String,
    },
    DuplicateCapabilityGroup {
        id: String,
        capability: &'static str,
    },
    UnsupportedCapabilityCombination {
        id: String,
        capability: &'static str,
        reason: &'static str,
    },
}

impl fmt::Display for AdapterDefinitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingAdapter { id } => {
                write!(formatter, "adapter definition {id} is missing adapter handle")
            }
            Self::MissingManifest { id } => {
                write!(formatter, "adapter definition {id} is missing manifest")
            }
            Self::ManifestIdMismatch { id, manifest_id } => write!(
                formatter,
                "adapter definition id {id} does not match manifest id {manifest_id}"
            ),
            Self::MissingRequiredHandlers { id, operations } => write!(
                formatter,
                "adapter definition {id} is missing required operation handlers: {:?}",
                operations
            ),
            Self::DuplicateOperationHandler { id, operation } => write!(
                formatter,
                "adapter definition {id} declares duplicate {operation} handler"
            ),
            Self::InvalidNativeOption { id, option, reason } => write!(
                formatter,
                "adapter definition {id} has invalid native option {option}: {reason}"
            ),
            Self::DuplicateNativeOptionDeclaration { id, option } => write!(
                formatter,
                "adapter definition {id} declares duplicate native option {option}"
            ),
            Self::DuplicateNativeOptionPath {
                id,
                owner,
                namespace,
                key,
            } => write!(
                formatter,
                "adapter definition {id} declares duplicate native option path {owner}.{namespace}.{key}"
            ),
            Self::DuplicateCapabilityGroup { id, capability } => write!(
                formatter,
                "adapter definition {id} declares duplicate {capability} capability group"
            ),
            Self::UnsupportedCapabilityCombination {
                id,
                capability,
                reason,
            } => write!(
                formatter,
                "adapter definition {id} has unsupported {capability} capability combination: {reason}"
            ),
        }
    }
}

impl std::error::Error for AdapterDefinitionError {}
