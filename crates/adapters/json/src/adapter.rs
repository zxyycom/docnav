use std::fs;
use std::path::Path;

use docnav_adapter_contracts::{
    Adapter, AdapterDefinition, AdapterError, AdapterResult, FindInput, InfoInput, OutlineInput,
    ReadInput, UnstructuredFullRead, UnstructuredFullReadCapabilities,
};
use docnav_protocol::{
    AdapterIdentity, Cost, Entry, FindResult, FormatDescriptor, InfoAdapter, InfoDocument,
    InfoResult, Manifest, Measurement, OutlineResult, ProbeReason, ProbeReasonCode, ProbeResult,
    ReadResult, RequestEnvelope, MANIFEST_VERSION, PROBE_VERSION,
};
use serde_json::json;

use crate::content::{full_read_facts, structured_value_facts};
use crate::document::{load, JsonDocument, JsonKind, LoadError};
use crate::find::FindEntry;
use crate::paging::{paginate_entries, paginate_find_entries, paginate_text};
use crate::reference::RefError;
use crate::traversal::JsonEntry;

pub(crate) const ADAPTER_ID: &str = "docnav-json";
const ADAPTER_NAME: &str = "Docnav JSON Adapter";
pub(crate) const FORMAT_ID_JSON: &str = "json";
pub(crate) const CONTENT_TYPE_JSON: &str = "application/json";

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct JsonAdapter;

impl Adapter for JsonAdapter {
    fn probe(&self, path: &str) -> ProbeResult {
        if !has_json_extension(path) {
            return probe_result(
                path,
                false,
                None,
                0.0,
                vec![ProbeReason {
                    code: ProbeReasonCode::ContentConflict,
                    detail: "path extension is not declared for JSON".to_owned(),
                }],
            );
        }

        let mut reasons = vec![ProbeReason {
            code: ProbeReasonCode::ExtensionMatch,
            detail: "path extension is declared for JSON".to_owned(),
        }];
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) => {
                reasons.push(ProbeReason {
                    code: ProbeReasonCode::ReadError,
                    detail: error.to_string(),
                });
                return probe_result(path, false, None, 0.0, reasons);
            }
        };

        match load(&bytes) {
            Ok(_) => {
                reasons.push(ProbeReason {
                    code: ProbeReasonCode::ContentMatch,
                    detail: "document is valid UTF-8 JSON input".to_owned(),
                });
                probe_result(path, true, Some(FORMAT_ID_JSON), 1.0, reasons)
            }
            Err(error) => {
                reasons.push(ProbeReason {
                    code: ProbeReasonCode::ContentConflict,
                    detail: loader_conflict_detail(error),
                });
                probe_result(path, false, None, 0.0, reasons)
            }
        }
    }

    fn outline(&self, input: &OutlineInput) -> AdapterResult<OutlineResult> {
        let document = reload_document(&input.document_path)?;
        let entries = document.preorder_entries();
        let (entries, page) = paginate_entries(&entries, input.page, input.limit);
        let entries = entries.into_iter().map(outline_entry).collect();

        Ok(OutlineResult::structured(entries, page))
    }

    fn read(&self, input: &ReadInput) -> AdapterResult<ReadResult> {
        let document = reload_document(&input.document_path)?;
        let node = document
            .resolve_ref(&input.ref_id)
            .map_err(|error| match error {
                RefError::Invalid { reason } => AdapterError::ref_invalid(&input.ref_id, reason),
                RefError::NotFound => AdapterError::ref_not_found(&input.ref_id),
            })?;
        let facts = structured_value_facts(node)
            .map_err(|_| AdapterError::internal("json-structured-serialization-failed"))?;
        let page = paginate_text(facts, input.page, input.limit);

        Ok(ReadResult {
            ref_id: input.ref_id.clone(),
            content: page.content,
            content_type: CONTENT_TYPE_JSON.to_owned(),
            cost: page.cost,
            page: page.page,
        })
    }

    fn find(&self, input: &FindInput) -> AdapterResult<FindResult> {
        if input.query.is_empty() {
            return Err(AdapterError::invalid_request(
                "arguments.query",
                "query must not be empty",
            ));
        }

        let document = reload_document(&input.document_path)?;
        let matches = document.find_entries(&input.query);
        let (matches, page) = paginate_find_entries(matches, input.page, input.limit);
        let matches = matches.into_iter().map(find_entry).collect();

        Ok(FindResult::new(matches, page))
    }

    fn info(&self, input: &InfoInput) -> AdapterResult<InfoResult> {
        let document = reload_document(&input.document_path)?;

        Ok(InfoResult {
            document: Some(InfoDocument {
                content_type: Some(CONTENT_TYPE_JSON.to_owned()),
                encoding: Some("UTF-8".to_owned()),
                size: Some(Measurement {
                    unit: "bytes".to_owned(),
                    value: document.original_byte_size as u64,
                    scope: None,
                }),
            }),
            adapter: Some(InfoAdapter {
                id: Some(ADAPTER_ID.to_owned()),
                format: Some(FORMAT_ID_JSON.to_owned()),
            }),
            metadata: Some(serde_json::Map::from_iter([
                (
                    "root_kind".to_owned(),
                    json!(kind_name(document.root_kind())),
                ),
                ("node_count".to_owned(), json!(document.node_count)),
                ("max_depth".to_owned(), json!(document.max_depth)),
            ])),
        })
    }

    fn unstructured_full_read(
        &self,
        request: &RequestEnvelope,
    ) -> AdapterResult<UnstructuredFullRead> {
        let document = reload_document(&request.document.path)?;
        let facts = full_read_facts(&document);
        let mut result = UnstructuredFullRead::new(facts.content, CONTENT_TYPE_JSON);
        result.facts.cost = Some(facts.cost);
        Ok(result)
    }

    fn measure_unstructured_full_read_cost(
        &self,
        request: &RequestEnvelope,
        requested_units: &[String],
    ) -> AdapterResult<Cost> {
        let document = reload_document(&request.document.path)?;
        let cost = full_read_facts(&document).cost;
        Ok(Cost {
            measurements: cost
                .measurements
                .into_iter()
                .filter(|measurement| requested_units.iter().any(|unit| unit == &measurement.unit))
                .collect(),
        })
    }
}

