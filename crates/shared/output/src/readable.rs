use std::io::Write;

use docnav_protocol::{
    protocol_error_default_guidance, Cost, Entry, InfoResult, Measurement, OperationResult,
    OutlineResult, ProtocolError,
};
use docnav_readable::{render_readable_view, to_readable_value, ReadableViewKind};
use serde_json::{json, Map, Value};

use crate::DocumentOutputError;

pub fn readable_payload(result: &OperationResult) -> Result<Value, DocumentOutputError> {
    let value = match result {
        OperationResult::Outline(result) => readable_outline_payload(result),
        OperationResult::Read(result) => json!({
            "ref": result.ref_id,
            "content": result.content,
            "content_type": result.content_type,
            "cost": cost_summary(&result.cost),
            "page": result.page,
        }),
        OperationResult::Find(result) => json!({
            "matches": result.matches.iter().map(readable_entry).collect::<Vec<_>>(),
            "page": result.page,
        }),
        OperationResult::Info(result) => json!({
            "display": info_display(result),
        }),
    };
    to_readable_value(&value).map_err(DocumentOutputError::ReadablePayload)
}

fn readable_outline_payload(result: &OutlineResult) -> Value {
    match result {
        OutlineResult::Structured(result) => json!({
            "kind": "structured",
            "entries": result.entries.iter().map(readable_entry).collect::<Vec<_>>(),
            "page": result.page,
        }),
        OutlineResult::Unstructured(result) => json!({
            "kind": "unstructured",
            "reason": result.reason,
            "content": result.content,
            "content_type": result.content_type,
            "cost": result.cost,
        }),
    }
}

pub fn view_kind_for_result(result: &OperationResult) -> ReadableViewKind {
    match result {
        OperationResult::Outline(OutlineResult::Structured(_)) => ReadableViewKind::Outline,
        OperationResult::Outline(OutlineResult::Unstructured(_)) => {
            ReadableViewKind::OutlineUnstructured
        }
        OperationResult::Read(_) => ReadableViewKind::Read,
        OperationResult::Find(_) => ReadableViewKind::Find,
        OperationResult::Info(_) => ReadableViewKind::Info,
    }
}

pub fn protocol_error_readable(error: &ProtocolError) -> Value {
    let mut value = Map::new();
    value.insert("code".to_owned(), json!(error.code().protocol_code()));
    value.insert("error".to_owned(), json!(error.message()));
    value.insert("owner".to_owned(), json!(error.owner()));
    if let Some(location) = error.location() {
        value.insert("location".to_owned(), location.clone());
    }
    if let Some(received) = error.received() {
        value.insert("received".to_owned(), received.clone());
    }
    if let Some(expected) = error.expected() {
        value.insert("expected".to_owned(), expected.clone());
    }
    value.insert("details".to_owned(), json!(error.details()));
    value.insert("guidance".to_owned(), json!(readable_guidance(error)));
    Value::Object(value)
}

fn readable_guidance(error: &ProtocolError) -> Vec<String> {
    error
        .guidance()
        .map(|guidance| guidance.to_vec())
        .unwrap_or_else(|| vec![protocol_error_default_guidance(error.code()).to_owned()])
}

fn readable_entry(entry: &Entry) -> Value {
    json!({
        "ref": entry.ref_id,
        "display": entry_display(entry),
    })
}

fn entry_display(entry: &Entry) -> String {
    match entry.kind.as_deref() {
        Some("heading") => heading_display(entry),
        Some("match") => match_display(entry),
        Some("document") => labeled_cost_display(entry),
        _ => generic_entry_display(entry),
    }
}

fn heading_display(entry: &Entry) -> String {
    let mut display = match heading_level(entry) {
        Some(level) => format!("H{} {}", level, entry.label),
        None => entry.label.clone(),
    };
    if let Some(cost) = entry.cost.as_ref().map(cost_summary) {
        display.push_str(" | ");
        display.push_str(&cost);
    }
    display
}

fn match_display(entry: &Entry) -> String {
    match entry
        .location
        .as_ref()
        .map(|location| location.line_start.get())
    {
        Some(line) => format!("L{}: {}", line, entry.label),
        None => entry.label.clone(),
    }
}

fn labeled_cost_display(entry: &Entry) -> String {
    match entry.cost.as_ref().map(cost_summary) {
        Some(cost) => format!("{} | {}", entry.label, cost),
        None => entry.label.clone(),
    }
}

fn generic_entry_display(entry: &Entry) -> String {
    if let Some(summary) = entry.summary.as_deref() {
        format!("{} | {}", entry.label, summary)
    } else if let Some(excerpt) = entry.excerpt.as_deref() {
        format!("{} | {}", entry.label, excerpt)
    } else if let Some(cost) = entry.cost.as_ref().map(cost_summary) {
        format!("{} | {}", entry.label, cost)
    } else {
        entry.label.clone()
    }
}

fn heading_level(entry: &Entry) -> Option<u64> {
    entry.metadata.as_ref()?.get("heading_level")?.as_u64()
}

fn info_display(info: &InfoResult) -> String {
    let mut parts = Vec::new();
    if let Some(format) = info
        .adapter
        .as_ref()
        .and_then(|adapter| adapter.format.as_deref())
    {
        parts.push(format_label(format));
    }
    if let Some(content_type) = info
        .document
        .as_ref()
        .and_then(|document| document.content_type.as_deref())
    {
        parts.push(content_type.to_owned());
    }
    if let Some(count) = info
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get("heading_count"))
        .and_then(Value::as_u64)
    {
        let label = if count == 1 { "heading" } else { "headings" };
        parts.push(format!("{count} {label}"));
    }
    if let Some(size) = info
        .document
        .as_ref()
        .and_then(|document| document.size.as_ref())
    {
        parts.push(measurement_summary(size));
    }
    if parts.is_empty() {
        "document info".to_owned()
    } else {
        parts.join(" | ")
    }
}

fn format_label(format: &str) -> String {
    match format {
        "markdown" => "Markdown".to_owned(),
        value => value.to_owned(),
    }
}

fn cost_summary(cost: &Cost) -> String {
    let summaries = cost
        .measurements
        .iter()
        .map(cost_measurement_summary)
        .collect::<Vec<_>>();
    if summaries.is_empty() {
        "cost unavailable".to_owned()
    } else {
        summaries.join(" | ")
    }
}

fn cost_measurement_summary(measurement: &Measurement) -> String {
    match measurement.unit.as_str() {
        "byte" | "bytes" => format!("{:.1} KB", measurement.value as f64 / 1024.0),
        _ => measurement_summary(measurement),
    }
}

fn measurement_summary(measurement: &Measurement) -> String {
    match measurement.unit.as_str() {
        "line" | "lines" => {
            let label = if measurement.value == 1 {
                "line"
            } else {
                "lines"
            };
            format!("{} {}", measurement.value, label)
        }
        "byte" | "bytes" => {
            if measurement.value < 1024 {
                format!("{} B", measurement.value)
            } else {
                format!("{:.1} KB", measurement.value as f64 / 1024.0)
            }
        }
        unit => format!("{} {}", measurement.value, unit),
    }
}

pub(crate) fn write_readable_view_value<W: Write>(
    value: Value,
    kind: ReadableViewKind,
    stdout: &mut W,
) -> Result<(), DocumentOutputError> {
    let rendered = render_readable_view(
        &value,
        kind,
        &docnav_readable::RendererConfig::default_config(),
    )
    .map_err(DocumentOutputError::ReadableViewRender)?;
    stdout
        .write_all(rendered.as_bytes())
        .map_err(DocumentOutputError::StdoutWrite)
}
