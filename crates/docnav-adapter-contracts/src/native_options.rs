use docnav_protocol::Operation;
use serde_json::{json, Map, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeOptionSpec {
    pub identity: &'static str,
    pub owner: &'static str,
    pub namespace: &'static str,
    pub key: &'static str,
    pub operations: &'static [Operation],
    pub value: NativeOptionValueSpec,
}

impl NativeOptionSpec {
    pub fn applies_to(self, operation: Operation) -> bool {
        self.operations.contains(&operation)
    }

    pub fn accepts_integer(self, value: i64) -> bool {
        match self.value {
            NativeOptionValueSpec::Integer { min, max } => (min..=max).contains(&value),
            NativeOptionValueSpec::String
            | NativeOptionValueSpec::Boolean
            | NativeOptionValueSpec::Json => false,
        }
    }

    pub fn expected_value_description(self) -> String {
        match self.value {
            NativeOptionValueSpec::Integer { min, max } => {
                format!("an integer from {min} to {max}")
            }
            NativeOptionValueSpec::String => "a string".to_owned(),
            NativeOptionValueSpec::Boolean => "a boolean".to_owned(),
            NativeOptionValueSpec::Json => "a JSON value".to_owned(),
        }
    }

    pub fn value_kind(self) -> NativeOptionValueKind {
        match self.value {
            NativeOptionValueSpec::Integer { .. } => NativeOptionValueKind::Integer,
            NativeOptionValueSpec::String => NativeOptionValueKind::String,
            NativeOptionValueSpec::Boolean => NativeOptionValueKind::Boolean,
            NativeOptionValueSpec::Json => NativeOptionValueKind::Json,
        }
    }

    pub fn config_key(self) -> String {
        format!("{}.{}", self.namespace, self.key)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeOptionValueSpec {
    Integer { min: i64, max: i64 },
    String,
    Boolean,
    Json,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeOptionValueKind {
    Integer,
    String,
    Boolean,
    Json,
}

impl NativeOptionValueKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Integer => "integer",
            Self::String => "string",
            Self::Boolean => "boolean",
            Self::Json => "json",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeOptionIssue {
    pub owner: String,
    pub namespace: String,
    pub key: String,
    pub source: String,
    pub reason_code: String,
    pub field: String,
    pub received: Option<String>,
    pub expected: Option<String>,
    pub type_variant: Option<String>,
}

impl NativeOptionIssue {
    pub fn into_json(self) -> Value {
        let mut issue = Map::new();
        issue.insert("owner".to_owned(), json!(self.owner));
        issue.insert("namespace".to_owned(), json!(self.namespace));
        issue.insert("key".to_owned(), json!(self.key));
        issue.insert("source".to_owned(), json!(self.source));
        issue.insert("reason_code".to_owned(), json!(self.reason_code));
        issue.insert("location".to_owned(), json!({ "field": self.field }));
        if let Some(type_variant) = self.type_variant {
            issue.insert("type_variant".to_owned(), json!(type_variant));
        }
        if let Some(received) = self.received {
            issue.insert("received".to_owned(), json!(received));
        }
        if let Some(expected) = self.expected {
            issue.insert("expected".to_owned(), json!(expected));
        }
        Value::Object(issue)
    }
}
