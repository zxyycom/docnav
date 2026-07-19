use docnav_protocol::Operation;
use serde_json::Value;

use crate::cli::OutputMode;
use crate::error::AppResult;
use crate::output::{write_error, write_outcome, CommandOutcome, ErrorOutput};

pub(in crate::runtime::tests) fn write_protocol_json(outcome: CommandOutcome) -> Value {
    let (exit_code, output) = write_protocol_json_with_exit(outcome);
    assert_eq!(exit_code, 0);
    output
}

pub(in crate::runtime::tests) fn write_protocol_json_with_exit(
    outcome: CommandOutcome,
) -> (i32, Value) {
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

pub(in crate::runtime::tests) fn write_outcome_text_with_exit(
    outcome: CommandOutcome,
) -> (i32, String, String) {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit_code = write_outcome(outcome, &mut stdout, &mut stderr);
    (
        exit_code,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

pub(in crate::runtime::tests) fn parse_single_json_value(stdout: &str) -> Value {
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

pub(in crate::runtime::tests) fn assert_no_invocation_event_text(stdout: &str) {
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

pub(in crate::runtime::tests) fn write_document_result(
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

pub(in crate::runtime::tests) fn first_entry_label(output: &Value) -> Option<&str> {
    output["result"]["entries"]
        .as_array()
        .and_then(|entries| entries.first())
        .and_then(|entry| entry["label"].as_str())
}

pub(in crate::runtime::tests) fn entry_labels(output: &Value) -> Vec<&str> {
    output["result"]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["label"].as_str().unwrap())
        .collect()
}
