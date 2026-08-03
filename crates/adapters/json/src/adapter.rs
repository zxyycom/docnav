use std::fs;

use docnav_adapter_contracts::{
    Adapter, AdapterDefinition, AdapterError, AdapterResult, DocumentContentInvalidReason,
    FindInput, InfoInput, OutlineInput, ReadInput, UnstructuredFullRead,
    UnstructuredFullReadCapabilities,
};
use docnav_protocol::{
    AdapterIdentity, Cost, Entry, FindResult, FormatDescriptor, InfoAdapter, InfoDocument,
    InfoResult, Manifest, Measurement, OutlineResult, ReadResult, RequestEnvelope,
    MANIFEST_VERSION,
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
            extensions: vec![".json".to_owned(), ".code-workspace".to_owned()],
            filenames: vec![".prettierrc".to_owned(), ".watchmanconfig".to_owned()],
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
        LoadError::InvalidJson { .. } => AdapterError::document_content_invalid(
            path,
            DocumentContentInvalidReason::JsonSyntaxInvalid,
        ),
        LoadError::TrailingInput { .. } => AdapterError::document_content_invalid(
            path,
            DocumentContentInvalidReason::JsonTrailingInput,
        ),
        LoadError::DuplicateMember { .. } => AdapterError::document_content_invalid(
            path,
            DocumentContentInvalidReason::JsonDuplicateMember,
        ),
        LoadError::MaximumDepthExceeded { .. } => AdapterError::document_content_invalid(
            path,
            DocumentContentInvalidReason::JsonMaximumDepthExceeded,
        ),
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

#[cfg(test)]
mod tests;
