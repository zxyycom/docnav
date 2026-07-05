use std::fmt;

use serde::Serialize;
use serde_json::Value;

mod conversion;
mod payload;

pub use payload::{
    AdapterConfigSourceDetails, AdapterReasonDetails, BoundaryDetails, DiagnosticDetailsPayload,
    FieldReasonDetails, FormatAmbiguousDetails, FormatCandidateDetails, FormatUnknownDetails,
    InternalDetails, PathDetails, PathEncodingDetails, PathReasonDetails, RefCandidateCountDetails,
    RefDetails, RefReasonDetails,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DetailFieldType {
    String,
    StringArray,
    ObjectArray,
    Boolean,
    U32,
    I32,
    Object,
    Any,
}

impl DetailFieldType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::StringArray => "array<string>",
            Self::ObjectArray => "array<object>",
            Self::Boolean => "boolean",
            Self::U32 => "u32",
            Self::I32 => "i32",
            Self::Object => "object",
            Self::Any => "any",
        }
    }

    fn matches(self, value: &Value) -> bool {
        match self {
            Self::String => value.is_string(),
            Self::StringArray => value
                .as_array()
                .is_some_and(|items| items.iter().all(Value::is_string)),
            Self::ObjectArray => value
                .as_array()
                .is_some_and(|items| items.iter().all(Value::is_object)),
            Self::Boolean => value.is_boolean(),
            Self::U32 => value.as_u64().is_some_and(|value| value <= u32::MAX as u64),
            Self::I32 => value
                .as_i64()
                .is_some_and(|value| value >= i32::MIN as i64 && value <= i32::MAX as i64),
            Self::Object => value.is_object(),
            Self::Any => true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DetailFieldRule {
    pub name: &'static str,
    pub kind: DetailFieldType,
    pub required: bool,
}

impl DetailFieldRule {
    pub const fn required(name: &'static str, kind: DetailFieldType) -> Self {
        Self {
            name,
            kind,
            required: true,
        }
    }

    pub const fn optional(name: &'static str, kind: DetailFieldType) -> Self {
        Self {
            name,
            kind,
            required: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticDetailsRule {
    fields: &'static [DetailFieldRule],
}

impl DiagnosticDetailsRule {
    pub const fn exact(fields: &'static [DetailFieldRule]) -> Self {
        Self { fields }
    }

    pub const fn fields(self) -> &'static [DetailFieldRule] {
        self.fields
    }

    pub fn required_field_names(self) -> impl Iterator<Item = &'static str> {
        self.fields
            .iter()
            .filter_map(|field| field.required.then_some(field.name))
    }

    pub fn validate_value(self, details: &Value) -> Result<(), DiagnosticDetailsError> {
        let object = details
            .as_object()
            .ok_or(DiagnosticDetailsError::NotAnObject)?;

        for field in self.fields {
            match object.get(field.name) {
                Some(value) if field.kind.matches(value) => {}
                Some(_) => {
                    return Err(DiagnosticDetailsError::WrongType {
                        field: field.name,
                        expected: field.kind,
                    });
                }
                None if field.required => {
                    return Err(DiagnosticDetailsError::MissingField { field: field.name });
                }
                None => {}
            }
        }

        for field in object.keys() {
            if !self.fields.iter().any(|rule| rule.name == field) {
                return Err(DiagnosticDetailsError::ExtraField {
                    field: field.clone(),
                });
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiagnosticDetailsError {
    NotAnObject,
    MissingField {
        field: &'static str,
    },
    WrongType {
        field: &'static str,
        expected: DetailFieldType,
    },
    ExtraField {
        field: String,
    },
}

impl fmt::Display for DiagnosticDetailsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAnObject => formatter.write_str("diagnostic details must be an object"),
            Self::MissingField { field } => write!(formatter, "diagnostic details missing {field}"),
            Self::WrongType { field, expected } => {
                write!(
                    formatter,
                    "diagnostic details.{field} must be {}",
                    expected.as_str()
                )
            }
            Self::ExtraField { field } => {
                write!(formatter, "diagnostic details contains unexpected {field}")
            }
        }
    }
}

impl std::error::Error for DiagnosticDetailsError {}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum DiagnosticDetails {
    FieldReason {
        field: String,
        reason: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        received: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        accepted: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        field_issues: Option<Vec<Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        config_issues: Option<Vec<AdapterConfigSourceDetails>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        typed_validation_failures: Option<Vec<Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        option_issues: Option<Vec<Value>>,
    },
    Path {
        path: String,
    },
    PathReason {
        path: String,
        reason: String,
    },
    PathEncoding {
        path: String,
        encoding: String,
    },
    FormatUnknown {
        path: String,
        reason: String,
        candidates: Vec<FormatCandidateDetails>,
        #[serde(skip_serializing_if = "Option::is_none")]
        candidate_failures: Option<Vec<FormatCandidateDetails>>,
    },
    FormatAmbiguous {
        path: String,
        candidates: Vec<FormatCandidateDetails>,
        #[serde(skip_serializing_if = "Option::is_none")]
        candidate_failures: Option<Vec<FormatCandidateDetails>>,
    },
    Ref {
        #[serde(rename = "ref")]
        ref_id: String,
    },
    RefCandidateCount {
        #[serde(rename = "ref")]
        ref_id: String,
        candidate_count: u32,
    },
    RefReason {
        #[serde(rename = "ref")]
        ref_id: String,
        reason: String,
    },
    AdapterReason {
        adapter_id: String,
        reason: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        selection_source: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        stage: Option<String>,
    },
    Internal {
        error_id: String,
    },
    Boundary {
        reason: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
}

impl DiagnosticDetails {
    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).expect("diagnostic details variants serialize to JSON values")
    }
}
