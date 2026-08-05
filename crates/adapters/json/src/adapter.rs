use std::fs;

use docnav_adapter_contracts::{
    Adapter, AdapterDefinition, AdapterDocument, AdapterError, AdapterResult,
    DocumentContentInvalidReason, FindInput, InfoInput, OutlineInput, ReadInput,
    UnstructuredFullRead, UnstructuredFullReadCapabilities,
};
use docnav_protocol::{
    AdapterIdentity, Cost, Entry, FindResult, FormatDescriptor, InfoAdapter, InfoDocument,
    InfoResult, Manifest, Measurement, OutlineResult, ReadResult, RequestEnvelope,
    MANIFEST_VERSION,
};
use serde_json::json;

use crate::content::{full_read_facts, selection_facts};
use crate::document::{load, JsonDocument, JsonKind, LoadError};
use crate::find::FindEntry;
use crate::paging::{paginate_entries, paginate_find_entries, paginate_text};
use crate::reference::{RefError, RefView};
use crate::traversal::{JsonEntry, JsonEntryKind};

pub(crate) const ADAPTER_ID: &str = "docnav-json";
const ADAPTER_NAME: &str = "Docnav JSON Adapter";
pub(crate) const FORMAT_ID_JSON: &str = "json";
pub(crate) const CONTENT_TYPE_JSON: &str = "application/json";
const CONTENT_TYPE_JSONC: &str = "application/jsonc";

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct JsonAdapter;

impl Adapter for JsonAdapter {
    fn create_document(&self, document_path: String) -> Box<dyn AdapterDocument + '_> {
        Box::new(JsonAdapterDocument {
            document_path,
            prepared: None,
        })
    }
}

struct JsonAdapterDocument {
    document_path: String,
    prepared: Option<AdapterResult<JsonDocument>>,
}

impl JsonAdapterDocument {
    fn prepared(&mut self) -> AdapterResult<&JsonDocument> {
        let document_path = &self.document_path;
        self.prepared
            .get_or_insert_with(|| reload_document(document_path))
            .as_ref()
            .map_err(Clone::clone)
    }
}

impl AdapterDocument for JsonAdapterDocument {
    fn outline(&mut self, input: &OutlineInput) -> AdapterResult<OutlineResult> {
        let document = self.prepared()?;
        let entries = document.preorder_entries();
        let (entries, page) = paginate_entries(&entries, input.page, input.limit);
        let entries = entries.into_iter().map(outline_entry).collect();

        Ok(OutlineResult::structured(entries, page))
    }

    fn read(&mut self, input: &ReadInput) -> AdapterResult<ReadResult> {
        let document = self.prepared()?;
        let selection = document
            .resolve_selection(&input.ref_id)
            .map_err(|error| match error {
                RefError::Invalid { reason } => AdapterError::ref_invalid(&input.ref_id, reason),
                RefError::NotFound => AdapterError::ref_not_found(&input.ref_id),
            })?;
        let content_type = match selection.view {
            RefView::Base => CONTENT_TYPE_JSON,
            RefView::DirectComments | RefView::TailComments => CONTENT_TYPE_JSONC,
        };
        let facts = selection_facts(document, &selection)
            .map_err(|_| AdapterError::internal("json-structured-serialization-failed"))?;
        let page = paginate_text(facts, input.page, input.limit);

        Ok(ReadResult {
            ref_id: input.ref_id.clone(),
            content: page.content,
            content_type: content_type.to_owned(),
            cost: page.cost,
            page: page.page,
        })
    }

    fn find(&mut self, input: &FindInput) -> AdapterResult<FindResult> {
        if input.query.is_empty() {
            return Err(AdapterError::invalid_request(
                "arguments.query",
                "query must not be empty",
            ));
        }

        let document = self.prepared()?;
        let matches = document.find_entries(&input.query);
        let (matches, page) = paginate_find_entries(matches, input.page, input.limit);
        let matches = matches.into_iter().map(find_entry).collect();

        Ok(FindResult::new(matches, page))
    }

    fn info(&mut self, _input: &InfoInput) -> AdapterResult<InfoResult> {
        let document = self.prepared()?;

        Ok(InfoResult {
            document: Some(InfoDocument {
                content_type: Some(source_content_type(document).to_owned()),
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
        &mut self,
        _request: &RequestEnvelope,
    ) -> AdapterResult<UnstructuredFullRead> {
        let document = self.prepared()?;
        let facts = full_read_facts(document);
        let mut result = UnstructuredFullRead::new(facts.content, source_content_type(document));
        result.facts.cost = Some(facts.cost);
        Ok(result)
    }

    fn measure_unstructured_full_read_cost(
        &mut self,
        _request: &RequestEnvelope,
        requested_units: &[String],
    ) -> AdapterResult<Cost> {
        let document = self.prepared()?;
        let cost = full_read_facts(document).cost;
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
            extensions: vec![
                ".json".to_owned(),
                ".code-workspace".to_owned(),
                ".jsonc".to_owned(),
                ".code-snippets".to_owned(),
                ".jsonld".to_owned(),
                ".geojson".to_owned(),
                ".har".to_owned(),
                ".webmanifest".to_owned(),
                ".ipynb".to_owned(),
                ".sarif".to_owned(),
            ],
            filenames: vec![
                ".prettierrc".to_owned(),
                ".watchmanconfig".to_owned(),
                "Pipfile.lock".to_owned(),
                "deno.lock".to_owned(),
            ],
            content_types: vec![CONTENT_TYPE_JSON.to_owned(), CONTENT_TYPE_JSONC.to_owned()],
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
        AdapterError::document_path_invalid(path, "document path could not be read")
    }
}

fn source_content_type(document: &JsonDocument) -> &'static str {
    if document.has_jsonc_syntax {
        CONTENT_TYPE_JSONC
    } else {
        CONTENT_TYPE_JSON
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
    let kind = match entry.kind {
        JsonEntryKind::Value(kind) => kind_name(kind),
        JsonEntryKind::TailComments => "tail_comments",
    };
    Entry {
        ref_id: entry.ref_id,
        label: entry.label,
        kind: Some(kind.to_owned()),
        location: None,
        summary: entry.summary,
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
