use serde::{Deserialize, Serialize};
use std::fmt;

use crate::Operation;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub manifest_version: String,
    pub adapter: AdapterIdentity,
    pub formats: Vec<FormatDescriptor>,
    pub capabilities: Vec<Operation>,
}

impl Manifest {
    pub fn validate_semantics(&self) -> Result<(), ManifestValidationError> {
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestValidationError;

impl fmt::Display for ManifestValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("manifest semantic validation failed")
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
