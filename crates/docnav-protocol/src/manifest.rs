use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

use crate::{Operation, Options, PagedOperation, PositiveInteger, ProtocolRange, ProtocolVersion};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub manifest_version: String,
    pub adapter: AdapterIdentity,
    pub protocol: ProtocolRange,
    pub formats: Vec<FormatDescriptor>,
    pub capabilities: Vec<Operation>,
    pub recommended_parameters: BTreeMap<PagedOperation, RecommendedParameters>,
}

impl Manifest {
    pub fn validate_semantics(&self) -> Result<(), ManifestValidationError> {
        if self.protocol.min > self.protocol.max {
            return Err(ManifestValidationError::InvalidProtocolRange {
                min: self.protocol.min,
                max: self.protocol.max,
            });
        }

        for operation in self.recommended_parameters.keys() {
            let capability = Operation::from(*operation);
            if !self.capabilities.contains(&capability) {
                return Err(
                    ManifestValidationError::RecommendedParameterWithoutCapability {
                        operation: *operation,
                    },
                );
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManifestValidationError {
    InvalidProtocolRange {
        min: ProtocolVersion,
        max: ProtocolVersion,
    },
    RecommendedParameterWithoutCapability {
        operation: PagedOperation,
    },
}

impl fmt::Display for ManifestValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProtocolRange { min, max } => write!(
                formatter,
                "manifest protocol range min {min} is greater than max {max}"
            ),
            Self::RecommendedParameterWithoutCapability { operation } => write!(
                formatter,
                "manifest recommended_parameters.{operation} is declared without matching capability"
            ),
        }
    }
}

impl std::error::Error for ManifestValidationError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterIdentity {
    pub id: String,
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormatDescriptor {
    pub id: String,
    pub extensions: Vec<String>,
    pub content_types: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecommendedParameters {
    pub limit_chars: PositiveInteger,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Options>,
}
