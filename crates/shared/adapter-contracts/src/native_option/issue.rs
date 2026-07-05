use docnav_diagnostics::AdapterConfigSourceDetails;
use serde_json::{json, Map, Value};

#[derive(Clone, Debug, PartialEq)]
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
    pub config_source: Option<AdapterConfigSourceDetails>,
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
