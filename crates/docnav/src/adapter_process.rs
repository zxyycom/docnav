use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use serde::Serialize;
use serde_json::Value;

use crate::project_paths::path_to_slash;
use crate::registry::AdapterRecord;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterProcessOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterProcessError {
    pub reason: String,
    pub exit_code: Option<i32>,
    pub stderr: String,
}

pub fn run_manifest(
    project_root: &Path,
    record: &AdapterRecord,
) -> Result<AdapterProcessOutput, AdapterProcessError> {
    run_no_stdin(
        project_root,
        record,
        &["manifest", "--output", "protocol-json"],
    )
}

pub fn run_probe(
    project_root: &Path,
    record: &AdapterRecord,
    path: &str,
) -> Result<AdapterProcessOutput, AdapterProcessError> {
    run_no_stdin(
        project_root,
        record,
        &["probe", path, "--output", "protocol-json"],
    )
}

pub fn run_invoke<T: Serialize>(
    project_root: &Path,
    record: &AdapterRecord,
    request: &T,
) -> Result<AdapterProcessOutput, AdapterProcessError> {
    let input = serde_json::to_vec(request).map_err(|error| AdapterProcessError {
        reason: format!("failed to serialize invoke request: {error}"),
        exit_code: None,
        stderr: String::new(),
    })?;

    let mut child = Command::new(&record.command_path)
        .arg("invoke")
        .current_dir(project_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| AdapterProcessError {
            reason: format!(
                "failed to start adapter command {}: {error}",
                path_to_slash(&record.command_path)
            ),
            exit_code: None,
            stderr: String::new(),
        })?;

    let Some(mut stdin) = child.stdin.take() else {
        return Err(AdapterProcessError {
            reason: "failed to open adapter stdin".to_owned(),
            exit_code: None,
            stderr: String::new(),
        });
    };
    stdin
        .write_all(&input)
        .map_err(|error| AdapterProcessError {
            reason: format!("failed to write adapter stdin: {error}"),
            exit_code: None,
            stderr: String::new(),
        })?;
    drop(stdin);

    Ok(collect_output(child.wait_with_output().map_err(
        |error| AdapterProcessError {
            reason: format!("failed to wait for adapter command: {error}"),
            exit_code: None,
            stderr: String::new(),
        },
    )?))
}

pub fn parse_single_json(stdout: &str) -> Result<Value, String> {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Err("stdout is empty".to_owned());
    }
    serde_json::from_str::<Value>(trimmed).map_err(|error| format!("stdout is not JSON: {error}"))
}

fn run_no_stdin(
    project_root: &Path,
    record: &AdapterRecord,
    args: &[&str],
) -> Result<AdapterProcessOutput, AdapterProcessError> {
    let output = Command::new(&record.command_path)
        .args(args)
        .current_dir(project_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| AdapterProcessError {
            reason: format!(
                "failed to start adapter command {}: {error}",
                path_to_slash(&record.command_path)
            ),
            exit_code: None,
            stderr: String::new(),
        })?;

    let output = collect_output(output);
    if output.exit_code == Some(0) {
        Ok(output)
    } else {
        Err(AdapterProcessError {
            reason: format!(
                "adapter exited with status {}",
                status_label(output.exit_code)
            ),
            exit_code: output.exit_code,
            stderr: output.stderr,
        })
    }
}

fn collect_output(output: std::process::Output) -> AdapterProcessOutput {
    AdapterProcessOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status.code(),
    }
}

fn status_label(exit_code: Option<i32>) -> String {
    exit_code
        .map(|code| code.to_string())
        .unwrap_or_else(|| "terminated by signal".to_owned())
}
