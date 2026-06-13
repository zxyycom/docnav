use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn write_doc(name: &str, content: &str) -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "docnav-markdown-cli-test-{}-{}",
        std::process::id(),
        id
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(name);
    fs::write(&path, content).expect("write temp document");
    path
}

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_docnav-markdown")
}

fn run(args: &[&str]) -> Output {
    Command::new(bin()).args(args).output().expect("run CLI")
}

fn run_with_stdin(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(bin())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn CLI");
    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    child.wait_with_output().expect("run CLI")
}

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[test]
fn direct_cli_and_invoke_share_find_execution_result() {
    let path = write_doc(
        "shared-find.md",
        "# Top\nintro\n\n#### Hidden\ntarget\n\n# Next\ntarget\n",
    );
    let path = path_arg(&path);

    let direct = run(&[
        "find",
        &path,
        "--query",
        "target",
        "--max-heading-level",
        "3",
        "--limit-chars",
        "6000",
        "--output",
        "readable-json",
    ]);
    assert!(direct.status.success());
    assert!(direct.stderr.is_empty());
    let direct_json: Value = serde_json::from_slice(&direct.stdout).expect("direct JSON");

    let request = serde_json::json!({
        "protocol_version": "0.1",
        "request_id": "shared-find",
        "operation": "find",
        "document": { "path": path },
        "arguments": {
            "query": "target",
            "limit_chars": 6000,
            "page": 1,
            "options": { "max_heading_level": 3 }
        }
    });
    let protocol = run_with_stdin(&["invoke"], &request.to_string());
    assert!(protocol.status.success());
    assert!(protocol.stderr.is_empty());
    let protocol_json: Value = serde_json::from_slice(&protocol.stdout).expect("protocol JSON");

    assert_eq!(protocol_json["operation"], "find");
    assert_eq!(protocol_json["ok"], true);
    assert_eq!(direct_json, protocol_json["result"]);
}

#[test]
fn readable_json_error_returns_ref_invalid_for_non_canonical_refs() {
    let path = write_doc("invalid-ref-read.md", "# Guide\nBody\n");
    let path = path_arg(&path);
    let invalid_ref = "L99:Missing";

    let output = run(&[
        "read",
        &path,
        "--ref",
        invalid_ref,
        "--output",
        "readable-json",
    ]);

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = std::str::from_utf8(&output.stdout).expect("readable error stdout");
    assert_no_legacy_ordinal_suffix(stdout);
    let error_json: Value = serde_json::from_slice(&output.stdout).expect("readable error JSON");
    // 旧格式 → REF_INVALID (不是 REF_NOT_FOUND)
    assert_eq!(error_json["code"], "REF_INVALID");
    assert_eq!(error_json["details"]["ref"], invalid_ref);
    assert!(error_json["details"]["reason"]
        .as_str()
        .is_some_and(|r| !r.is_empty()));
    assert!(error_json["guidance"].as_array().is_some());
    assert!(error_json["protocol_version"].is_null());
    assert!(error_json["ok"].is_null());
}

#[test]
fn readable_json_error_returns_ref_not_found_for_canonical_no_match() {
    let path = write_doc("canonical-no-match.md", "# Guide\nBody\n");
    let path = path_arg(&path);
    let canonical_ref = "H:L99:H1:I1";

    let output = run(&[
        "read",
        &path,
        "--ref",
        canonical_ref,
        "--output",
        "readable-json",
    ]);

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());
    let error_json: Value = serde_json::from_slice(&output.stdout).expect("readable error JSON");
    // canonical grammar 但无匹配 → REF_NOT_FOUND
    assert_eq!(error_json["code"], "REF_NOT_FOUND");
    assert_eq!(error_json["details"]["ref"], canonical_ref);
    assert!(error_json["guidance"].as_array().is_some());
}

#[test]
fn protocol_json_error_returns_ref_invalid_for_non_canonical_refs() {
    let path = write_doc("missing-ref-protocol.md", "# Guide\nBody\n");
    let path = path_arg(&path);
    let invalid_ref = "L99:Missing";

    let output = run(&[
        "read",
        &path,
        "--ref",
        invalid_ref,
        "--output",
        "protocol-json",
    ]);

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = std::str::from_utf8(&output.stdout).expect("protocol error stdout");
    assert_no_legacy_ordinal_suffix(stdout);
    let error_json: Value = serde_json::from_slice(&output.stdout).expect("protocol error JSON");
    assert_eq!(error_json["operation"], "read");
    assert_eq!(error_json["ok"], false);
    // 旧格式 → REF_INVALID
    assert_eq!(error_json["error"]["code"], "REF_INVALID");
    assert_eq!(error_json["error"]["details"]["ref"], invalid_ref);
    assert!(error_json["error"]["details"]["reason"]
        .as_str()
        .is_some_and(|r| !r.is_empty()));
    assert!(error_json["result"].is_null());
}

#[test]
fn text_error_returns_ref_invalid_for_non_canonical_ref() {
    let path = write_doc("missing-ref-text.md", "# Guide\nBody\n");
    let path = path_arg(&path);

    let output = run(&["read", &path, "--ref", "P:Guide", "--output", "text"]);

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("text error stdout");
    assert!(stdout.contains("error: REF_INVALID"));
    assert!(stdout.contains("ref=P:Guide"));
    assert!(stdout.contains("reason"));
    assert_no_legacy_ordinal_suffix(&stdout);
}

fn assert_no_legacy_ordinal_suffix(value: &str) {
    assert!(!value.contains(&legacy_ordinal_prefix()));
}

fn legacy_ordinal_prefix() -> String {
    ["[", "docnav", ":"].concat()
}
