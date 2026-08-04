use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::super::super::{AdapterRuntime, DocnavRuntime, DocumentRequest};
use super::super::support::*;
use crate::output::write_outcome;

const CONCURRENT_CHILD_ENV: &str = "DOCNAV_TEST_INVOCATION_LOG_CONCURRENT_CHILD";
const CONCURRENT_LOG_PATH_ENV: &str = "DOCNAV_TEST_INVOCATION_LOG_PATH";
const CONCURRENT_DOCUMENT_PATH_ENV: &str = "DOCNAV_TEST_INVOCATION_DOCUMENT_PATH";
const CONCURRENT_START_PATH_ENV: &str = "DOCNAV_TEST_INVOCATION_START_PATH";
const CONCURRENT_PROCESS_COUNT: usize = 24;
const INVOCATIONS_PER_PROCESS: usize = 4;

#[test]
fn invocation_logging_enabled_success_writes_jsonl_with_request_id() {
    let (_workspace, project_root) = markdown_project("invocation-success", "# One\n");
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("invocation.jsonl");
    let mut command = outline_command(None, None);
    command.invocation_log = Some(".log/invocation.jsonl".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    assert!(
        !log_path.exists(),
        "success must not be logged before output is written"
    );
    let (exit_code, stdout) = write_protocol_json_with_exit(outcome);
    let events = read_jsonl_events(&log_path);

    assert_eq!(exit_code, 0);
    assert_eq!(stdout["ok"], true);
    assert!(
        !String::from_utf8(serde_json::to_vec(&stdout).unwrap())
            .unwrap()
            .contains("operation_completed"),
        "stdout should not contain invocation log events"
    );
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["schema_version"], "0.1");
    assert_eq!(events[0]["event"], "operation_completed");
    assert_eq!(events[0]["status"], "success");
    assert_eq!(events[0]["operation"], "outline");
    assert_eq!(events[0]["adapter_id"], "docnav-markdown");
    assert!(events[0]["request_id"].as_str().is_some());
}

#[test]
fn concurrent_processes_append_complete_parseable_jsonl_events() {
    if std::env::var_os(CONCURRENT_CHILD_ENV).is_some() {
        run_concurrent_log_child();
        return;
    }

    let (workspace, project_root) = markdown_project("invocation-concurrent-processes", "# One\n");
    let document_path = project_root.join("docs").join("guide.md");
    let log_path = project_root.join(".log").join("concurrent.jsonl");
    let start_path = workspace.path().join("start");
    let children = spawn_concurrent_log_children(&document_path, &log_path, &start_path);

    fs::write(&start_path, b"start").unwrap();
    for child in children {
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "child failed: status={}; stdout={}; stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let log_text = fs::read_to_string(&log_path).unwrap();
    let non_empty_lines = log_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    let events = non_empty_lines
        .iter()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(
        events.len(),
        CONCURRENT_PROCESS_COUNT * INVOCATIONS_PER_PROCESS
    );
    assert!(events
        .iter()
        .all(|event| event["event"] == "operation_completed"));
}

#[test]
fn invocation_output_write_failure_logs_output_projection_without_completion() {
    let (_workspace, project_root) = markdown_project("invocation-output-write-failure", "# One\n");
    let context = default_context(project_root.clone());
    let log_path = project_root.join(".log").join("output-failure.jsonl");
    let mut command = outline_command(None, None);
    command.invocation_log = Some(".log/output-failure.jsonl".to_owned());
    let request = DocumentRequest::from_config_context(command, context);

    let outcome = AdapterRuntime.execute_document(request).unwrap();
    assert!(
        !log_path.exists(),
        "completion must not be logged before output write"
    );
    let mut stdout = FailingWriter;
    let mut stderr = LogAbsentWriter::new(&log_path);
    let exit_code = write_outcome(outcome, &mut stdout, &mut stderr);
    let stderr = stderr.into_string();
    let events = read_jsonl_events(&log_path);

    assert_ne!(exit_code, 0);
    assert!(
        !stderr.is_empty(),
        "writer failure should still report output failure"
    );
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event"], "operation_failed");
    assert_eq!(events[0]["failure"]["layer"], "output_projection");
    assert!(
        events
            .iter()
            .all(|event| event["event"] != "operation_completed"),
        "output failure must not log completion: {events:#?}"
    );
}

fn spawn_concurrent_log_children(
    document_path: &Path,
    log_path: &Path,
    start_path: &Path,
) -> Vec<Child> {
    let current_test = std::env::current_exe().unwrap();
    (0..CONCURRENT_PROCESS_COUNT)
        .map(|_| {
            Command::new(&current_test)
                .arg("concurrent_processes_append_complete_parseable_jsonl_events")
                .arg("--nocapture")
                .env(CONCURRENT_CHILD_ENV, "1")
                .env(CONCURRENT_DOCUMENT_PATH_ENV, document_path)
                .env(CONCURRENT_LOG_PATH_ENV, log_path)
                .env(CONCURRENT_START_PATH_ENV, start_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect()
}

fn run_concurrent_log_child() {
    let document_path = env_path(CONCURRENT_DOCUMENT_PATH_ENV);
    let log_path = env_path(CONCURRENT_LOG_PATH_ENV);
    let start_path = env_path(CONCURRENT_START_PATH_ENV);
    let deadline = Instant::now() + Duration::from_secs(30);
    while !start_path.exists() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for start barrier"
        );
        thread::sleep(Duration::from_millis(2));
    }

    for _ in 0..INVOCATIONS_PER_PROCESS {
        let args = vec![
            "outline".to_owned(),
            "--output".to_owned(),
            "protocol-json".to_owned(),
            "--invocation-log".to_owned(),
            log_path.display().to_string(),
            document_path.display().to_string(),
        ];
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit_code = crate::run(args, io::empty(), &mut stdout, &mut stderr);
        assert_eq!(exit_code, 0);
        assert!(
            stderr.is_empty(),
            "stderr: {}",
            String::from_utf8_lossy(&stderr)
        );
        let output: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
        assert_eq!(output["ok"], true);
    }
}

fn env_path(name: &str) -> PathBuf {
    std::env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("missing {name}"))
}
