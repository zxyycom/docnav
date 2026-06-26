use crate::metadata::{BuildError, FieldPath};
use crate::processing::ProcessingId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessingInputKind {
    JsonValue,
    RustField,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProcessStrategy {
    kind: ProcessStrategyKind,
}

impl ProcessStrategy {
    pub fn json_path<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            kind: ProcessStrategyKind::JsonPath(segments.into_iter().map(Into::into).collect()),
        }
    }

    pub fn rust_field() -> Self {
        Self {
            kind: ProcessStrategyKind::RustField,
        }
    }

    pub(crate) fn build(self) -> Result<BuiltProcessStrategy, crate::metadata::BuildError> {
        match self.kind {
            ProcessStrategyKind::JsonPath(segments) => {
                Ok(BuiltProcessStrategy::JsonPath(FieldPath::new(segments)?))
            }
            ProcessStrategyKind::RustField => Ok(BuiltProcessStrategy::RustField),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum ProcessStrategyKind {
    JsonPath(Vec<String>),
    RustField,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum BuiltProcessStrategy {
    JsonPath(FieldPath),
    RustField,
}

impl BuiltProcessStrategy {
    pub(crate) fn input_kind(&self) -> ProcessingInputKind {
        match self {
            Self::JsonPath(_) => ProcessingInputKind::JsonValue,
            Self::RustField => ProcessingInputKind::RustField,
        }
    }

    pub(crate) fn json_path(&self) -> Option<&FieldPath> {
        match self {
            Self::JsonPath(path) => Some(path),
            Self::RustField => None,
        }
    }
}

pub(crate) fn validate_processing_id(id: ProcessingId) -> Result<ProcessingId, BuildError> {
    id.validate().map_err(|_| BuildError::EmptyProcessingId)
}
