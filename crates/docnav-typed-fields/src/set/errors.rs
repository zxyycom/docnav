use std::fmt;

use crate::extraction::{ExtractionInputKind, ExtractionStrategyId};
use crate::metadata::{BuildError, FieldDuplicateIdentityError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[doc(hidden)]
pub struct ExpectedFieldShape {
    pub required: bool,
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
    ExtractionInputKindConflict {
        strategy_id: ExtractionStrategyId,
        previous: ExtractionInputKind,
        current: ExtractionInputKind,
    },
}

impl From<FieldDuplicateIdentityError> for FieldDefSetBuildError {
    fn from(value: FieldDuplicateIdentityError) -> Self {
        Self::DuplicateIdentity(value)
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
            Self::ExtractionInputKindConflict {
                strategy_id,
                previous,
                current,
            } => write!(
                formatter,
                "extraction strategy {strategy_id} has conflicting input kinds: {previous:?} and {current:?}"
            ),
        }
    }
}

impl std::error::Error for FieldDefSetBuildError {}

#[derive(Debug, PartialEq)]
pub enum FieldExtractionError {
    UnknownStrategy {
        strategy_id: ExtractionStrategyId,
    },
    InputKindMismatch {
        strategy_id: ExtractionStrategyId,
        expected: ExtractionInputKind,
        actual: ExtractionInputKind,
    },
    Validation(super::FieldValidationErrors),
}

impl fmt::Display for FieldExtractionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownStrategy { strategy_id } => {
                write!(
                    formatter,
                    "extraction strategy {strategy_id} is not registered"
                )
            }
            Self::InputKindMismatch {
                strategy_id,
                expected,
                actual,
            } => write!(
                formatter,
                "extraction strategy {strategy_id} expects {expected:?} input, got {actual:?}"
            ),
            Self::Validation(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for FieldExtractionError {}
