use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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
    assert!(text_stdout.contains("L1:Guide [docnav:1]"));
    assert!(text_stdout.contains("page: null"));

    let readable = run(&["outline", &path, "--output", "readable-json"]);
    assert!(readable.status.success());
    let readable_json: Value = serde_json::from_slice(&readable.stdout).expect("readable JSON");
    let ref_id = readable_json["entries"][0]["ref"]
        .as_str()
        .expect("outline ref")
        .to_owned();
    assert!(readable_json["page"].is_null());

    let protocol = run(&["read", &path, "--ref", &ref_id, "--output", "protocol-json"]);
    assert!(protocol.status.success());
    assert!(protocol.stderr.is_empty());
    let protocol_json: Value = serde_json::from_slice(&protocol.stdout).expect("protocol JSON");
    assert_eq!(protocol_json["operation"], "read");
    assert_eq!(protocol_json["ok"], true);
    assert_eq!(protocol_json["result"]["content_type"], "text/markdown");
}

#[test]
fn readable_json_error_keeps_code_details_and_omits_protocol_envelope() {
    let path = write_doc("missing-ref.md", "# Guide\nBody\n");
    let path = path_arg(&path);

    let output = run(&[
        "read",
        &path,
        "--ref",
        "L99:Missing [docnav:1]",
        "--output",
        "readable-json",
    ]);

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());
    let error_json: Value = serde_json::from_slice(&output.stdout).expect("readable error JSON");
    assert_eq!(error_json["code"], "REF_NOT_FOUND");
    assert_eq!(error_json["details"]["ref"], "L99:Missing [docnav:1]");
    assert!(error_json["guidance"].as_array().is_some());
    assert!(error_json["protocol_version"].is_null());
    assert!(error_json["ok"].is_null());
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
}
