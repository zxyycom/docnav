use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use cli_config_resolution::{
    FieldIdentity, Source, SourceCandidate, SourceId, SourceKind, SourceLocator,
};
use docnav_navigation::{DOCUMENT_CLI_SOURCE_ID, DOCUMENT_CLI_SOURCE_PRIORITY};
use docnav_protocol::Operation;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::cli::{DocumentCommand, OutputMode};
use crate::config::{ConfigContext, CoreConfig};
use crate::error::AppResult;
use crate::output::{write_error, write_outcome, CommandOutcome, ErrorOutput};
use crate::project_context::{ProjectContext, SelectedConfigPath, SelectedConfigPaths};

pub(super) fn write_protocol_json(outcome: CommandOutcome) -> Value {
    let (exit_code, output) = write_protocol_json_with_exit(outcome);
    assert_eq!(exit_code, 0);
    output
}

pub(super) fn markdown_project(name: &str, content: &str) -> (TempWorkspace, PathBuf) {
    let workspace = temp_workspace(name);
    let project_root = workspace.path().join("project");
    let docs_dir = project_root.join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(docs_dir.join("guide.md"), content).unwrap();
    (workspace, project_root)
}

pub(super) fn write_native_option_config(path: &Path, value: Value) {
    write_config_file(
        path,
        json!({
            "options": {
                "docnav-markdown": {
                    "max_heading_level": value
                }
            }
        }),
    );
}

pub(super) fn write_config_file(path: &Path, value: Value) {
    fs::create_dir_all(path.parent().expect("config parent")).unwrap();
    fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

pub(super) fn default_context(project_root: PathBuf) -> ConfigContext {
    ConfigContext {
        project: project_context(project_root.clone(), project_root),
        project_config: CoreConfig::default(),
        user_config: CoreConfig::default(),
    }
}

pub(super) fn outline_command(
    max_heading_level: Option<u32>,
    adapter: Option<&str>,
) -> DocumentCommand {
    let mut candidates = vec![
        cli_value_candidate("docnav.defaults.pagination.limit", "--limit", json!(80)),
        cli_value_candidate("docnav.defaults.output", "--output", json!("protocol-json")),
    ];
    if let Some(value) = max_heading_level {
        candidates.push(cli_value_candidate(
            "docnav.adapters.docnav-markdown.options.max_heading_level",
            "--max-heading-level",
            json!(value),
        ));
    }
    if let Some(adapter) = adapter {
        candidates.push(cli_value_candidate(
            "docnav.defaults.adapter",
            "--adapter",
            json!(adapter),
        ));
    }
    DocumentCommand {
        operation: Operation::Outline,
        path: "docs/guide.md".to_owned(),
        ref_id: None,
        query: None,
        cli_source: cli_source(candidates),
        invocation_log: None,
        invocation_log_content_root: None,
        config_paths: Default::default(),
    }
}

pub(super) fn read_command(ref_id: &str) -> DocumentCommand {
    DocumentCommand {
        operation: Operation::Read,
        path: "docs/guide.md".to_owned(),
        ref_id: Some(ref_id.to_owned()),
        query: None,
        cli_source: cli_source(vec![
            cli_value_candidate("docnav.defaults.pagination.limit", "--limit", json!(80)),
            cli_value_candidate("docnav.defaults.output", "--output", json!("protocol-json")),
        ]),
        invocation_log: None,
        invocation_log_content_root: None,
        config_paths: Default::default(),
    }
}

pub(super) fn set_cli_value(
    command: &mut DocumentCommand,
    identity: &str,
    flag: &str,
    value: Value,
) {
    let mut candidates = command
        .cli_source
        .candidates()
        .iter()
        .filter(|candidate| candidate.field().as_str() != identity)
        .cloned()
        .collect::<Vec<_>>();
    candidates.push(cli_value_candidate(identity, flag, value));
    command.cli_source = cli_source(candidates);
}

pub(super) fn cli_source(candidates: Vec<SourceCandidate>) -> Box<Source> {
    Box::new(
        Source::new(
            SourceId::new(DOCUMENT_CLI_SOURCE_ID).unwrap(),
            SourceKind::Cli,
            DOCUMENT_CLI_SOURCE_PRIORITY,
            candidates,
        )
        .unwrap(),
    )
}

fn cli_value_candidate(identity: &str, flag: &str, value: Value) -> SourceCandidate {
    SourceCandidate::value(
        FieldIdentity::new(identity).unwrap(),
        SourceLocator::CliFlag(flag.to_owned()),
        value,
    )
}

pub(super) fn read_jsonl_events(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

pub(super) fn event_named<'a>(events: &'a [Value], event: &str) -> &'a Value {
    events
        .iter()
        .find(|value| value["event"] == event)
        .unwrap_or_else(|| panic!("missing event {event}: {events:#?}"))
}

