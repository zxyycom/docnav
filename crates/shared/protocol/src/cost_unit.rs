use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CostUnit {
    Lines,
    Bytes,
    Tokens,
}

impl CostUnit {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lines => "lines",
            Self::Bytes => "bytes",
            Self::Tokens => "tokens",
        }
    }
}

impl fmt::Display for CostUnit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for CostUnit {
    type Err = CostUnitParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "lines" => Ok(Self::Lines),
            "bytes" => Ok(Self::Bytes),
            "tokens" => Ok(Self::Tokens),
            _ => Err(CostUnitParseError(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CostUnitParseError(String);

impl fmt::Display for CostUnitParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid cost unit: {}", self.0)
    }
}

impl std::error::Error for CostUnitParseError {}
