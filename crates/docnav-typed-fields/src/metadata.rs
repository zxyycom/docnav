use std::fmt;

use serde_json::Value;

use crate::process_strategy::ProcessingInputKind;
use crate::processing::ProcessingId;
use crate::range::{FieldBound, FieldLength, FieldNumericBound, FieldNumericRange};

mod validation;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FieldIdentity(String);

impl FieldIdentity {
    pub fn new(value: impl Into<String>) -> Result<Self, BuildError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(BuildError::EmptyIdentity);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldPath(Vec<String>);

impl FieldPath {
    pub fn new<I, S>(segments: I) -> Result<Self, BuildError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let segments = segments.into_iter().map(Into::into).collect::<Vec<_>>();
        if segments.is_empty() {
            return Err(BuildError::EmptyPath);
        }
        if segments.iter().any(|segment| segment.is_empty()) {
            return Err(BuildError::EmptyPathSegment);
        }
        Ok(Self(segments))
    }

    pub fn segments(&self) -> Vec<&str> {
        self.0.iter().map(String::as_str).collect()
    }

    pub(crate) fn raw_segments(&self) -> &[String] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueKind {
    String,
    Integer,
    Number,
    Boolean,
    Array,
    Object,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActualValueKind {
    String,
    Integer,
    Number,
    Boolean,
    Array,
    Object,
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypedValue {
    String(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Object(serde_json::Map<String, Value>),
    Null,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum DefaultMetadata {
    #[default]
    None,
    Static(Value),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FieldConstraints {
    pub required: bool,
    pub nullable: bool,
    pub enum_values: Option<Vec<Value>>,
    pub numeric_range: FieldNumericRange,
    pub length_range: Option<FieldLength>,
    pub regex: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SchemaMetadataView {
    pub identity: FieldIdentity,
    pub path: FieldPath,
    pub value_kind: ValueKind,
    pub constraints: FieldConstraints,
    pub default: DefaultMetadata,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProcessingMetadataView {
    pub identity: FieldIdentity,
    pub processing_id: ProcessingId,
    pub path: FieldPath,
    pub input_kind: ProcessingInputKind,
    pub value_kind: ValueKind,
    pub constraints: FieldConstraints,
    pub default: DefaultMetadata,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BuildError {
    EmptyIdentity,
    MissingProcessingStrategy,
    EmptyPath,
    EmptyPathSegment,
    MissingValidation,
    EmptyProcessingId,
    EmptyEnumValues,
    NonFiniteRangeBound,
    NonFiniteDefaultValue,
    InvalidRange,
    InvalidRegexPattern { pattern: String, error: String },
    InvalidEnumValue(ValidationFailure),
    InvalidDefault(ValidationFailure),
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
