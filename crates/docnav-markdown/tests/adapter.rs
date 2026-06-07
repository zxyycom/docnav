use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use docnav_adapter_sdk::{invoke_once, Adapter, AdapterExitCode};
use docnav_markdown::MarkdownAdapter;
use docnav_protocol::{
    positive_result, Document, FindArguments, FindResult, InfoArguments, Operation,
    OperationArguments, Options, OutlineArguments, OutlineResult, ProtocolResponse, ReadArguments,
    RequestEnvelope, StableError, StableErrorCode, PROTOCOL_VERSION,
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
const MAX_HEADING_LEVEL_OPTION: &str = "max_heading_level";

fn positive(value: u32) -> docnav_protocol::PositiveInteger {
    positive_result(value).expect("positive test integer")
}

fn write_doc(name: &str, content: &str) -> PathBuf {
    write_bytes(name, content.as_bytes())
}

fn write_bytes(name: &str, content: &[u8]) -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "docnav-markdown-adapter-test-{}-{}",
        std::process::id(),
        id
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(name);
    fs::write(&path, content).expect("write temp document");
    path
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn make_request(
    path: &Path,
    operation: Operation,
    arguments: OperationArguments,
) -> RequestEnvelope {
    RequestEnvelope {
        protocol_version: PROTOCOL_VERSION.to_owned(),
        request_id: "test-request".to_owned(),
        operation,
        document: Document {
            path: path_string(path),
        },
        arguments,
    }
}

fn outline_args(limit_chars: u32, page: u32, max_heading_level: Option<u8>) -> OutlineArguments {
    OutlineArguments {
        limit_chars: positive(limit_chars),
        page: positive(page),
        options: max_heading_level.map(max_heading_level_options),
    }
}

fn find_args(
    query: &str,
    limit_chars: u32,
    page: u32,
    max_heading_level: Option<u8>,
) -> FindArguments {
    FindArguments {
        query: query.to_owned(),
        limit_chars: positive(limit_chars),
        page: positive(page),
        options: max_heading_level.map(max_heading_level_options),
    }
}

fn max_heading_level_options(level: u8) -> Options {
    let mut options = Options::new();
    options.insert(
        MAX_HEADING_LEVEL_OPTION.to_owned(),
        serde_json::Value::from(level),
    );
    options
}

fn outline_result(path: &Path, arguments: &OutlineArguments) -> OutlineResult {
    let request = make_request(
        path,
        Operation::Outline,
        OperationArguments::Outline(arguments.clone()),
    );
    MarkdownAdapter
        .outline(&request, arguments)
        .expect("outline result")
}

fn find_result(path: &Path, arguments: &FindArguments) -> FindResult {
    let request = make_request(
        path,
        Operation::Find,
        OperationArguments::Find(arguments.clone()),
    );
    MarkdownAdapter
        .find(&request, arguments)
        .expect("find result")
}

#[test]
fn manifest_declares_markdown_v0_capabilities() {
    let manifest = MarkdownAdapter.manifest();

    manifest.validate_semantics().expect("manifest semantics");
    assert_eq!(manifest.adapter.id, "docnav-markdown");
    assert_eq!(manifest.formats[0].id, "markdown");
    assert!(manifest.formats[0].extensions.contains(&".md".to_owned()));
    assert!(manifest.formats[0]
        .extensions
        .contains(&".markdown".to_owned()));
    assert!(manifest.formats[0]
        .content_types
        .contains(&"text/markdown".to_owned()));
    assert_eq!(
        manifest.capabilities,
        vec![
            Operation::Outline,
            Operation::Read,
            Operation::Find,
            Operation::Info
        ]
    );

    let value = serde_json::to_value(&manifest).expect("manifest JSON");
    assert!(value.get("protocol").is_none());
    assert!(value.get("recommended_parameters").is_none());
}

#[test]
fn probe_returns_format_evidence_without_navigation_payload() {
    let path = write_doc("sample.md", "# Title\n");
    let probe = MarkdownAdapter.probe(&path_string(&path));
    let value = serde_json::to_value(&probe).expect("probe JSON");

    assert!(probe.supported);
    assert_eq!(probe.format.as_deref(), Some("markdown"));
    assert!(probe
        .reasons
        .iter()
        .any(|reason| reason.detail.contains("Markdown")));
    assert!(value.get("entries").is_none());
    assert!(value.get("content").is_none());
}

#[test]
fn outline_is_flat_default_h1_to_h3_and_ignores_code_fences() {
    let path = write_doc(
        "nested.md",
        "# Guide\nIntro\n\n```md\n## Fake\n```\n\n## Install\nBody\n\n#### Hidden\nDeep\n",
    );
    let arguments = outline_args(6000, 1, None);
    let request = make_request(
        &path,
        Operation::Outline,
        OperationArguments::Outline(arguments.clone()),
    );

    let result = MarkdownAdapter
        .outline(&request, &arguments)
        .expect("outline result");

    assert_eq!(result.entries.len(), 2);
    assert_eq!(result.entries[0].ref_id, "L1:Guide");
    assert_eq!(result.entries[1].ref_id, "L8:Guide > Install");
    for entry in &result.entries {
        assert_canonical_ref(&entry.ref_id);
    }
    assert!(!result
        .entries
        .iter()
        .any(|entry| entry.ref_id.contains("Fake")));
    assert!(!result
        .entries
        .iter()
        .any(|entry| entry.ref_id.contains("Hidden")));
}

#[test]
fn outline_falls_back_to_full_document_for_no_visible_heading() {
    for content in ["plain body\nwith no heading\n", "#### Deep\nBody\n"] {
        let path = write_doc("fallback.md", content);
        let arguments = outline_args(6000, 1, Some(3));
        let request = make_request(
            &path,
            Operation::Outline,
            OperationArguments::Outline(arguments.clone()),
        );
        let outline = MarkdownAdapter
            .outline(&request, &arguments)
            .expect("outline result");
        assert_eq!(outline.entries[0].ref_id, "doc:full");

        let read_arguments = ReadArguments {
            ref_id: outline.entries[0].ref_id.clone(),
            limit_chars: positive(6000),
            page: positive(1),
            options: None,
        };
        let read_request = make_request(
            &path,
            Operation::Read,
            OperationArguments::Read(read_arguments.clone()),
        );
        let read = MarkdownAdapter
            .read(&read_request, &read_arguments)
            .expect("read full document");
        assert_eq!(read.content, content);
        assert_eq!(read.content_type, "text/markdown");
    }
}

#[test]
fn duplicate_heading_paths_generate_unique_refs_and_read_unique_sections() {
    let path = write_doc("duplicates.md", "# A\n## B\nfirst\n# A\n## B\nsecond\n");
    let arguments = outline_args(6000, 1, Some(3));
    let request = make_request(
        &path,
        Operation::Outline,
        OperationArguments::Outline(arguments.clone()),
    );

    let outline = MarkdownAdapter
        .outline(&request, &arguments)
        .expect("outline result");
    let refs: Vec<String> = outline
        .entries
        .iter()
        .filter(|entry| entry.ref_id.contains("A > B"))
        .map(|entry| entry.ref_id.clone())
        .collect();

    let all_refs: Vec<String> = outline
        .entries
        .iter()
        .map(|entry| entry.ref_id.clone())
        .collect();
    assert_eq!(all_refs, vec!["L1:A", "L2:A > B", "L4#2:A", "L5#2:A > B"]);
    assert_eq!(refs, vec!["L2:A > B", "L5#2:A > B"]);
    for ref_id in &all_refs {
        assert_canonical_ref(ref_id);
    }

    let first = read_ref(&path, "L2:A > B");
    let second = read_ref(&path, "L5#2:A > B");
    let explicit_first = read_ref(&path, "L2#1:A > B");
    assert!(first.content.contains("first"));
    assert!(!first.content.contains("second"));
    assert!(second.content.contains("second"));
    assert!(!second.content.contains("first"));
    assert!(explicit_first.content.contains("first"));
    assert!(!explicit_first.content.contains("second"));
}

#[test]
fn read_reports_ref_not_found_for_missing_and_unsupported_refs() {
    let path = write_doc("refs.md", "# A\n## B\nfirst\n# A\n## B\nsecond\n");

    let missing_ref = "L99:Missing";
    let missing = read_ref_error(&path, missing_ref);
    assert_ref_not_found(&missing, missing_ref);

    let unsupported_ref = "P:A > B";
    let unsupported_path_ref = read_ref_error(&path, unsupported_ref);
    assert_ref_not_found(&unsupported_path_ref, unsupported_ref);

    let invalid_ordinal_ref = "L2#0:A > B";
    let invalid_ordinal = read_ref_error(&path, invalid_ordinal_ref);
    assert_ref_not_found(&invalid_ordinal, invalid_ordinal_ref);
}

#[test]
fn read_rejects_legacy_bracketed_ordinal_suffix() {
    let path = write_doc("legacy-ref.md", "# A\n## B\nfirst\n");
    let legacy_ref = legacy_ordinal_ref("L2:A > B", 1);

    let error = read_ref_error(&path, &legacy_ref);

    assert_ref_not_found(&error, &legacy_ref);
}

#[test]
fn read_paginates_unicode_without_splitting_characters() {
    let path = write_doc("unicode.md", "# A\n界界界abc\n");
    let ref_id = "L1:A";
    let arguments = ReadArguments {
        ref_id: ref_id.to_owned(),
        limit_chars: positive(5),
        page: positive(1),
        options: None,
    };
    let request = make_request(
        &path,
        Operation::Read,
        OperationArguments::Read(arguments.clone()),
    );

    let first = MarkdownAdapter
        .read(&request, &arguments)
        .expect("first page");
    assert_eq!(first.ref_id, ref_id);
    assert_eq!(first.content, "# A\n界");
    assert_eq!(first.page, Some(positive(2)));

    let second_arguments = ReadArguments {
        page: positive(2),
        ..arguments
    };
    let second_request = make_request(
        &path,
        Operation::Read,
        OperationArguments::Read(second_arguments.clone()),
    );
    let second = MarkdownAdapter
        .read(&second_request, &second_arguments)
        .expect("second page");
    assert!(second.content.starts_with("界界"));
}

#[test]
fn find_ref_targets_current_visible_region_and_read_contains_match() {
    let path = write_doc(
        "find-current-region.md",
        "# Current\nintro\n\n#### Hidden\ntarget\n\n# Next\nother\n",
    );
    let arguments = find_args("target", 6000, 1, Some(3));

    let result = find_result(&path, &arguments);

    assert_eq!(result.matches.len(), 1);
    assert_eq!(result.matches[0].ref_id, "L1:Current");
    assert!(result.matches[0].display.contains("target"));

    let read = read_ref(&path, &result.matches[0].ref_id);
    assert!(read.content.contains("target"));
    assert!(!read.content.contains("# Next"));
}

#[test]
fn find_match_before_first_visible_heading_falls_back_to_full_document() {
    let path = write_doc("find-before-heading.md", "target before\n\n# Later\nbody\n");
    let arguments = find_args("target before", 6000, 1, Some(3));

    let result = find_result(&path, &arguments);

    assert_eq!(result.matches.len(), 1);
    assert_eq!(result.matches[0].ref_id, "doc:full");

    let read = read_ref(&path, &result.matches[0].ref_id);
    assert!(read.content.contains("target before"));
}

#[test]
fn find_falls_back_to_full_document_when_no_heading_is_visible() {
    let path = write_doc("fallback-find.md", "#### Deep\ntarget\n");
    let arguments = find_args("target", 6000, 1, Some(3));

    let result = find_result(&path, &arguments);

    assert_eq!(result.matches.len(), 1);
    assert_eq!(result.matches[0].ref_id, "doc:full");
}

#[test]
fn outline_paginates_with_response_page_until_end_and_past_end() {
    let path = write_doc("outline-pages.md", "# A\none\n# B\ntwo\n# C\nthree\n");
    let first_arguments = outline_args(8, 1, Some(3));

    let first = outline_result(&path, &first_arguments);
    assert_eq!(entry_refs(&first.entries), vec!["L1:A"]);
    let second_page = first.page.expect("second page");

    let second_arguments = OutlineArguments {
        page: second_page,
        ..first_arguments.clone()
    };
    let second = outline_result(&path, &second_arguments);
    assert_eq!(entry_refs(&second.entries), vec!["L3:B"]);
    let third_page = second.page.expect("third page");

    let third_arguments = OutlineArguments {
        page: third_page,
        ..first_arguments.clone()
    };
    let third = outline_result(&path, &third_arguments);
    assert_eq!(entry_refs(&third.entries), vec!["L5:C"]);
    assert_eq!(third.page, None);

    let past_end_arguments = OutlineArguments {
        page: positive(4),
        ..first_arguments
    };
    let past_end = outline_result(&path, &past_end_arguments);
    assert!(past_end.entries.is_empty());
    assert_eq!(past_end.page, None);
}

#[test]
fn find_paginates_with_response_page_until_end_and_past_end() {
    let path = write_doc(
        "find-pages.md",
        "# A\ntarget 1\n# B\ntarget 2\n# C\ntarget 3\n",
    );
    let first_arguments = find_args("target", 10, 1, Some(3));

    let first = find_result(&path, &first_arguments);
    assert_eq!(entry_refs(&first.matches), vec!["L1:A"]);
    let second_page = first.page.expect("second page");

    let second_arguments = FindArguments {
        page: second_page,
        ..first_arguments.clone()
    };
    let second = find_result(&path, &second_arguments);
    assert_eq!(entry_refs(&second.matches), vec!["L3:B"]);
    let third_page = second.page.expect("third page");

    let third_arguments = FindArguments {
        page: third_page,
        ..first_arguments.clone()
    };
    let third = find_result(&path, &third_arguments);
    assert_eq!(entry_refs(&third.matches), vec!["L5:C"]);
    assert_eq!(third.page, None);

    let past_end_arguments = FindArguments {
        page: positive(4),
        ..first_arguments
    };
    let past_end = find_result(&path, &past_end_arguments);
    assert!(past_end.matches.is_empty());
    assert_eq!(past_end.page, None);
}

#[test]
fn adapter_owned_options_shape_outline_and_find_granularity() {
    let path = write_doc("adapter-owned-options.md", "# Top\n\n#### Deep\nneedle\n");
    let default_outline = OutlineArguments {
        limit_chars: positive(6000),
        page: positive(1),
        options: None,
    };
    let expanded_outline = OutlineArguments {
        options: Some(max_heading_level_options(4)),
        ..default_outline.clone()
    };

    let default = outline_result(&path, &default_outline);
    assert_eq!(entry_refs(&default.entries), vec!["L1:Top"]);

    let expanded = outline_result(&path, &expanded_outline);
    assert_eq!(
        entry_refs(&expanded.entries),
        vec!["L1:Top", "L3:Top > Deep"]
    );

    let default_find = FindArguments {
        query: "needle".to_owned(),
        limit_chars: positive(6000),
        page: positive(1),
        options: None,
    };
    let expanded_find = FindArguments {
        options: Some(max_heading_level_options(4)),
        ..default_find.clone()
    };

    let default_matches = find_result(&path, &default_find);
    assert_eq!(entry_refs(&default_matches.matches), vec!["L1:Top"]);

    let expanded_matches = find_result(&path, &expanded_find);
    assert_eq!(entry_refs(&expanded_matches.matches), vec!["L3:Top > Deep"]);
}

#[test]
fn info_returns_markdown_summary_and_capabilities() {
    let path = write_doc("info.md", "# A\nBody\n");
    let arguments = InfoArguments { options: None };
    let request = make_request(
        &path,
        Operation::Info,
        OperationArguments::Info(arguments.clone()),
    );

    let info = MarkdownAdapter.info(&request, &arguments).expect("info");

    assert!(info.display.contains("text/markdown"));
    assert_eq!(
        info.capabilities,
        vec![
            Operation::Outline,
            Operation::Read,
            Operation::Find,
            Operation::Info
        ]
    );
}

#[test]
fn non_utf8_document_returns_stable_encoding_error() {
    let path = write_bytes("bad.md", &[0xFF, 0xFE, 0x00]);
    let arguments = outline_args(6000, 1, None);
    let request = make_request(
        &path,
        Operation::Outline,
        OperationArguments::Outline(arguments.clone()),
    );

    let error = MarkdownAdapter
        .outline(&request, &arguments)
        .expect_err("non UTF-8 fails");

    assert_eq!(
        error.error().code,
        StableErrorCode::DocumentEncodingUnsupported
    );
}

#[test]
fn invoke_writes_protocol_envelope() {
    let path = write_doc("invoke.md", "# A\nBody\n");
    let request = make_request(
        &path,
        Operation::Outline,
        OperationArguments::Outline(outline_args(6000, 1, None)),
    );
    let input = serde_json::to_vec(&request).expect("request JSON");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = invoke_once(&MarkdownAdapter, input.as_slice(), &mut stdout, &mut stderr);

    assert_eq!(exit, AdapterExitCode::Success.code());
    assert!(stderr.is_empty());
    let response: ProtocolResponse = serde_json::from_slice(&stdout).expect("protocol response");
    match response {
        ProtocolResponse::Success(response) => {
            assert_eq!(response.operation, Operation::Outline);
            assert!(response.ok);
        }
        ProtocolResponse::Failure(_) => panic!("expected success"),
    }
}

fn entry_refs(entries: &[docnav_protocol::Entry]) -> Vec<&str> {
    entries.iter().map(|entry| entry.ref_id.as_str()).collect()
}

fn read_ref(path: &Path, ref_id: &str) -> docnav_protocol::ReadResult {
    let arguments = ReadArguments {
        ref_id: ref_id.to_owned(),
        limit_chars: positive(6000),
        page: positive(1),
        options: None,
    };
    let request = make_request(
        path,
        Operation::Read,
        OperationArguments::Read(arguments.clone()),
    );
    MarkdownAdapter
        .read(&request, &arguments)
        .expect("read ref")
}

fn read_ref_error(path: &Path, ref_id: &str) -> StableError {
    let arguments = ReadArguments {
        ref_id: ref_id.to_owned(),
        limit_chars: positive(6000),
        page: positive(1),
        options: None,
    };
    let request = make_request(
        path,
        Operation::Read,
        OperationArguments::Read(arguments.clone()),
    );
    MarkdownAdapter
        .read(&request, &arguments)
        .expect_err("read ref error")
        .into_error()
}

fn assert_ref_not_found(error: &StableError, ref_id: &str) {
    assert_eq!(error.code, StableErrorCode::RefNotFound);
    assert_eq!(
        error.details.get("ref").and_then(serde_json::Value::as_str),
        Some(ref_id)
    );
}

fn assert_canonical_ref(ref_id: &str) {
    assert!(!ref_id.contains("#1"));
    assert!(!ref_id.contains(&legacy_ordinal_prefix()));
}

fn legacy_ordinal_ref(prefix: &str, ordinal: u32) -> String {
    let ordinal = ordinal.to_string();
    [prefix, " [", "docnav", ":", ordinal.as_str(), "]"].concat()
}

fn legacy_ordinal_prefix() -> String {
    ["[", "docnav", ":"].concat()
}
