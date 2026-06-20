use serde_json::Value;

use crate::constants::fields;
use crate::Operation;

pub fn extract_request_context(input: &str) -> PartialRequestContext {
    let Ok(value) = serde_json::from_str::<Value>(input) else {
        return PartialRequestContext::default();
    };

    extract_request_context_from_value(&value)
}

pub fn extract_request_context_from_value(value: &Value) -> PartialRequestContext {
    PartialRequestContext {
        protocol_version: value
            .get(fields::PROTOCOL_VERSION)
            .and_then(Value::as_str)
            .map(str::to_owned),
        request_id: value
            .get(fields::REQUEST_ID)
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_owned),
        operation: value
            .get(fields::OPERATION)
            .and_then(Value::as_str)
            .and_then(|value| value.parse::<Operation>().ok()),
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PartialRequestContext {
    pub protocol_version: Option<String>,
    pub request_id: Option<String>,
    pub operation: Option<Operation>,
}
