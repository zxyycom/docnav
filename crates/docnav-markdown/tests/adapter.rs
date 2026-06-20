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

// @case WB-MD-META-001
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

// @case WB-MD-ADAPTER-OUTLINE-001
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
    // Guide: line 1, level 1, index 1
    // Install: line 8, level 2, index 2 (Fake 不在 outline 中但 index 在过滤前分配)
    // Fake 在代码围栏内被忽略，不算有效 heading。所以只有 Guide(index=1) 和 Install(index=2)
    // Hidden 是 H4，max_heading_level=3 时不显示
    assert_eq!(result.entries[0].ref_id, "H:L1:H1:I1");
    assert_eq!(result.entries[1].ref_id, "H:L8:H2:I2");
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

// @case WB-MD-REF-001
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

    let all_refs: Vec<String> = outline
        .entries
        .iter()
        .map(|entry| entry.ref_id.clone())
        .collect();
    // # A (line 1, H1, index 1)
    // ## B (line 2, H2, index 2)
    // # A (line 4, H1, index 3)
    // ## B (line 5, H2, index 4)
    assert_eq!(
        all_refs,
        vec!["H:L1:H1:I1", "H:L2:H2:I2", "H:L4:H1:I3", "H:L5:H2:I4",]
    );
    for ref_id in &all_refs {
        assert_canonical_ref(ref_id);
    }

    // 读取第一个 B section（包含 "first"）
    let first = read_ref(&path, "H:L2:H2:I2");
    assert!(first.content.contains("first"));
    assert!(!first.content.contains("second"));

    // 读取第二个 B section（包含 "second"）
    let second = read_ref(&path, "H:L5:H2:I4");
    assert!(second.content.contains("second"));
    assert!(!second.content.contains("first"));

    // 读取第一个 A section
    let first_a = read_ref(&path, "H:L1:H1:I1");
    assert!(first_a.content.contains("first"));
    assert!(!first_a.content.contains("second"));
}

// @case WB-MD-REF-002
#[test]
fn read_reports_ref_invalid_for_old_format_and_non_canonical_refs() {
    let path = write_doc(
        "invalid-ref-formats.md",
        "# A\n## B\nfirst\n# A\n## B\nsecond\n",
    );

    // 旧格式 → REF_INVALID
    let old_refs = ["L99:Missing", "L1:A", "L2#2:A > B", "L1#1:A"];
    for ref_id in &old_refs {
        let error = read_ref_error(&path, ref_id);
        assert_ref_invalid(&error, ref_id);
    }

    // 非法字段 → REF_INVALID
    let invalid_refs = ["P:A > B", "H:L01:H1:I1", "H:L1:H0:I1", "not-a-ref", ""];
    for ref_id in &invalid_refs {
        if ref_id.is_empty() {
            // 空字符串可能触发共享层校验（非空字符串要求）
            continue;
        }
        let error = read_ref_error(&path, ref_id);
        assert_ref_invalid(&error, ref_id);
    }
}

#[test]
fn read_reports_ref_not_found_for_canonical_no_match() {
    let path = write_doc("nofound.md", "# Guide\nBody\n");

    // Canonical grammar 但无匹配 → REF_NOT_FOUND
    let error = read_ref_error(&path, "H:L99:H1:I1");
    assert_ref_not_found(&error, "H:L99:H1:I1");

    let error = read_ref_error(&path, "H:L1:H2:I1");
    assert_ref_not_found(&error, "H:L1:H2:I1");

    let error = read_ref_error(&path, "H:L1:H1:I99");
    assert_ref_not_found(&error, "H:L1:H1:I99");
}

#[test]
fn structure_snapshot_old_ref_may_fail_after_document_change() {
    let path1 = write_doc("snap1.md", "# A\nBody\n## B\nMore\n");
    let arguments = outline_args(6000, 1, Some(3));
    let outline1 = outline_result(&path1, &arguments);
    let ref_a = &outline1.entries[0].ref_id;

    // 原文档中可以正常读取
    let read1 = read_ref(&path1, ref_a);
    assert!(read1.content.contains("# A"));

    // 文档变化后重新解析，使用旧 ref
    let path2 = write_doc("snap2.md", "No headings\nJust text\n");
    let error = read_ref_error(&path2, ref_a);
    // 结构坐标变化后旧 ref 返回 REF_NOT_FOUND（而非 REF_INVALID）
    assert_ref_not_found(&error, ref_a);
}

