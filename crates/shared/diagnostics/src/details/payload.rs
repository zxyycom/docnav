use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::DiagnosticDetails;

pub trait DiagnosticDetailsPayload:
    Clone + Serialize + serde::de::DeserializeOwned + Into<DiagnosticDetails>
{
}

impl<T> DiagnosticDetailsPayload for T where
    T: Clone + Serialize + serde::de::DeserializeOwned + Into<DiagnosticDetails>
{
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldReasonDetails {
    pub field: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub received: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_issues: Option<Vec<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_issues: Option<Vec<AdapterConfigSourceDetails>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typed_validation_failures: Option<Vec<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option_issues: Option<Vec<Value>>,
}

impl FieldReasonDetails {
    pub fn new(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            reason: reason.into(),
            path: None,
            received: None,
            accepted: None,
            field_issues: None,
            config_issues: None,
            typed_validation_failures: None,
            option_issues: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PathDetails {
    pub path: String,
}

impl PathDetails {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PathReasonDetails {
    pub path: String,
    pub reason: String,
}

impl PathReasonDetails {
    pub fn new(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            reason: reason.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PathEncodingDetails {
    pub path: String,
    pub encoding: String,
}

impl PathEncodingDetails {
    pub fn new(path: impl Into<String>, encoding: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            encoding: encoding.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DocumentContentInvalidReason {
    #[serde(rename = "JSON_SYNTAX_INVALID")]
    JsonSyntaxInvalid,
    #[serde(rename = "JSON_TRAILING_INPUT")]
    JsonTrailingInput,
    #[serde(rename = "JSON_DUPLICATE_MEMBER")]
    JsonDuplicateMember,
    #[serde(rename = "JSON_MAXIMUM_DEPTH_EXCEEDED")]
    JsonMaximumDepthExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DocumentContentInvalidDetails {
    pub path: String,
    pub reason: DocumentContentInvalidReason,
}

impl DocumentContentInvalidDetails {
    pub fn new(path: impl Into<String>, reason: DocumentContentInvalidReason) -> Self {
        Self {
            path: path.into(),
            reason,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum FormatUnknownReason {
    #[serde(rename = "FORMAT_NOT_RECOGNIZED")]
    FormatNotRecognized,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormatUnknownDetails {
    pub path: String,
    pub reason: FormatUnknownReason,
    pub candidates: [Value; 0],
}

impl FormatUnknownDetails {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            reason: FormatUnknownReason::FormatNotRecognized,
            candidates: [],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RefDetails {
    #[serde(rename = "ref")]
    pub ref_id: String,
}

impl RefDetails {
    pub fn new(ref_id: impl Into<String>) -> Self {
        Self {
            ref_id: ref_id.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RefCandidateCountDetails {
    #[serde(rename = "ref")]
    pub ref_id: String,
    pub candidate_count: u32,
}

impl RefCandidateCountDetails {
    pub fn new(ref_id: impl Into<String>, candidate_count: u32) -> Self {
        Self {
            ref_id: ref_id.into(),
            candidate_count,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RefReasonDetails {
    #[serde(rename = "ref")]
    pub ref_id: String,
    pub reason: String,
}

impl RefReasonDetails {
    pub fn new(ref_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            ref_id: ref_id.into(),
            reason: reason.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum AdapterUnavailableReason {
    #[serde(rename = "ADAPTER_NOT_FOUND")]
    AdapterNotFound,
}

impl AdapterUnavailableReason {
    const fn as_str(self) -> &'static str {
        match self {
            Self::AdapterNotFound => "ADAPTER_NOT_FOUND",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum AdapterUnavailableStage {
    #[serde(rename = "resolve")]
    Resolve,
}

impl AdapterUnavailableStage {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Resolve => "resolve",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterUnavailableDetails {
    pub adapter_id: String,
    reason: AdapterUnavailableReason,
    pub selection_source: String,
    stage: AdapterUnavailableStage,
}

impl AdapterUnavailableDetails {
    pub fn new(adapter_id: impl Into<String>, selection_source: impl Into<String>) -> Self {
        Self {
            adapter_id: adapter_id.into(),
            reason: AdapterUnavailableReason::AdapterNotFound,
            selection_source: selection_source.into(),
            stage: AdapterUnavailableStage::Resolve,
        }
    }

    pub(super) fn into_parts(self) -> (String, String, String, String) {
        (
            self.adapter_id,
            self.reason.as_str().to_owned(),
            self.selection_source,
            self.stage.as_str().to_owned(),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InternalDetails {
    pub error_id: String,
}

impl InternalDetails {
    pub fn new(error_id: impl Into<String>) -> Self {
        Self {
            error_id: error_id.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdapterConfigSourceDetails {
    pub source_level: String,
    pub path_origin: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    pub reason_code: String,
}

impl AdapterConfigSourceDetails {
    pub fn new(
        source_level: impl Into<String>,
        path_origin: impl Into<String>,
        path: impl Into<String>,
        reason_code: impl Into<String>,
    ) -> Self {
        Self {
            source_level: source_level.into(),
            path_origin: path_origin.into(),
            path: path.into(),
            field: None,
            reason_code: reason_code.into(),
        }
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoundaryDetails {
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl BoundaryDetails {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
            label: None,
        }
    }
}
