use std::fmt;

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
    DeclarationShape(FieldDeclarationShapeError),
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
            Self::DeclarationShape(error) => {
                write!(
                    formatter,
                    "field declaration presence does not match field metadata: {error}"
                )
            }
        }
    }
}

impl std::error::Error for FieldDefSetBuildError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldDeclarationShapeError {
    pub declaration_path: Option<Vec<String>>,
    pub expected: ExpectedFieldShape,
    pub actual: ExpectedFieldShape,
}

impl fmt::Display for FieldDeclarationShapeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.declaration_path {
            Some(path) => write!(
                formatter,
                "{} expected {:?}, actual {:?}",
                path.join("."),
                self.expected,
                self.actual
            ),
            None => write!(
                formatter,
                "expected {:?}, actual {:?}",
                self.expected, self.actual
            ),
        }
    }
}

impl std::error::Error for FieldDeclarationShapeError {}