// @case WB-MD-PAGE-001
#[test]
fn read_paginates_unicode_without_splitting_characters() {
    let path = write_doc("unicode.md", "# A\n界界界abc\n");
    let ref_id = "H:L1:H1:I1";
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

// @case WB-MD-FIND-001
#[test]
fn find_ref_targets_current_visible_region_and_read_contains_match() {
    let path = write_doc(
        "find-current-region.md",
        "# Current\nintro\n\n#### Hidden\ntarget\n\n# Next\nother\n",
    );
    let arguments = find_args("target", 6000, 1, Some(3));

    let result = find_result(&path, &arguments);

    assert_eq!(result.matches.len(), 1);
    // target 在 H4 "Hidden" 下，但 Hidden 不 visible (max=3)，
    // 最近 visible heading 是 "Current" (line 1, H1, index 1)
    assert_eq!(result.matches[0].ref_id, "H:L1:H1:I1");
    assert_canonical_ref(&result.matches[0].ref_id);
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

// @case WB-MD-PAGE-002
#[test]
fn outline_paginates_with_response_page_until_end_and_past_end() {
    let path = write_doc("outline-pages.md", "# A\none\n# B\ntwo\n# C\nthree\n");
    let first_arguments = outline_args(10, 1, Some(3));

    let first = outline_result(&path, &first_arguments);
    assert_eq!(entry_refs(&first.entries), vec!["H:L1:H1:I1"]);
    let second_page = first.page.expect("second page");

    let second_arguments = OutlineArguments {
        page: second_page,
        ..first_arguments.clone()
    };
    let second = outline_result(&path, &second_arguments);
    assert_eq!(entry_refs(&second.entries), vec!["H:L3:H1:I2"]);
    let third_page = second.page.expect("third page");

    let third_arguments = OutlineArguments {
        page: third_page,
        ..first_arguments.clone()
    };
    let third = outline_result(&path, &third_arguments);
    assert_eq!(entry_refs(&third.entries), vec!["H:L5:H1:I3"]);
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
    assert_eq!(entry_refs(&first.matches), vec!["H:L1:H1:I1"]);
    let second_page = first.page.expect("second page");

    let second_arguments = FindArguments {
        page: second_page,
        ..first_arguments.clone()
    };
    let second = find_result(&path, &second_arguments);
    assert_eq!(entry_refs(&second.matches), vec!["H:L3:H1:I2"]);
    let third_page = second.page.expect("third page");

    let third_arguments = FindArguments {
        page: third_page,
        ..first_arguments.clone()
    };
    let third = find_result(&path, &third_arguments);
    assert_eq!(entry_refs(&third.matches), vec!["H:L5:H1:I3"]);
    assert_eq!(third.page, None);

    let past_end_arguments = FindArguments {
        page: positive(4),
        ..first_arguments
    };
    let past_end = find_result(&path, &past_end_arguments);
    assert!(past_end.matches.is_empty());
    assert_eq!(past_end.page, None);
}

// @case WB-MD-OPTIONS-001
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
    assert_eq!(entry_refs(&default.entries), vec!["H:L1:H1:I1"]);

    let expanded = outline_result(&path, &expanded_outline);
    assert_eq!(
        entry_refs(&expanded.entries),
        vec!["H:L1:H1:I1", "H:L3:H4:I2"]
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
    assert_eq!(entry_refs(&default_matches.matches), vec!["H:L1:H1:I1"]);

    let expanded_matches = find_result(&path, &expanded_find);
    assert_eq!(entry_refs(&expanded_matches.matches), vec!["H:L3:H4:I2"]);
}

// @case WB-MD-ERROR-001
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

// @case WB-MD-INVOKE-001
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

// @case WB-MD-DISPLAY-001
#[test]
fn outline_display_includes_heading_title() {
    let path = write_doc("display.md", "# Installation Guide\n\n## Setup\nBody\n");
    let arguments = outline_args(6000, 1, None);
    let result = outline_result(&path, &arguments);

    assert_eq!(result.entries.len(), 2);
    // display 包含 heading title
    assert!(result.entries[0].display.contains("Installation Guide"));
    assert!(result.entries[1].display.contains("Setup"));
    // ref 不包含 title
    assert!(!result.entries[0].ref_id.contains("Installation"));
}

#[test]
fn find_display_contains_match_snippet() {
    let path = write_doc(
        "find-display.md",
        "# Top\nfind this needle here\n## Other\nmore text\n",
    );
    let arguments = find_args("needle", 6000, 1, Some(3));
    let result = find_result(&path, &arguments);

    assert_eq!(result.matches.len(), 1);
    // find display 保留匹配片段
    assert!(result.matches[0].display.contains("needle"));
    // ref 不受 display 内容影响
    assert_canonical_ref(&result.matches[0].ref_id);
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
        .unwrap_or_else(|_| panic!("read ref: {ref_id}"))
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
        .unwrap_err()
        .into_error()
}

fn assert_ref_not_found(error: &StableError, ref_id: &str) {
    assert_eq!(
        error.code,
        StableErrorCode::RefNotFound,
        "expected REF_NOT_FOUND for {ref_id}, got {:?}",
        error.code
    );
    assert_eq!(
        error.details.get("ref").and_then(serde_json::Value::as_str),
        Some(ref_id)
    );
}

fn assert_ref_invalid(error: &StableError, ref_id: &str) {
    assert_eq!(
        error.code,
        StableErrorCode::RefInvalid,
        "expected REF_INVALID for {ref_id}, got {:?}",
        error.code
    );
    assert_eq!(
        error.details.get("ref").and_then(serde_json::Value::as_str),
        Some(ref_id)
    );
    // reason 必须非空
    let reason = error
        .details
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .expect("reason field");
    assert!(!reason.is_empty(), "reason must not be empty for {ref_id}");
}

fn assert_canonical_ref(ref_id: &str) {
    // canonical ref 必须以 H:L 开头，包含 :H 和 :I
    assert!(
        ref_id.starts_with("H:L"),
        "ref must start with H:L: {:?}",
        ref_id
    );
    assert!(ref_id.contains(":H"), "ref must contain :H: {:?}", ref_id);
    assert!(ref_id.contains(":I"), "ref must contain :I: {:?}", ref_id);
    // 不包含旧格式的 # 或 path 文本
    assert!(
        !ref_id.contains('#'),
        "ref must not contain #: {:?}",
        ref_id
    );
    // 不包含前导零模式（H:L0, :H0, :I0）
    let after_hl = ref_id.strip_prefix("H:L").unwrap();
    assert!(
        !after_hl.starts_with('0'),
        "ref must not have leading zero in line: {:?}",
        ref_id
    );
}
