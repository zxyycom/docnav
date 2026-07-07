use std::time::Duration;

use docnav_protocol::ProtocolResponse;
use serde_json::{json, Value};

use super::time::{duration_ms, Timestamp};
use super::{DocumentLogContext, SCHEMA_VERSION};

pub(super) fn operation_event_base(
    context: &DocumentLogContext,
    event: OperationEvent<'_>,
) -> ValueBuilder {
    let mut value = json!({
        "schema_version": SCHEMA_VERSION,
        "timestamp": Timestamp::now().full,
        "event": event.name,
        "correlation_id": event.correlation_id,
        "operation": context.operation.as_str(),
        "status": event.status,
        "duration_ms": duration_ms(event.duration),
        "document": context.document,
    });
    if let Some(request_id) = event.request_id {
        value["request_id"] = json!(request_id);
    }
    if let Some(adapter_id) = event.adapter_id {
        value["adapter_id"] = json!(adapter_id);
    }
    if !context
        .arguments
        .as_object()
        .is_some_and(serde_json::Map::is_empty)
    {
        value["arguments"] = context.arguments.clone();
    }
    ValueBuilder(value)
}

pub(super) struct OperationEvent<'a> {
    pub(super) name: &'a str,
    pub(super) status: &'a str,
    pub(super) correlation_id: &'a str,
    pub(super) request_id: Option<&'a str>,
    pub(super) adapter_id: Option<&'a str>,
    pub(super) duration: Duration,
}

pub(super) fn response_size_bytes(response: &ProtocolResponse) -> usize {
    serde_json::to_vec(response).map_or(0, |bytes| bytes.len())
}

pub(super) struct ValueBuilder(Value);

impl ValueBuilder {
    pub(super) fn with_field(mut self, key: &str, value: Value) -> Value {
        self.0[key] = value;
        self.0
    }
}
