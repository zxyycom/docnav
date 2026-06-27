use std::fmt;

use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DetailFieldType {
    String,
    StringArray,
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
    allow_extra_fields: bool,
}

impl DiagnosticDetailsRule {
    pub const fn exact(fields: &'static [DetailFieldRule]) -> Self {
        Self {
            fields,
            allow_extra_fields: false,
        }
    }

    pub const fn fields(self) -> &'static [DetailFieldRule] {
        self.fields
    }

    pub const fn allows_extra_fields(self) -> bool {
        self.allow_extra_fields
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

        if !self.allow_extra_fields {
            for field in object.keys() {
                if !self.fields.iter().any(|rule| rule.name == field) {
                    return Err(DiagnosticDetailsError::ExtraField {
                        field: field.clone(),
                    });
                }
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
        candidates: Value,
    },
    FormatAmbiguous {
        path: String,
        candidates: Value,
    },
    CapabilityAdapter {
        capability: String,
        adapter_id: String,
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
        exit_code: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        stderr: Option<String>,
    },
    Internal {
        error_id: String,
    },
    CliArgv {
        tokens: Vec<String>,
    },
    AdapterCandidate {
        adapter_id: String,
        stage: String,
        code: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        preselected: Option<bool>,
    },
    AdapterConfigSource {
        source_level: String,
        path_origin: String,
        path: String,
        reason_code: String,
    },
    Boundary {
        reason: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    Object(Map<String, Value>),
}

impl DiagnosticDetails {
    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or_else(|_| Value::Object(Map::new()))
    }
}
