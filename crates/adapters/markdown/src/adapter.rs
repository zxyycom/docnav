use docnav_adapter_contracts::{
    Adapter, AdapterDefinition, AdapterDocument, AdapterError, AdapterResult, FindInput, InfoInput,
    OutlineInput, ReadInput, UnstructuredFullRead, UnstructuredFullReadCapabilities,
};
use docnav_protocol::{
    AdapterIdentity, FindResult, FormatDescriptor, InfoAdapter, InfoDocument, InfoResult, Manifest,
    Measurement, OutlineResult, ReadResult, RequestEnvelope, MANIFEST_VERSION,
};
use serde_json::json;

use crate::markdown::{cost_for, max_heading_level, MarkdownDocument, ResolvedRef};
use crate::paging::{paginate_entries, paginate_text};

pub const ADAPTER_ID: &str = "docnav-markdown";
pub const ADAPTER_NAME: &str = "Docnav Markdown Adapter";
pub const FORMAT_ID_MARKDOWN: &str = "markdown";
pub const CONTENT_TYPE_MARKDOWN: &str = "text/markdown";

#[derive(Clone, Copy, Debug, Default)]
pub struct MarkdownAdapter;

impl Adapter for MarkdownAdapter {
    fn create_document(&self, document_path: String) -> Box<dyn AdapterDocument + '_> {
        Box::new(MarkdownAdapterDocument {
            document_path,
            prepared: None,
        })
    }
}

struct MarkdownAdapterDocument {
    document_path: String,
    prepared: Option<AdapterResult<MarkdownDocument>>,
}

impl MarkdownAdapterDocument {
    fn prepared(&mut self) -> AdapterResult<&MarkdownDocument> {
        if self.prepared.is_none() {
            self.prepared = Some(MarkdownDocument::load(&self.document_path));
        }

        match self
            .prepared
            .as_ref()
            .expect("prepared state was initialized")
        {
            Ok(document) => Ok(document),
            Err(error) => Err(error.clone()),
        }
    }
}

impl AdapterDocument for MarkdownAdapterDocument {
    fn outline(&mut self, input: &OutlineInput) -> AdapterResult<OutlineResult> {
        let document = self.prepared()?;
        let max_heading_level = max_heading_level(input.max_heading_level)?;
        let entries = document.outline_entries(max_heading_level);
        let (entries, page) = paginate_entries(&entries, input.page, input.limit);
        Ok(OutlineResult::structured(entries, page))
    }

    fn read(&mut self, input: &ReadInput) -> AdapterResult<ReadResult> {
        let document = self.prepared()?;
        let resolved = document.resolve_ref(&input.ref_id)?;
        let content = match resolved {
            ResolvedRef::FullDocument => document.source(),
            ResolvedRef::DocumentHead => document.document_head_content(),
            ResolvedRef::Heading(heading) => document.section_content(heading),
        };
        let (content_page, page) = paginate_text(content, input.page, input.limit);

        Ok(ReadResult {
            ref_id: input.ref_id.clone(),
            content: content_page,
            content_type: CONTENT_TYPE_MARKDOWN.to_owned(),
            cost: cost_for(content),
            page,
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
        let max_heading_level = max_heading_level(input.max_heading_level)?;
        let matches = document.find_entries(&input.query, max_heading_level);
        let (matches, page) = paginate_entries(&matches, input.page, input.limit);

        Ok(FindResult::new(matches, page))
    }

    fn info(&mut self, _input: &InfoInput) -> AdapterResult<InfoResult> {
        let document = self.prepared()?;
        Ok(InfoResult {
            document: Some(InfoDocument {
                content_type: Some(CONTENT_TYPE_MARKDOWN.to_owned()),
                encoding: Some("utf-8".to_owned()),
                size: Some(Measurement {
                    unit: "bytes".to_owned(),
                    value: document.source().len() as u64,
                    scope: None,
                }),
            }),
            adapter: Some(InfoAdapter {
                id: Some(ADAPTER_ID.to_owned()),
                format: Some(FORMAT_ID_MARKDOWN.to_owned()),
            }),
            metadata: Some(serde_json::Map::from_iter([(
                "heading_count".to_owned(),
                json!(document.headings().len()),
            )])),
        })
    }

    fn unstructured_full_read(
        &mut self,
        _request: &RequestEnvelope,
    ) -> AdapterResult<UnstructuredFullRead> {
        let document = self.prepared()?;
        let mut result = UnstructuredFullRead::new(document.source(), CONTENT_TYPE_MARKDOWN);
        result.facts.cost = Some(cost_for(document.source()));
        Ok(result)
    }

    fn measure_unstructured_full_read_cost(
        &mut self,
        _request: &RequestEnvelope,
        requested_units: &[String],
    ) -> AdapterResult<docnav_protocol::Cost> {
        let document = self.prepared()?;
        let cost = cost_for(document.source());
        Ok(docnav_protocol::Cost {
            measurements: cost
                .measurements
                .into_iter()
                .filter(|measurement| requested_units.iter().any(|unit| unit == &measurement.unit))
                .collect(),
        })
    }
}

pub fn markdown_adapter_definition() -> AdapterDefinition<'static> {
    AdapterDefinition::new(
        markdown_manifest(),
        &MarkdownAdapter,
        Some(markdown_full_read_capabilities()),
    )
    .expect("Markdown adapter definition is valid")
}

fn markdown_manifest() -> Manifest {
    Manifest {
        manifest_version: MANIFEST_VERSION.to_owned(),
        adapter: AdapterIdentity {
            id: ADAPTER_ID.to_owned(),
            name: ADAPTER_NAME.to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        },
        formats: vec![FormatDescriptor {
            id: FORMAT_ID_MARKDOWN.to_owned(),
            extensions: vec![".md".to_owned(), ".markdown".to_owned()],
            filenames: vec![],
            content_types: vec![CONTENT_TYPE_MARKDOWN.to_owned()],
        }],
    }
}

fn markdown_full_read_capabilities() -> UnstructuredFullReadCapabilities {
    UnstructuredFullReadCapabilities {
        content_hook: true,
        cost_measurement_units: vec!["lines".to_owned(), "bytes".to_owned(), "tokens".to_owned()],
        result_facts_hook: false,
    }
}
