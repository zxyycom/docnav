use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use docnav_adapter_contracts::{Adapter, FindInput, InfoInput, OutlineInput, ReadInput};
use docnav_markdown::{markdown_adapter_definition, MarkdownAdapter};
use docnav_protocol::{
    positive_result, FindResult, ProtocolDiagnosticCode, ProtocolError, StructuredOutlineResult,
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
const DEFAULT_MAX_HEADING_LEVEL: i64 = 3;

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

fn outline_input(
    path: &Path,
    limit: u32,
    page: u32,
    max_heading_level: Option<i64>,
) -> OutlineInput {
    OutlineInput {
        document_path: path_string(path),
        limit: positive(limit),
        page: positive(page),
        max_heading_level: Some(max_heading_level.unwrap_or(DEFAULT_MAX_HEADING_LEVEL)),
    }
}

fn find_input(
    path: &Path,
    query: &str,
    limit: u32,
    page: u32,
    max_heading_level: Option<i64>,
) -> FindInput {
    FindInput {
        document_path: path_string(path),
        query: query.to_owned(),
        limit: positive(limit),
        page: positive(page),
        max_heading_level: Some(max_heading_level.unwrap_or(DEFAULT_MAX_HEADING_LEVEL)),
    }
}

fn outline_result(input: &OutlineInput) -> StructuredOutlineResult {
    MarkdownAdapter
        .outline(input)
        .expect("outline result")
        .into_structured()
        .expect("structured outline result")
}

fn find_result(input: &FindInput) -> FindResult {
    MarkdownAdapter.find(input).expect("find result")
}

fn entry_refs(entries: &[docnav_protocol::Entry]) -> Vec<&str> {
    entries.iter().map(|entry| entry.ref_id.as_str()).collect()
}

fn read_ref(path: &Path, ref_id: &str) -> docnav_protocol::ReadResult {
    read_ref_with_page(path, ref_id, 6000, 1)
}

fn read_ref_with_page(
    path: &Path,
    ref_id: &str,
    limit: u32,
    page: u32,
) -> docnav_protocol::ReadResult {
    let input = ReadInput {
        document_path: path_string(path),
        ref_id: ref_id.to_owned(),
        limit: positive(limit),
        page: positive(page),
    };
    MarkdownAdapter
        .read(&input)
        .unwrap_or_else(|_| panic!("read ref: {ref_id}"))
}

fn assert_cost_measurements(cost: &docnav_protocol::Cost, scope: &str, text: &str) {
    let expected = [
        docnav_text_cost::line_cost(text),
        docnav_text_cost::byte_cost(text),
        docnav_text_cost::token_cost(text),
    ];

    assert_eq!(cost.measurements.len(), expected.len());
    for (actual, expected) in cost.measurements.iter().zip(expected.iter()) {
        assert_eq!(actual.unit, expected.unit);
        assert_eq!(actual.value, expected.value);
        assert_eq!(expected.scope, None);
        assert_eq!(actual.scope.as_deref(), Some(scope));
    }
}

fn read_ref_error(path: &Path, ref_id: &str) -> ProtocolError {
    let input = ReadInput {
        document_path: path_string(path),
        ref_id: ref_id.to_owned(),
        limit: positive(6000),
        page: positive(1),
    };
    MarkdownAdapter.read(&input).unwrap_err().protocol_error()
}

fn assert_ref_not_found(error: &ProtocolError, ref_id: &str) {
    assert_eq!(
        error.code(),
        ProtocolDiagnosticCode::RefNotFound,
        "expected REF_NOT_FOUND for {ref_id}, got {:?}",
        error.code()
    );
    assert_eq!(
        error
            .details()
            .get("ref")
            .and_then(serde_json::Value::as_str),
        Some(ref_id)
    );
}

fn assert_ref_invalid(error: &ProtocolError, ref_id: &str) {
    assert_eq!(
        error.code(),
        ProtocolDiagnosticCode::RefInvalid,
        "expected REF_INVALID for {ref_id}, got {:?}",
        error.code()
    );
    assert_eq!(
        error
            .details()
            .get("ref")
            .and_then(serde_json::Value::as_str),
        Some(ref_id)
    );
    // reason 必须非空
    let reason = error
        .details()
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .expect("reason field");
    assert!(!reason.is_empty(), "reason must not be empty for {ref_id}");
}

fn assert_canonical_ref(ref_id: &str) {
    // canonical ref 使用 line 和 level 两个结构字段。
    let parts: Vec<_> = ref_id.split(':').collect();
    assert_eq!(
        parts.len(),
        3,
        "ref must contain type, line, and level fields: {:?}",
        ref_id
    );
    assert_eq!(parts[0], "H", "ref type must be H: {:?}", ref_id);
    // 数值字段使用 canonical 十进制形式。
    let after_hl = parts[1].strip_prefix('L').expect("line field");
    assert!(
        !after_hl.starts_with('0'),
        "line field must use canonical decimal digits: {:?}",
        ref_id
    );
    assert!(
        after_hl.chars().all(|ch| ch.is_ascii_digit()),
        "line field must be decimal digits: {:?}",
        ref_id
    );
    let level = parts[2].strip_prefix('H').expect("level field");
    assert!(
        level.len() == 1 && matches!(level, "1" | "2" | "3" | "4" | "5" | "6"),
        "level field must be 1-6: {:?}",
        ref_id
    );
}

#[path = "adapter/meta.rs"]
mod meta;
#[path = "adapter/options_error_display.rs"]
mod options_error_display;
#[path = "adapter/outline_ref.rs"]
mod outline_ref;
#[path = "adapter/paging_find.rs"]
mod paging_find;
