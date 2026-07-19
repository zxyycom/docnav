use std::fmt;

use serde_json::Value;

use crate::processing::ProcessingId;
use crate::range::{FieldBound, FieldNumericBound};

use super::{ActualValueKind, FieldIdentity, FieldPath, MergeStrategy, ValueKind};

#[derive(Clone, Debug, PartialEq)]
pub enum BuildError {
    EmptyIdentity,
    MissingProcessingStrategy,
    EmptyPath,
    EmptyPathSegment,
    MissingValidation,
    EmptyProcessingId,
    DuplicateProcessingId {
        processing_id: ProcessingId,
    },
    EmptyEnumValues,
    NonFiniteRangeBound,
    NonFiniteDefaultValue,
    InvalidRange,
    InvalidRegexPattern {
        pattern: String,
        error: String,
    },
    InvalidEnumValue(ValidationFailure),
    InvalidDefault(ValidationFailure),
    InvalidCliFlag,
    CliMetadataRequiresCliFlag,
    DuplicateCliMetadata,
    IncompatibleCliBooleanEncoding {
        value_kind: ValueKind,
    },
    IncompleteCliBooleanMapping,
    AmbiguousCliBooleanMapping {
        token: String,
    },
    InvalidEnvVar,
    IncompatibleMergeStrategy {
        value_kind: ValueKind,
        merge_strategy: MergeStrategy,
    },
}

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyIdentity => write!(formatter, "field identity is empty"),
            Self::MissingProcessingStrategy => {
                write!(formatter, "field processing strategy is missing")
            }
            Self::EmptyPath => write!(formatter, "field path is empty"),
            Self::EmptyPathSegment => write!(formatter, "field path contains an empty segment"),
            Self::MissingValidation => write!(formatter, "field validation is missing"),
            Self::EmptyProcessingId => write!(formatter, "processing id is empty"),
            Self::DuplicateProcessingId { processing_id } => {
                write!(
                    formatter,
                    "processing id {processing_id} is declared more than once"
                )
            }
            Self::EmptyEnumValues => write!(formatter, "enum constraint has no allowed values"),
            Self::NonFiniteRangeBound => write!(formatter, "numeric range bound is not finite"),
            Self::NonFiniteDefaultValue => {
                write!(
                    formatter,
                    "static default number value is not finite: Rust f64 can represent \
                     non-finite values, but JSON numbers cannot encode NaN or infinity"
                )
            }
            Self::InvalidRange => write!(formatter, "minimum range bound excludes all values"),
            Self::InvalidRegexPattern { pattern, error } => {
                write!(formatter, "regex pattern {pattern:?} is invalid: {error}")
            }
            Self::InvalidEnumValue(error) => write!(formatter, "enum value is invalid: {error}"),
            Self::InvalidDefault(error) => write!(formatter, "default value is invalid: {error}"),
            Self::InvalidCliFlag => write!(formatter, "CLI flag locator is invalid"),
            Self::CliMetadataRequiresCliFlag => {
                write!(
                    formatter,
                    "CLI metadata requires a CLI flag processing strategy"
                )
            }
            Self::DuplicateCliMetadata => {
                write!(formatter, "CLI metadata is declared more than once")
            }
            Self::IncompatibleCliBooleanEncoding { value_kind } => write!(
                formatter,
                "CLI Boolean encoding is incompatible with value kind {value_kind:?}"
            ),
            Self::IncompleteCliBooleanMapping => write!(
                formatter,
                "CLI Boolean token mapping must declare true and false tokens"
            ),
            Self::AmbiguousCliBooleanMapping { token } => write!(
                formatter,
                "CLI Boolean token mapping assigns {token:?} to both true and false"
            ),
            Self::InvalidEnvVar => write!(formatter, "environment variable locator is invalid"),
            Self::IncompatibleMergeStrategy {
                value_kind,
                merge_strategy,
            } => write!(
                formatter,
                "merge strategy {merge_strategy:?} is incompatible with value kind {value_kind:?}"
            ),
        }
    }
}

impl std::error::Error for BuildError {}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidationFailure {
    pub field: FieldIdentity,
    pub path: FieldPath,
    pub reason: ValidationReason,
}

impl fmt::Display for ValidationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {} failed validation: {:?}",
            self.field.as_str(),
            self.path.raw_segments().join("."),
            self.reason
        )
    }
}

impl std::error::Error for ValidationFailure {}

#[derive(Clone, Debug, PartialEq)]
pub enum ValidationReason {
    MissingRequired,
    WrongType {
        expected: ValueKind,
        actual: ActualValueKind,
    },
    DisallowedEnumValue {
        allowed: Vec<Value>,
    },
    BelowMinimum {
        minimum: FieldNumericBound,
    },
    AboveMaximum {
        maximum: FieldNumericBound,
    },
    BelowMinimumLength {
        minimum: FieldBound<u64>,
    },
    AboveMaximumLength {
        maximum: FieldBound<u64>,
    },
    RegexMismatch {
        pattern: String,
    },
    DuplicateArrayItem {
        first_index: usize,
        duplicate_index: usize,
    },
}

#[derive(Debug, PartialEq)]
pub struct FieldDuplicateIdentityError {
    pub field: FieldIdentity,
    pub path: FieldPath,
    pub declaration_path: Option<Vec<String>>,
    pub previous_path: FieldPath,
    pub previous_declaration_path: Option<Vec<String>>,
}

impl fmt::Display for FieldDuplicateIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "field {} is declared more than once at {} (previous declaration at {})",
            self.field.as_str(),
            display_optional_path(&self.declaration_path, &self.path),
            display_optional_path(&self.previous_declaration_path, &self.previous_path),
        )
    }
}

impl std::error::Error for FieldDuplicateIdentityError {}

fn display_optional_path(declaration_path: &Option<Vec<String>>, field_path: &FieldPath) -> String {
    declaration_path
        .as_ref()
        .map(|path| path.join("."))
        .unwrap_or_else(|| field_path.raw_segments().join("."))
}
