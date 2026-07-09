use std::fmt;

use crate::metadata::{BuildError, FieldDuplicateIdentityError, FieldIdentity, FieldPath};
use crate::{ProcessingId, ProcessingInputKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpectedFieldShape {
    pub required: bool,
    pub nullable: bool,
}

impl ExpectedFieldShape {
    pub const fn required() -> Self {
        Self {
            required: true,
            nullable: false,
        }
    }

    pub const fn optional() -> Self {
        Self {
            required: false,
            nullable: true,
        }
    }

    pub const fn required_nullable() -> Self {
        Self {
            required: true,
            nullable: true,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FieldDefBuildFailure {
    pub declaration_path: Option<Vec<String>>,
    pub error: BuildError,
}

impl fmt::Display for FieldDefBuildFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.declaration_path {
            Some(path) => write!(formatter, "{}: {}", path.join("."), self.error),
            None => write!(formatter, "{}", self.error),
        }
    }
}

impl std::error::Error for FieldDefBuildFailure {}

#[derive(Debug, PartialEq)]
pub enum FieldDefSetBuildError {
    Field(FieldDefBuildFailure),
    DuplicateIdentity(FieldDuplicateIdentityError),
    ProcessingInputKindConflict {
        processing_id: ProcessingId,
        previous: ProcessingInputKind,
        current: ProcessingInputKind,
    },
    DuplicateProcessingPath(Box<FieldDuplicateProcessingPathError>),
}

#[derive(Debug, PartialEq)]
pub struct FieldDuplicateProcessingPathError {
    pub processing_id: ProcessingId,
    pub path: FieldPath,
    pub previous_identity: FieldIdentity,
    pub previous_declaration_path: Option<Vec<String>>,
    pub current_identity: FieldIdentity,
    pub current_declaration_path: Option<Vec<String>>,
}

impl From<FieldDuplicateIdentityError> for FieldDefSetBuildError {
    fn from(value: FieldDuplicateIdentityError) -> Self {
        Self::DuplicateIdentity(value)
    }
}

impl From<FieldDuplicateProcessingPathError> for FieldDefSetBuildError {
    fn from(value: FieldDuplicateProcessingPathError) -> Self {
        Self::DuplicateProcessingPath(Box::new(value))
    }
}

impl From<BuildError> for FieldDefSetBuildError {
    fn from(value: BuildError) -> Self {
        Self::Field(FieldDefBuildFailure {
            declaration_path: None,
            error: value,
        })
    }
}

impl fmt::Display for FieldDefSetBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Field(error) => write!(formatter, "field def build failed: {error}"),
            Self::DuplicateIdentity(error) => {
                write!(formatter, "field def identity is duplicated: {error}")
            }
            Self::ProcessingInputKindConflict {
                processing_id,
                previous,
                current,
            } => write!(
                formatter,
                "processing {processing_id} has conflicting input kinds: {previous:?} and {current:?}"
            ),
            Self::DuplicateProcessingPath(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for FieldDefSetBuildError {}

impl fmt::Display for FieldDuplicateProcessingPathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "processing {} path {} is declared by {} at {} and {} at {}",
            self.processing_id,
            self.path.raw_segments().join("."),
            self.current_identity.as_str(),
            display_declaration_path(&self.current_declaration_path),
            self.previous_identity.as_str(),
            display_declaration_path(&self.previous_declaration_path),
        )
    }
}

impl std::error::Error for FieldDuplicateProcessingPathError {}

fn display_declaration_path(path: &Option<Vec<String>>) -> String {
    path.as_ref()
        .map(|path| path.join("."))
        .unwrap_or_else(|| "<unknown>".to_owned())
}

#[derive(Debug, PartialEq)]
pub enum FieldExtractionError {
    UnknownProcessing {
        processing_id: ProcessingId,
    },
    InputKindMismatch {
        processing_id: ProcessingId,
        expected: ProcessingInputKind,
        actual: ProcessingInputKind,
    },
    Validation(super::FieldValidationErrors),
}

impl fmt::Display for FieldExtractionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownProcessing { processing_id } => {
                write!(formatter, "processing {processing_id} is not registered")
            }
            Self::InputKindMismatch {
                processing_id,
                expected,
                actual,
            } => write!(
                formatter,
                "processing {processing_id} expects {expected:?} input, got {actual:?}"
            ),
            Self::Validation(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for FieldExtractionError {}
