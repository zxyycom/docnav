use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use docnav_adapter_sdk::{invoke_once, Adapter, AdapterExitCode};
use docnav_markdown::MarkdownAdapter;
use docnav_protocol::{
    positive_result, Document, FindArguments, InfoArguments, Operation, OperationArguments,
    OutlineArguments, ProtocolResponse, ReadArguments, RequestEnvelope, StableErrorCode,
    PROTOCOL_VERSION,
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

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
        options: max_heading_level.map(|level| {
            let mut options = docnav_protocol::Options::new();
            options.insert(
                "max_heading_level".to_owned(),
                serde_json::Value::from(level),
            );
            options
        }),
    }
}

#[test]
fn manifest_declares_markdown_v0_capabilities() {
    let manifest = MarkdownAdapter.manifest();

    manifest.validate_semantics().expect("manifest semantics");
    assert_eq!(manifest.adapter.id, "docnav-markdown");
    assert_eq!(manifest.formats[0].id, "markdown");
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
    assert!(result.entries[0].ref_id.starts_with("L1:Guide"));
    assert!(result.entries[1].ref_id.starts_with("L8:Guide > Install"));
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

    assert_eq!(refs, vec!["L2:A > B [docnav:1]", "L5:A > B [docnav:2]"]);
    let first = read_ref(&path, &refs[0]);
    let second = read_ref(&path, &refs[1]);
    assert!(first.content.contains("first"));
    assert!(second.content.contains("second"));
}

#[test]
fn read_reports_ref_not_found_for_missing_and_unsupported_refs() {
    let path = write_doc("refs.md", "# A\n## B\nfirst\n# A\n## B\nsecond\n");

    let missing = read_ref_error(&path, "L99:A > B [docnav:1]");
    assert_eq!(missing, StableErrorCode::RefNotFound);

    let unsupported_path_ref = read_ref_error(&path, "P:A > B");
    assert_eq!(unsupported_path_ref, StableErrorCode::RefNotFound);

    let loose_heading_ref = read_ref_error(&path, "L2:A > B");
    assert_eq!(loose_heading_ref, StableErrorCode::RefNotFound);
}

#[test]
fn read_paginates_unicode_without_splitting_characters() {
    let path = write_doc("unicode.md", "# A\n界界界abc\n");
    let ref_id = "L1:A [docnav:1]";
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
fn find_uses_nearest_visible_outline_ref_and_full_fallback() {
    let path = write_doc("find.md", "# First\nalpha\n\n# Second\ntarget\n");
    let arguments = FindArguments {
        query: "target".to_owned(),
        limit_chars: positive(6000),
        page: positive(1),
        options: None,
    };
    let request = make_request(
        &path,
        Operation::Find,
        OperationArguments::Find(arguments.clone()),
    );
    let result = MarkdownAdapter.find(&request, &arguments).expect("find");

    assert_eq!(result.matches.len(), 1);
    assert!(result.matches[0].ref_id.starts_with("L4:Second"));
    assert!(result.matches[0].display.contains("target"));

    let fallback_path = write_doc("fallback-find.md", "#### Deep\ntarget\n");
    let fallback_request = make_request(
        &fallback_path,
        Operation::Find,
        OperationArguments::Find(arguments.clone()),
    );
    let fallback = MarkdownAdapter
        .find(&fallback_request, &arguments)
        .expect("fallback find");
    assert_eq!(fallback.matches[0].ref_id, "doc:full");
}

#[test]
fn find_paginates_matches() {
    let path = write_doc("find-pages.md", "# A\ntarget 1\ntarget 2\ntarget 3\n");
    let arguments = FindArguments {
        query: "target".to_owned(),
        limit_chars: positive(24),
        page: positive(1),
        options: None,
    };
    let request = make_request(
        &path,
        Operation::Find,
        OperationArguments::Find(arguments.clone()),
    );

    let first = MarkdownAdapter
        .find(&request, &arguments)
        .expect("find page");

    assert_eq!(first.matches.len(), 1);
    assert_eq!(first.page, Some(positive(2)));
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

fn read_ref_error(path: &Path, ref_id: &str) -> StableErrorCode {
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
        .error()
        .code
}
