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
fn manifest_and_probe_emit_protocol_json_schemas() {
    let manifest = run(&["manifest", "--output", "protocol-json"]);
    assert!(manifest.status.success());
    assert!(manifest.stderr.is_empty());
    let manifest_json: Value = serde_json::from_slice(&manifest.stdout).expect("manifest JSON");
    assert_eq!(manifest_json["adapter"]["id"], "docnav-markdown");
    assert!(manifest_json["protocol_version"].is_null());
    assert!(manifest_json["capabilities"]
        .as_array()
        .unwrap()
        .contains(&Value::from("find")));

    let path = write_doc("probe.md", "# A\n");
    let path = path_arg(&path);
    let probe = run(&["probe", &path, "--output", "protocol-json"]);
    assert!(probe.status.success());
    assert!(probe.stderr.is_empty());
    let probe_json: Value = serde_json::from_slice(&probe.stdout).expect("probe JSON");
    assert_eq!(probe_json["supported"], true);
    assert!(probe_json["entries"].is_null());
}

#[test]
fn direct_cli_supports_text_readable_json_and_protocol_json() {
    let path = write_doc("cli.md", "# Guide\nBody\n");
    let path = path_arg(&path);

    let text = run(&["outline", &path, "--output", "text"]);
    assert!(text.status.success());
    let text_stdout = String::from_utf8(text.stdout).expect("text stdout");
    assert!(text_stdout.contains("L1:Guide | H1"));
    assert!(!text_stdout.contains("#1"));
    assert_no_legacy_ordinal_suffix(&text_stdout);
    assert!(text_stdout.contains("page: null"));

    let readable = run(&["outline", &path, "--output", "readable-json"]);
    assert!(readable.status.success());
    let readable_stdout = std::str::from_utf8(&readable.stdout).expect("readable stdout");
    assert_no_legacy_ordinal_suffix(readable_stdout);
    let readable_json: Value = serde_json::from_slice(&readable.stdout).expect("readable JSON");
    let ref_id = readable_json["entries"][0]["ref"]
        .as_str()
        .expect("outline ref")
        .to_owned();
    assert_eq!(ref_id, "L1:Guide");
    assert!(readable_json["page"].is_null());

    let protocol = run(&["read", &path, "--ref", &ref_id, "--output", "protocol-json"]);
    assert!(protocol.status.success());
    assert!(protocol.stderr.is_empty());
    let protocol_stdout = std::str::from_utf8(&protocol.stdout).expect("protocol stdout");
    assert_no_legacy_ordinal_suffix(protocol_stdout);
    let protocol_json: Value = serde_json::from_slice(&protocol.stdout).expect("protocol JSON");
    assert_eq!(protocol_json["operation"], "read");
    assert_eq!(protocol_json["ok"], true);
    assert_eq!(
        protocol_json["result"]["ref"].as_str(),
        Some(ref_id.as_str())
    );
    assert_eq!(protocol_json["result"]["content_type"], "text/markdown");

    let read_text = run(&["read", &path, "--ref", &ref_id, "--output", "text"]);
    assert!(read_text.status.success());
    assert!(read_text.stderr.is_empty());
    let read_text_stdout = String::from_utf8(read_text.stdout).expect("read text stdout");
    assert!(read_text_stdout.contains(&format!("ref: {ref_id}")));
    assert!(read_text_stdout.contains("content_type: text/markdown"));
    assert_no_legacy_ordinal_suffix(&read_text_stdout);
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
fn readable_json_error_keeps_code_details_and_omits_protocol_envelope() {
    let path = write_doc("missing-ref.md", "# Guide\nBody\n");
    let path = path_arg(&path);
    let missing_ref = "L99:Missing";

    let output = run(&[
        "read",
        &path,
        "--ref",
        missing_ref,
        "--output",
        "readable-json",
    ]);

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = std::str::from_utf8(&output.stdout).expect("readable error stdout");
    assert_no_legacy_ordinal_suffix(stdout);
    let error_json: Value = serde_json::from_slice(&output.stdout).expect("readable error JSON");
    assert_eq!(error_json["code"], "REF_NOT_FOUND");
    assert_eq!(error_json["details"]["ref"], missing_ref);
    assert!(error_json["guidance"].as_array().is_some());
    assert!(error_json["protocol_version"].is_null());
    assert!(error_json["ok"].is_null());
}

#[test]
fn protocol_json_error_keeps_stable_ref_details() {
    let path = write_doc("missing-ref-protocol.md", "# Guide\nBody\n");
    let path = path_arg(&path);
    let missing_ref = "L99:Missing";

    let output = run(&[
        "read",
        &path,
        "--ref",
        missing_ref,
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
    assert_eq!(error_json["error"]["code"], "REF_NOT_FOUND");
    assert_eq!(error_json["error"]["details"]["ref"], missing_ref);
    assert!(error_json["result"].is_null());
}

#[test]
fn text_error_writes_readable_error_to_stdout() {
    let path = write_doc("missing-ref-text.md", "# Guide\nBody\n");
    let path = path_arg(&path);

    let output = run(&["read", &path, "--ref", "P:Guide", "--output", "text"]);

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("text error stdout");
    assert!(stdout.contains("error: REF_NOT_FOUND"));
    assert!(stdout.contains("details: ref=P:Guide"));
    assert_no_legacy_ordinal_suffix(&stdout);
}

#[test]
fn operation_missing_path_before_flag_reports_input_error() {
    let cases = vec![
        (
            vec!["outline", "--output", "text"],
            "outline requires <path>",
        ),
        (vec!["read", "--ref", "L1:Guide"], "read requires <path>"),
        (vec!["find", "--query", "target"], "find requires <path>"),
        (vec!["info", "--output", "text"], "info requires <path>"),
    ];

    for (args, message) in cases {
        let output = run(&args);

        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
        assert!(
            stderr.contains(message),
            "stderr {stderr:?} should include {message:?}"
        );
    }
}

fn assert_no_legacy_ordinal_suffix(value: &str) {
    assert!(!value.contains(&legacy_ordinal_prefix()));
}

fn legacy_ordinal_prefix() -> String {
    ["[", "docnav", ":"].concat()
}