pub fn json_adapter_definition() -> AdapterDefinition<'static> {
    AdapterDefinition::new(
        json_manifest(),
        &JsonAdapter,
        Some(json_full_read_capabilities()),
    )
    .expect("JSON adapter definition is valid")
}

fn json_manifest() -> Manifest {
    Manifest {
        manifest_version: MANIFEST_VERSION.to_owned(),
        adapter: AdapterIdentity {
            id: ADAPTER_ID.to_owned(),
            name: ADAPTER_NAME.to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        },
        formats: vec![FormatDescriptor {
            id: FORMAT_ID_JSON.to_owned(),
            extensions: vec![".json".to_owned()],
            content_types: vec![CONTENT_TYPE_JSON.to_owned()],
        }],
    }
}

fn json_full_read_capabilities() -> UnstructuredFullReadCapabilities {
    UnstructuredFullReadCapabilities {
        content_hook: true,
        cost_measurement_units: vec!["lines".to_owned(), "bytes".to_owned(), "tokens".to_owned()],
        result_facts_hook: false,
    }
}

fn has_json_extension(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case(FORMAT_ID_JSON))
}

fn loader_conflict_detail(error: LoadError) -> String {
    match error {
        LoadError::InvalidUtf8 { .. } => "document is not valid UTF-8".to_owned(),
        LoadError::InvalidJson { .. } => "document is not valid JSON".to_owned(),
        LoadError::TrailingInput { .. } => "document has trailing non-whitespace input".to_owned(),
        LoadError::DuplicateMember { name } => {
            format!("document has duplicate decoded member name {name:?}")
        }
        LoadError::MaximumDepthExceeded { maximum, actual } => {
            format!("document maximum depth {actual} exceeds supported maximum {maximum}")
        }
    }
}

fn reload_document(path: &str) -> AdapterResult<JsonDocument> {
    let bytes = fs::read(path).map_err(|error| read_error(path, error))?;
    load(&bytes).map_err(|error| reload_error(path, error))
}

fn read_error(path: &str, error: std::io::Error) -> AdapterError {
    if error.kind() == std::io::ErrorKind::NotFound {
        AdapterError::document_not_found(path)
    } else {
        AdapterError::document_path_invalid(path, error.to_string())
    }
}

fn reload_error(path: &str, error: LoadError) -> AdapterError {
    match error {
        LoadError::InvalidUtf8 { .. } => {
            AdapterError::document_encoding_unsupported(path, "non-utf-8")
        }
        LoadError::InvalidJson { .. }
        | LoadError::TrailingInput { .. }
        | LoadError::DuplicateMember { .. }
        | LoadError::MaximumDepthExceeded { .. } => {
            AdapterError::internal("json-document-changed-after-probe")
        }
    }
}

fn outline_entry(entry: JsonEntry) -> Entry {
    Entry {
        ref_id: entry.ref_id,
        label: entry.label,
        kind: Some(kind_name(entry.kind).to_owned()),
        location: None,
        summary: None,
        excerpt: None,
        rank: None,
        cost: None,
        metadata: None,
    }
}

fn find_entry(entry: FindEntry) -> Entry {
    Entry {
        ref_id: entry.ref_id,
        label: entry.label,
        kind: Some("match".to_owned()),
        location: Some(entry.location),
        summary: None,
        excerpt: None,
        rank: None,
        cost: None,
        metadata: None,
    }
}

const fn kind_name(kind: JsonKind) -> &'static str {
    match kind {
        JsonKind::Object => "object",
        JsonKind::Array => "array",
        JsonKind::String => "string",
        JsonKind::Number => "number",
        JsonKind::Boolean => "boolean",
        JsonKind::Null => "null",
    }
}

fn probe_result(
    path: &str,
    supported: bool,
    format: Option<&str>,
    confidence: f64,
    reasons: Vec<ProbeReason>,
) -> ProbeResult {
    ProbeResult {
        probe_version: PROBE_VERSION.to_owned(),
        adapter_id: ADAPTER_ID.to_owned(),
        path: path.to_owned(),
        supported,
        format: format.map(str::to_owned),
        confidence,
        reasons,
    }
}

#[cfg(test)]
mod tests;
