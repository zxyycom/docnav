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
#[path = "adapter/options_error_invoke_display.rs"]
mod options_error_invoke_display;
#[path = "adapter/outline_ref.rs"]
mod outline_ref;
#[path = "adapter/paging_find.rs"]
mod paging_find;
