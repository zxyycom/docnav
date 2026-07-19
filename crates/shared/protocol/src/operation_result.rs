use serde::{Deserialize, Serialize};

use crate::{Metadata, Operation, PositiveInteger};

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(rename = "ref")]
    pub ref_id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<Cost>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    pub line_start: PositiveInteger,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_end: Option<PositiveInteger>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cost {
    pub measurements: Vec<Measurement>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Measurement {
    pub unit: String,
    pub value: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OutlineResult {
    Structured(StructuredOutlineResult),
    Unstructured(UnstructuredOutlineResult),
}

impl OutlineResult {
    pub fn structured(entries: Vec<Entry>, page: Option<PositiveInteger>) -> Self {
        Self::Structured(StructuredOutlineResult {
            entries,
            page,
            auto_read: None,
        })
    }

    pub fn unstructured(
        reason: UnstructuredOutlineReason,
        content: impl Into<String>,
        content_type: impl Into<String>,
        cost: Cost,
    ) -> Self {
        Self::Unstructured(UnstructuredOutlineResult {
            reason,
            content: content.into(),
            content_type: content_type.into(),
            cost,
        })
    }

    pub const fn as_structured(&self) -> Option<&StructuredOutlineResult> {
        match self {
            Self::Structured(result) => Some(result),
            Self::Unstructured(_) => None,
        }
    }

    pub fn into_structured(self) -> Option<StructuredOutlineResult> {
        match self {
            Self::Structured(result) => Some(result),
            Self::Unstructured(_) => None,
        }
    }

    pub const fn as_unstructured(&self) -> Option<&UnstructuredOutlineResult> {
        match self {
            Self::Structured(_) => None,
            Self::Unstructured(result) => Some(result),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuredOutlineResult {
    pub entries: Vec<Entry>,
    pub page: Option<PositiveInteger>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_read: Option<AutoReadResult>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnstructuredOutlineResult {
    pub reason: UnstructuredOutlineReason,
    pub content: String,
    pub content_type: String,
    pub cost: Cost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnstructuredOutlineReason {
    PathRule,
    CostThreshold,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadResult {
    #[serde(rename = "ref")]
    pub ref_id: String,
    pub content: String,
    pub content_type: String,
    pub cost: Cost,
    pub page: Option<PositiveInteger>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AutoReadResult {
    pub reason: AutoReadReason,
    pub read: ReadResult,
}

impl AutoReadResult {
    pub fn unique_ref(read: ReadResult) -> Self {
        Self {
            reason: AutoReadReason::UniqueRef,
            read,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoReadReason {
    UniqueRef,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindResult {
    pub matches: Vec<Entry>,
    pub page: Option<PositiveInteger>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_read: Option<AutoReadResult>,
}

impl FindResult {
    pub fn new(matches: Vec<Entry>, page: Option<PositiveInteger>) -> Self {
        Self {
            matches,
            page,
            auto_read: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<InfoDocument>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter: Option<InfoAdapter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoDocument {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<Measurement>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoAdapter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}
