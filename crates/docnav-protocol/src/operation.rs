use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::constants::operation_names;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Outline,
    Read,
    Find,
    Info,
}

impl Operation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Outline => operation_names::OUTLINE,
            Self::Read => operation_names::READ,
            Self::Find => operation_names::FIND,
            Self::Info => operation_names::INFO,
        }
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Operation {
    type Err = OperationParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            operation_names::OUTLINE => Ok(Self::Outline),
            operation_names::READ => Ok(Self::Read),
            operation_names::FIND => Ok(Self::Find),
            operation_names::INFO => Ok(Self::Info),
            _ => Err(OperationParseError(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationParseError(String);

impl fmt::Display for OperationParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid operation: {}", self.0)
    }
}

impl std::error::Error for OperationParseError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PagedOperation {
    Outline,
    Read,
    Find,
}

impl PagedOperation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Outline => operation_names::OUTLINE,
            Self::Read => operation_names::READ,
            Self::Find => operation_names::FIND,
        }
    }
}

impl fmt::Display for PagedOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<PagedOperation> for Operation {
    fn from(operation: PagedOperation) -> Self {
        match operation {
            PagedOperation::Outline => Self::Outline,
            PagedOperation::Read => Self::Read,
            PagedOperation::Find => Self::Find,
        }
    }
}
