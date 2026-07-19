use serde_json::Value;

use crate::field::FieldDef;
use crate::process_strategy::{CliProcessingMetadata, ProcessingInputKind, ProcessingLocator};
use crate::processing::ProcessingId;
use crate::range::{FieldLength, FieldNumericRange};

mod errors;
mod validation;

pub use errors::{BuildError, FieldDuplicateIdentityError, ValidationFailure, ValidationReason};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FieldIdentity(String);

impl FieldIdentity {
    pub fn new(value: impl Into<String>) -> Result<Self, BuildError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(BuildError::EmptyIdentity);
        }
        FieldPath::new(value.split('.'))?;
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
    Json,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MergeStrategy {
    #[default]
    Replace,
    Append,
    MapMerge,
    DenyConflict,
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
    Json(Value),
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
    pub unique_items: bool,
}

#[derive(Clone, Debug)]
pub struct SchemaMetadataView<'a> {
    pub(crate) field: &'a FieldDef,
    pub path: FieldPath,
}

impl SchemaMetadataView<'_> {
    pub fn field(&self) -> &FieldDef {
        self.field
    }

    pub fn identity(&self) -> &FieldIdentity {
        self.field.identity()
    }

    pub fn value_kind(&self) -> ValueKind {
        self.field.value_kind()
    }

    pub fn constraints(&self) -> &FieldConstraints {
        self.field.constraints()
    }

    pub fn default(&self) -> &DefaultMetadata {
        self.field.default()
    }

    pub fn merge_strategy(&self) -> MergeStrategy {
        self.field.merge_strategy()
    }
}

impl PartialEq for SchemaMetadataView<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.identity() == other.identity()
            && self.path == other.path
            && self.value_kind() == other.value_kind()
            && self.constraints() == other.constraints()
            && self.default() == other.default()
            && self.merge_strategy() == other.merge_strategy()
    }
}

#[derive(Clone, Debug)]
pub struct ProcessingMetadataView<'a> {
    pub(crate) field: &'a FieldDef,
    pub processing_id: ProcessingId,
    pub path: FieldPath,
    pub input_kind: ProcessingInputKind,
    pub locator: ProcessingLocator,
    pub cli: Option<CliProcessingMetadata>,
}

impl ProcessingMetadataView<'_> {
    pub fn field(&self) -> &FieldDef {
        self.field
    }

    pub fn identity(&self) -> &FieldIdentity {
        self.field.identity()
    }

    pub fn value_kind(&self) -> ValueKind {
        self.field.value_kind()
    }

    pub fn constraints(&self) -> &FieldConstraints {
        self.field.constraints()
    }

    pub fn default(&self) -> &DefaultMetadata {
        self.field.default()
    }

    pub fn merge_strategy(&self) -> MergeStrategy {
        self.field.merge_strategy()
    }
}

impl PartialEq for ProcessingMetadataView<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.identity() == other.identity()
            && self.processing_id == other.processing_id
            && self.path == other.path
            && self.input_kind == other.input_kind
            && self.locator == other.locator
            && self.value_kind() == other.value_kind()
            && self.constraints() == other.constraints()
            && self.default() == other.default()
            && self.merge_strategy() == other.merge_strategy()
            && self.cli == other.cli
    }
}