pub(super) fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn test_sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut text = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(text, "{byte:02x}");
    }
    text
}

pub(super) fn write_protocol_json_with_exit(outcome: CommandOutcome) -> (i32, Value) {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit_code = write_outcome(outcome, &mut stdout, &mut stderr);
    assert!(
        stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&stderr)
    );
    (exit_code, serde_json::from_slice(&stdout).unwrap())
}

pub(super) fn write_outcome_text_with_exit(outcome: CommandOutcome) -> (i32, String, String) {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit_code = write_outcome(outcome, &mut stdout, &mut stderr);
    (
        exit_code,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

pub(super) fn parse_single_json_value(stdout: &str) -> Value {
    let mut values = serde_json::Deserializer::from_str(stdout).into_iter::<Value>();
    let value = values
        .next()
        .expect("stdout should contain one JSON value")
        .expect("stdout JSON should parse");
    assert!(
        values.next().is_none(),
        "stdout should contain a single JSON value: {stdout}"
    );
    value
}

pub(super) fn assert_no_invocation_event_text(stdout: &str) {
    for forbidden in [
        "operation_completed",
        "operation_failed",
        "content_captured",
        "content_capture_failed",
        "correlation_id",
        "\"event\"",
    ] {
        assert!(
            !stdout.contains(forbidden),
            "stdout should not contain invocation log text {forbidden:?}: {stdout}"
        );
    }
}

pub(super) fn write_document_result(
    result: AppResult<CommandOutcome>,
    operation: Operation,
    output_mode: OutputMode,
) -> (i32, Value) {
    match result {
        Ok(outcome) => write_protocol_json_with_exit(outcome),
        Err(error) => {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let exit_code = write_error(ErrorOutput {
                error: &error,
                output_mode,
                operation: Some(operation),
                stdout: &mut stdout,
                stderr: &mut stderr,
            });
            assert!(
                stderr.is_empty(),
                "stderr: {}",
                String::from_utf8_lossy(&stderr)
            );
            (exit_code, serde_json::from_slice(&stdout).unwrap())
        }
    }
}

pub(super) fn first_entry_label(output: &Value) -> Option<&str> {
    output["result"]["entries"]
        .as_array()
        .and_then(|entries| entries.first())
        .and_then(|entry| entry["label"].as_str())
}

pub(super) fn entry_labels(output: &Value) -> Vec<&str> {
    output["result"]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["label"].as_str().unwrap())
        .collect()
}

pub(super) fn project_context(project_root: PathBuf, cwd: PathBuf) -> ProjectContext {
    ProjectContext {
        cwd,
        config_paths: SelectedConfigPaths {
            project: SelectedConfigPath::default(project_root.join(".docnav").join("docnav.json")),
            user: SelectedConfigPath::default(
                project_root.join(".docnav-user").join("docnav.json"),
            ),
        },
        project_root,
    }
}

pub(super) struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(super) fn temp_workspace(name: &str) -> TempWorkspace {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir()
        .join("docnav-runtime-tests")
        .join(format!("{name}-{suffix}"));
    fs::create_dir_all(&path).unwrap();
    TempWorkspace { path }
}

pub(super) struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("stdout closed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) struct LogAbsentWriter<'a> {
    log_path: &'a Path,
    bytes: Vec<u8>,
}

impl<'a> LogAbsentWriter<'a> {
    pub(super) fn new(log_path: &'a Path) -> Self {
        Self {
            log_path,
            bytes: Vec::new(),
        }
    }

    pub(super) fn into_string(self) -> String {
        String::from_utf8(self.bytes).unwrap()
    }
}

impl Write for LogAbsentWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        assert!(
            !self.log_path.exists(),
            "output failure log must be written after fallback output error projection"
        );
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
