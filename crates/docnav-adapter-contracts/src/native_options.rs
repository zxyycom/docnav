use docnav_protocol::Operation;
use serde_json::{json, Map, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeOptionSpec {
    pub identity: &'static str,
    pub owner: &'static str,
    pub namespace: &'static str,
    pub key: &'static str,
    pub operations: &'static [Operation],
    pub cli_flag: Option<&'static str>,
    pub value: NativeOptionValueSpec,
    pub default: Option<NativeOptionDefaultValue>,
}

impl NativeOptionSpec {
    pub const fn builder(identity: &'static str) -> NativeOptionSpecBuilder {
        NativeOptionSpecBuilder {
            identity,
            owner: "",
            namespace: "",
            key: "",
            operations: &[],
            cli_flag: None,
            value: NativeOptionValueSpec::Json,
            default: None,
        }
    }

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
                format!("integer in range {min}..{max}")
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

    pub fn cli_arg_id(self) -> Option<&'static str> {
        self.cli_flag
            .map(|flag| flag.strip_prefix("--").unwrap_or(flag))
    }

    pub fn default_value(self) -> Option<Value> {
        self.default.map(NativeOptionDefaultValue::into_json)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeOptionSpecBuilder {
    identity: &'static str,
    owner: &'static str,
    namespace: &'static str,
    key: &'static str,
    operations: &'static [Operation],
    cli_flag: Option<&'static str>,
    value: NativeOptionValueSpec,
    default: Option<NativeOptionDefaultValue>,
}

impl NativeOptionSpecBuilder {
    pub const fn owner(mut self, owner: &'static str) -> Self {
        self.owner = owner;
        self
    }

    pub const fn namespace(mut self, namespace: &'static str) -> Self {
        self.namespace = namespace;
        self
    }

    pub const fn key(mut self, key: &'static str) -> Self {
        self.key = key;
        self
    }

    pub const fn operations(mut self, operations: &'static [Operation]) -> Self {
        self.operations = operations;
        self
    }

    pub const fn cli_flag(mut self, cli_flag: &'static str) -> Self {
        self.cli_flag = Some(cli_flag);
        self
    }

    pub const fn value(mut self, value: NativeOptionValueSpec) -> Self {
        self.value = value;
        self
    }

    pub const fn default_integer(mut self, value: i64) -> Self {
        self.default = Some(NativeOptionDefaultValue::Integer(value));
        self
    }

    pub const fn default_string(mut self, value: &'static str) -> Self {
        self.default = Some(NativeOptionDefaultValue::String(value));
        self
    }

    pub const fn default_boolean(mut self, value: bool) -> Self {
        self.default = Some(NativeOptionDefaultValue::Boolean(value));
        self
    }

    pub const fn build(self) -> NativeOptionSpec {
        NativeOptionSpec {
            identity: self.identity,
            owner: self.owner,
            namespace: self.namespace,
            key: self.key,
            operations: self.operations,
            cli_flag: self.cli_flag,
            value: self.value,
            default: self.default,
        }
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
pub enum NativeOptionDefaultValue {
    Integer(i64),
    String(&'static str),
    Boolean(bool),
}

impl NativeOptionDefaultValue {
    pub fn into_json(self) -> Value {
        match self {
            Self::Integer(value) => json!(value),
            Self::String(value) => json!(value),
            Self::Boolean(value) => json!(value),
        }
    }
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
