use std::fmt;

use crate::metadata::FieldPath;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExtractionStrategyId(String);

impl ExtractionStrategyId {
    pub(crate) fn validate(self) -> Result<Self, InvalidExtractionStrategyId> {
        if self.0.trim().is_empty() {
            Err(InvalidExtractionStrategyId)
        } else {
            Ok(self)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ExtractionStrategyId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for ExtractionStrategyId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl fmt::Display for ExtractionStrategyId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidExtractionStrategyId;

impl fmt::Display for InvalidExtractionStrategyId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("extraction strategy id is empty")
    }
}

impl std::error::Error for InvalidExtractionStrategyId {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtractionInputKind {
    JsonValue,
    RustField,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExtractStrategy {
    kind: ExtractStrategyKind,
}

impl ExtractStrategy {
    pub fn json_path<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            kind: ExtractStrategyKind::JsonPath(segments.into_iter().map(Into::into).collect()),
        }
    }

    pub fn rust_field() -> Self {
        Self {
            kind: ExtractStrategyKind::RustField,
        }
    }

    pub(crate) fn build(self) -> Result<BuiltExtractStrategy, crate::metadata::BuildError> {
        match self.kind {
            ExtractStrategyKind::JsonPath(segments) => {
                Ok(BuiltExtractStrategy::JsonPath(FieldPath::new(segments)?))
            }
            ExtractStrategyKind::RustField => Ok(BuiltExtractStrategy::RustField),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum ExtractStrategyKind {
    JsonPath(Vec<String>),
    RustField,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum BuiltExtractStrategy {
    JsonPath(FieldPath),
    RustField,
}

impl BuiltExtractStrategy {
    pub(crate) fn input_kind(&self) -> ExtractionInputKind {
        match self {
            Self::JsonPath(_) => ExtractionInputKind::JsonValue,
            Self::RustField => ExtractionInputKind::RustField,
        }
    }

    pub(crate) fn json_path(&self) -> Option<&FieldPath> {
        match self {
            Self::JsonPath(path) => Some(path),
            Self::RustField => None,
        }
    }
}
