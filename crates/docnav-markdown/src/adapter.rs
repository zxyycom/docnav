use docnav_adapter_contracts::{
    Adapter, AdapterError, AdapterResult, NativeOptionSpec, NativeOptionValueSpec,
};
use docnav_protocol::{
    AdapterIdentity, FindArguments, FindResult, FormatDescriptor, InfoAdapter, InfoArguments,
    InfoDocument, InfoResult, Manifest, Measurement, Operation, OutlineArguments, OutlineResult,
    ProbeReason, ProbeReasonCode, ProbeResult, ReadArguments, ReadResult, RequestEnvelope,
    MANIFEST_VERSION, PROBE_VERSION,
};
use serde_json::json;

use crate::markdown::{
    cost_for, is_markdown_extension, is_utf8_markdown_candidate, max_heading_level_from_options,
    MarkdownDocument, ResolvedRef,
};
use crate::paging::{paginate_entries, paginate_text};

pub const ADAPTER_ID: &str = "docnav-markdown";
pub const ADAPTER_NAME: &str = "Docnav Markdown Adapter";
pub const FORMAT_ID_MARKDOWN: &str = "markdown";
pub const CONTENT_TYPE_MARKDOWN: &str = "text/markdown";
pub const DEFAULT_MAX_HEADING_LEVEL: u8 = 3;
// Markdown adapter 原生 option key，来自 adapter 契约；接入层只原样传递。
pub const NATIVE_OPTIONS_NAMESPACE: &str = "options";
pub const MAX_HEADING_LEVEL_OPTION: &str = "max_heading_level";
pub const MAX_HEADING_LEVEL_IDENTITY: &str =
    "docnav.adapters.docnav-markdown.options.max_heading_level";
const MARKDOWN_CAPABILITIES: &[Operation] = &[
    Operation::Outline,
    Operation::Read,
    Operation::Find,
    Operation::Info,
];
const MAX_HEADING_LEVEL_OPERATIONS: &[Operation] = &[Operation::Outline, Operation::Find];
const NATIVE_OPTIONS: &[NativeOptionSpec] = &[NativeOptionSpec {
    identity: MAX_HEADING_LEVEL_IDENTITY,
    owner: ADAPTER_ID,
    namespace: NATIVE_OPTIONS_NAMESPACE,
    key: MAX_HEADING_LEVEL_OPTION,
    operations: MAX_HEADING_LEVEL_OPERATIONS,
    value: NativeOptionValueSpec::Integer { min: 1, max: 6 },
}];

#[derive(Clone, Copy, Debug, Default)]
pub struct MarkdownAdapter;

impl Adapter for MarkdownAdapter {
    fn adapter_id(&self) -> &str {
        ADAPTER_ID
    }

    fn manifest(&self) -> Manifest {
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
                content_types: vec![CONTENT_TYPE_MARKDOWN.to_owned()],
            }],
            capabilities: MARKDOWN_CAPABILITIES.to_vec(),
        }
    }

    fn native_options(&self) -> &'static [NativeOptionSpec] {
        NATIVE_OPTIONS
    }

    fn probe(&self, path: &str) -> ProbeResult {
        let extension_match = is_markdown_extension(path);
        let mut reasons = Vec::new();

        if extension_match {
            reasons.push(ProbeReason {
                code: ProbeReasonCode::ExtensionMatch,
                detail: "path extension is declared for Markdown".to_owned(),
            });
        }

        match is_utf8_markdown_candidate(path) {
            Ok(true) if extension_match => {
                reasons.push(ProbeReason {
                    code: ProbeReasonCode::ContentMatch,
                    detail: "document is valid UTF-8 Markdown input".to_owned(),
                });
                probe(path, true, Some(FORMAT_ID_MARKDOWN), 1.0, reasons)
            }
            Ok(false) if extension_match => {
                reasons.push(ProbeReason {
                    code: ProbeReasonCode::ContentConflict,
                    detail: "document is not valid UTF-8".to_owned(),
                });
                probe(path, false, None, 0.0, reasons)
            }
            Ok(_) => {
                reasons.push(ProbeReason {
                    code: ProbeReasonCode::ContentConflict,
                    detail: "path extension is not declared for Markdown".to_owned(),
                });
                probe(path, false, None, 0.0, reasons)
            }
            Err(error) => {
                reasons.push(ProbeReason {
                    code: ProbeReasonCode::ReadError,
                    detail: error.to_string(),
                });
                probe(path, false, None, 0.0, reasons)
            }
        }
    }

    fn outline(
        &self,
        request: &RequestEnvelope,
        arguments: &OutlineArguments,
    ) -> AdapterResult<OutlineResult> {
        let document = MarkdownDocument::load(&request.document.path)?;
        let max_heading_level = max_heading_level_from_options(arguments.options.as_ref())?;
        let entries = document.outline_entries(max_heading_level);
        let (entries, page) = paginate_entries(&entries, arguments.page, arguments.limit);
        Ok(OutlineResult { entries, page })
    }

    fn read(
        &self,
        request: &RequestEnvelope,
        arguments: &ReadArguments,
    ) -> AdapterResult<ReadResult> {
        let document = MarkdownDocument::load(&request.document.path)?;
        let resolved = document.resolve_ref(&arguments.ref_id)?;
        let content = match resolved {
            ResolvedRef::FullDocument => document.source(),
            ResolvedRef::Heading(heading) => document.section_content(heading),
        };
        let (content_page, page) = paginate_text(content, arguments.page, arguments.limit);

        Ok(ReadResult {
            ref_id: arguments.ref_id.clone(),
            content: content_page,
            content_type: CONTENT_TYPE_MARKDOWN.to_owned(),
            cost: cost_for(content),
            page,
        })
    }

    fn find(
        &self,
        request: &RequestEnvelope,
        arguments: &FindArguments,
    ) -> AdapterResult<FindResult> {
        if arguments.query.is_empty() {
            return Err(AdapterError::invalid_request(
                "arguments.query",
                "query must not be empty",
            ));
        }

        let document = MarkdownDocument::load(&request.document.path)?;
        let max_heading_level = max_heading_level_from_options(arguments.options.as_ref())?;
        let matches = document.find_entries(&arguments.query, max_heading_level);
        let (matches, page) = paginate_entries(&matches, arguments.page, arguments.limit);

        Ok(FindResult { matches, page })
    }

    fn info(
        &self,
        request: &RequestEnvelope,
        _arguments: &InfoArguments,
    ) -> AdapterResult<InfoResult> {
        let document = MarkdownDocument::load(&request.document.path)?;
        Ok(InfoResult {
            capabilities: MARKDOWN_CAPABILITIES.to_vec(),
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
}

fn probe(
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
