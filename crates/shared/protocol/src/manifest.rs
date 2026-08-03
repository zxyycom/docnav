use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub manifest_version: String,
    pub adapter: AdapterIdentity,
    pub formats: Vec<FormatDescriptor>,
}

impl Manifest {
    pub fn validate_semantics(&self) -> Result<(), ManifestValidationError> {
        for format in &self.formats {
            if format.extensions.is_empty()
                || format
                    .extensions
                    .iter()
                    .any(|extension| !valid_extension_hint(extension))
                || format
                    .filenames
                    .iter()
                    .any(|filename| !valid_filename_hint(filename))
            {
                return Err(ManifestValidationError);
            }
        }
        Ok(())
    }
}

fn valid_extension_hint(extension: &str) -> bool {
    extension.len() > 1 && extension.starts_with('.') && !extension.contains(['/', '\\'])
}

fn valid_filename_hint(filename: &str) -> bool {
    !filename.is_empty() && filename != "." && filename != ".." && !filename.contains(['/', '\\'])
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
    pub filenames: Vec<String>,
    pub content_types: Vec<String>,
}
