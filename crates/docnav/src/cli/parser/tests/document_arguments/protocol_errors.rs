use docnav_protocol::Operation;
use serde_json::{json, Value};

use crate::cli::OutputMode;
use crate::error::{AppError, DocnavExitCode};
use crate::output::{write_error, ErrorOutput};

use super::parse;

#[test]
fn unknown_document_argument_protocol_error_has_repair_context() {
    let error = parse([
        "outline",
        "docs/navigation.md",
        "--bogus",
        "--output",
        "protocol-json",
    ])
    .expect_err("unknown argument should fail");
    let output = protocol_error_output(&error, Operation::Outline);

    assert_protocol_error_context(&output, "unknown_argument", "--bogus");
    assert_expected_contains(&output, "supported option");
    assert_guidance_contains(&output, "Remove");
}

#[test]
fn extra_document_positional_protocol_error_has_repair_context() {
    let error = parse([
        "outline",
        "docs/navigation.md",
        "extra.md",
        "--output",
        "protocol-json",
    ])
    .expect_err("extra positional should fail");
    let output = protocol_error_output(&error, Operation::Outline);

    assert_protocol_error_context(&output, "extra_positional", "extra.md");
    assert_expected_contains(&output, "positional arguments");
    assert_guidance_contains(&output, "Remove");
}

#[test]
fn unsupported_info_page_protocol_error_has_repair_context() {
    let error = parse([
        "info",
        "docs/navigation.md",
        "--page",
        "2",
        "--output",
        "protocol-json",
    ])
    .expect_err("info should not accept page");
    let output = protocol_error_output(&error, Operation::Info);

    assert_protocol_error_context(&output, "unsupported_argument", "--page 2");
    assert_expected_contains(&output, "info <path>");
    assert_guidance_contains(&output, "Remove --page");
}

fn protocol_error_output(error: &AppError, operation: Operation) -> Value {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = write_error(ErrorOutput {
        error,
        output_mode: OutputMode::ProtocolJson,
        operation: Some(operation),
        stdout: &mut stdout,
        stderr: &mut stderr,
    });

    assert_eq!(exit, DocnavExitCode::InputError.code());
    assert!(stderr.is_empty());
    serde_json::from_slice(&stdout).expect("protocol-json failure parses")
}

fn assert_protocol_error_context(output: &Value, reason: &str, received: &str) {
    assert_eq!(output["ok"], false);
    assert_eq!(output["error"]["code"], "INVALID_REQUEST");
    assert_eq!(output["error"]["details"]["reason"], reason);
    assert_eq!(output["error"]["received"], json!(received));
    assert!(
        output["error"].get("expected").is_some(),
        "expected protocol error.expected to be present: {output}"
    );
    assert!(
        output["error"]
            .get("guidance")
            .and_then(Value::as_array)
            .is_some_and(|guidance| !guidance.is_empty()),
        "expected protocol error.guidance to be non-empty: {output}"
    );
}

fn assert_expected_contains(output: &Value, fragment: &str) {
    let expected = output["error"]["expected"]
        .as_array()
        .expect("expected is projected from accepted values");
    assert!(
        expected
            .iter()
            .filter_map(Value::as_str)
            .any(|value| value.contains(fragment)),
        "expected should contain {fragment:?}, got {expected:?}"
    );
}

fn assert_guidance_contains(output: &Value, fragment: &str) {
    let guidance = output["error"]["guidance"]
        .as_array()
        .expect("guidance is an array");
    assert!(
        guidance
            .iter()
            .filter_map(Value::as_str)
            .any(|value| value.contains(fragment)),
        "guidance should contain {fragment:?}, got {guidance:?}"
    );
}
