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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormatUnknownDetails {
    pub path: String,
    pub reason: String,
    pub candidates: Vec<FormatCandidateDetails>,
}

impl FormatUnknownDetails {
    pub fn new(
        path: impl Into<String>,
        reason: impl Into<String>,
        candidates: Vec<FormatCandidateDetails>,
    ) -> Self {
        Self {
            path: path.into(),
            reason: reason.into(),
            candidates,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormatAmbiguousDetails {
    pub path: String,
    pub candidates: Vec<FormatCandidateDetails>,
}

impl FormatAmbiguousDetails {
    pub fn new(path: impl Into<String>, candidates: Vec<FormatCandidateDetails>) -> Self {
        Self {
            path: path.into(),
            candidates,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FormatCandidateDetails {
    pub adapter_id: String,
    pub stage: String,
    pub reason: String,
}

impl FormatCandidateDetails {
    pub fn new(
        adapter_id: impl Into<String>,
        stage: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            adapter_id: adapter_id.into(),
            stage: stage.into(),
            reason: reason.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CapabilityAdapterDetails {
    pub capability: String,
    pub adapter_id: String,
}

impl CapabilityAdapterDetails {
    pub fn new(capability: impl Into<String>, adapter_id: impl Into<String>) -> Self {
        Self {
            capability: capability.into(),
            adapter_id: adapter_id.into(),
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdapterReasonDetails {
    pub adapter_id: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
}

impl AdapterReasonDetails {
    pub fn new(adapter_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            adapter_id: adapter_id.into(),
            reason: reason.into(),
            selection_source: None,
            stage: None,
        }
    }

    pub fn with_selection_context(
        mut self,
        selection_source: impl Into<String>,
        stage: impl Into<String>,
    ) -> Self {
        self.selection_source = Some(selection_source.into());
        self.stage = Some(stage.into());
        self
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
