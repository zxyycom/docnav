use std::collections::BTreeMap;

use docnav_adapter_sdk::{Adapter, AdapterResult};
use docnav_protocol::{
    try_positive, AdapterIdentity, FindArguments, FindResult, FormatDescriptor, InfoArguments,
    InfoResult, Manifest, Operation, Options, OutlineArguments, OutlineResult, PagedOperation,
    ProbeReason, ProbeReasonCode, ProbeResult, ProtocolRange, ReadArguments, ReadResult,
    RecommendedParameters, RequestEnvelope, StableError, MANIFEST_VERSION, PROBE_VERSION,
};
use serde_json::Value;

use crate::markdown::{
    cost_for, is_markdown_extension, is_utf8_markdown_candidate, max_heading_level_from_options,
    MarkdownDocument, ResolvedRef,
};
use crate::paging::{paginate_entries, paginate_text};

pub const ADAPTER_ID: &str = "docnav-markdown";
pub const ADAPTER_NAME: &str = "Docnav Markdown Adapter";
pub const FORMAT_ID_MARKDOWN: &str = "markdown";
pub const CONTENT_TYPE_MARKDOWN: &str = "text/markdown";
pub const DEFAULT_LIMIT_CHARS: u32 = 6000;
pub const DEFAULT_MAX_HEADING_LEVEL: u8 = 3;

#[derive(Clone, Copy, Debug, Default)]
pub struct MarkdownAdapter;

impl Adapter for MarkdownAdapter {
    fn adapter_id(&self) -> &str {
        ADAPTER_ID
    }

    fn manifest(&self) -> Manifest {
        let mut recommended_parameters = BTreeMap::new();
        recommended_parameters.insert(
            PagedOperation::Outline,
            RecommendedParameters {
                limit_chars: positive(DEFAULT_LIMIT_CHARS),
                options: Some(max_heading_options(DEFAULT_MAX_HEADING_LEVEL)),
            },
        );
        recommended_parameters.insert(
            PagedOperation::Read,
            RecommendedParameters {
                limit_chars: positive(DEFAULT_LIMIT_CHARS),
                options: None,
            },
        );
        recommended_parameters.insert(
            PagedOperation::Find,
            RecommendedParameters {
                limit_chars: positive(DEFAULT_LIMIT_CHARS),
                options: Some(max_heading_options(DEFAULT_MAX_HEADING_LEVEL)),
            },
        );

        Manifest {
            manifest_version: MANIFEST_VERSION.to_owned(),
            adapter: AdapterIdentity {
                id: ADAPTER_ID.to_owned(),
                name: ADAPTER_NAME.to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
            },
            protocol: ProtocolRange::v0_1(),
            formats: vec![FormatDescriptor {
                id: FORMAT_ID_MARKDOWN.to_owned(),
                extensions: vec![".md".to_owned(), ".markdown".to_owned()],
                content_types: vec![CONTENT_TYPE_MARKDOWN.to_owned()],
            }],
            capabilities: vec![
                Operation::Outline,
                Operation::Read,
                Operation::Find,
                Operation::Info,
            ],
            recommended_parameters,
        }
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
        let (entries, page) = paginate_entries(&entries, arguments.page, arguments.limit_chars);
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
        let (content_page, page) = paginate_text(content, arguments.page, arguments.limit_chars);

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
            return Err(
                StableError::invalid_request("arguments.query", "query must not be empty").into(),
            );
        }

        let document = MarkdownDocument::load(&request.document.path)?;
        let max_heading_level = max_heading_level_from_options(arguments.options.as_ref())?;
        let matches = document.find_entries(&arguments.query, max_heading_level);
        let (matches, page) = paginate_entries(&matches, arguments.page, arguments.limit_chars);

        Ok(FindResult { matches, page })
    }

    fn info(
        &self,
        request: &RequestEnvelope,
        _arguments: &InfoArguments,
    ) -> AdapterResult<InfoResult> {
        let document = MarkdownDocument::load(&request.document.path)?;
        Ok(InfoResult {
            display: format!(
                "Markdown | {} | {} headings | {}",
                CONTENT_TYPE_MARKDOWN,
                document.headings().len(),
                cost_for(document.source())
            ),
            capabilities: vec![
                Operation::Outline,
                Operation::Read,
                Operation::Find,
                Operation::Info,
            ],
        })
    }
}

pub fn direct_outline_arguments(
    limit_chars: docnav_protocol::PositiveInteger,
    page: docnav_protocol::PositiveInteger,
    max_heading_level: u8,
) -> OutlineArguments {
    OutlineArguments {
        limit_chars,
        page,
        options: Some(max_heading_options(max_heading_level)),
    }
}

pub fn direct_find_arguments(
    query: String,
    limit_chars: docnav_protocol::PositiveInteger,
    page: docnav_protocol::PositiveInteger,
    max_heading_level: u8,
) -> FindArguments {
    FindArguments {
        query,
        limit_chars,
        page,
        options: Some(max_heading_options(max_heading_level)),
    }
}

fn max_heading_options(max_heading_level: u8) -> Options {
    let mut options = Options::new();
    options.insert(
        "max_heading_level".to_owned(),
        Value::from(max_heading_level),
    );
    options
}

fn positive(value: u32) -> docnav_protocol::PositiveInteger {
    try_positive(value).expect("static positive integer")
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
