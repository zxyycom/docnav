use std::path::Path;

use docnav_protocol::{OperationResult, PositiveInteger};
use serde_json::{json, Value};

use super::hash::{sha256_hex, HASH_ALGORITHM};
use super::summary::failure_summary;
use super::time::Timestamp;
use super::writer::write_content_file;
use super::{DocumentLogContext, SCHEMA_VERSION};

#[derive(Clone, Debug)]
pub(super) struct ContentReference<'a> {
    pub(super) value: Value,
    pub(super) bytes: &'a [u8],
}

impl ContentReference<'_> {
    pub(super) fn content_hash(&self) -> &str {
        self.value["content_hash"]
            .as_str()
            .expect("content reference has hash")
    }
}

pub(super) fn content_reference_for_result(
    result: &OperationResult,
) -> Option<ContentReference<'_>> {
    match result {
        OperationResult::Read(result) => Some(content_reference(
            &result.content,
            Some(&result.content_type),
        )),
        OperationResult::Outline(result) => result
            .as_unstructured()
            .map(|result| content_reference(&result.content, Some(&result.content_type))),
        OperationResult::Find(_) | OperationResult::Info(_) => None,
    }
}

pub(super) fn result_page(result: &OperationResult) -> Option<PositiveInteger> {
    match result {
        OperationResult::Outline(result) => result.as_structured().and_then(|result| result.page),
        OperationResult::Read(result) => result.page,
        OperationResult::Find(result) => result.page,
        OperationResult::Info(_) => None,
    }
}

pub(super) fn capture_content_event(
    root: &Path,
    correlation_id: &str,
    context: &DocumentLogContext,
    request_id: Option<&str>,
    content: &ContentReference,
) -> Value {
    let now = Timestamp::now();
    let relative_path = format!("{}/sha256-{}.content", now.date, content.content_hash());
    let absolute_path = root.join(relative_path.replace('/', std::path::MAIN_SEPARATOR_STR));
    let result = write_content_file(&absolute_path, content.bytes);
    let event_name = if result.is_ok() {
        "content_captured"
    } else {
        "content_capture_failed"
    };
    let mut event = json!({
        "schema_version": SCHEMA_VERSION,
        "timestamp": now.full,
        "event": event_name,
        "correlation_id": correlation_id,
        "operation": context.operation.as_str(),
        "captured_at": now.full,
        "content": content.value,
        "relative_path": relative_path,
    });
    if let Some(request_id) = request_id {
        event["request_id"] = json!(request_id);
    }
    if let Err(error) = result {
        event["failure"] = failure_summary(
            "operation",
            None,
            format!("content capture failed: {error}"),
        );
    }
    event
}

fn content_reference<'a>(content: &'a str, content_type: Option<&str>) -> ContentReference<'a> {
    let bytes = content.as_bytes();
    let mut value = json!({
        "hash_algorithm": HASH_ALGORITHM,
        "content_hash": sha256_hex(bytes),
        "size_bytes": bytes.len(),
        "size_chars": content.chars().count(),
        "summary": {
            "kind": "length_hash",
            "length": content.chars().count(),
            "hash_algorithm": HASH_ALGORITHM,
            "value_hash": sha256_hex(bytes),
        }
    });
    if let Some(content_type) = content_type {
        value["content_type"] = json!(content_type);
    }
    ContentReference { value, bytes }
}
