use serde::{Deserialize, Serialize};

use crate::{Operation, PositiveInteger};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OperationResult {
    Outline(OutlineResult),
    Read(ReadResult),
    Find(FindResult),
    Info(InfoResult),
}

impl OperationResult {
    pub const fn operation(&self) -> Operation {
        match self {
            Self::Outline(_) => Operation::Outline,
            Self::Read(_) => Operation::Read,
            Self::Find(_) => Operation::Find,
            Self::Info(_) => Operation::Info,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(rename = "ref")]
    pub ref_id: String,
    pub display: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutlineResult {
    pub entries: Vec<Entry>,
    pub page: Option<PositiveInteger>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadResult {
    #[serde(rename = "ref")]
    pub ref_id: String,
    pub content: String,
    pub content_type: String,
    pub cost: String,
    pub page: Option<PositiveInteger>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindResult {
    pub matches: Vec<Entry>,
    pub page: Option<PositiveInteger>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoResult {
    pub display: String,
    pub capabilities: Vec<Operation>,
}
