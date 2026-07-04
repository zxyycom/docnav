use super::super::parse;
use crate::cli::{CliCommand, OutputMode};
use crate::error::{AppError, DocnavExitCode};
use crate::output::{write_error, ErrorOutput};
use docnav_protocol::Operation;
use serde_json::{json, Value};

// @case WB-CORE-ARGS-001
#[test]
fn used_known_argument_stays_strict() {
    let error = parse(["outline", "doc.md", "--page", "0"]).expect_err("page is invalid");
    assert_diagnostic(error, "--page", "invalid_value");
}

#[test]
fn explicit_pagination_value_is_parsed() {
    let parsed = parse(["outline", "doc.md", "--pagination", "disabled"])
        .expect("parse pagination disabled");

    match parsed.command {
        CliCommand::Document(command) => {
            assert_eq!(command.pagination_enabled, Some(false));
        }
        command => panic!("expected document command, got {command:?}"),
    }
}

#[test]
fn explicit_max_heading_level_value_is_parsed_for_outline() {
    let parsed =
        parse(["outline", "doc.md", "--max-heading-level", "2"]).expect("parse max heading level");

    match parsed.command {
        CliCommand::Document(command) => {
            assert_eq!(command.native_options.len(), 1);
            assert_eq!(command.native_options[0].flag, "--max-heading-level");
            assert_eq!(command.native_options[0].value, "2");
        }
        command => panic!("expected document command, got {command:?}"),
    }
}

#[test]
fn max_heading_level_is_rejected_for_read() {
    let error = parse([
        "read",
        "doc.md",
        "--ref",
        "doc:full",
        "--max-heading-level",
        "2",
    ])
    .expect_err("read should not accept max heading level");
    assert_diagnostic(error, "--max-heading-level", "unsupported_argument");
}

#[test]
fn invalid_pagination_value_returns_error() {
    let error = parse(["outline", "doc.md", "--pagination", "off"])
        .expect_err("invalid pagination value should be rejected");
    assert_diagnostic(error, "--pagination", "invalid_value");
}

#[test]
fn unused_known_argument_value_is_rejected_before_execution() {
    let error = parse([
        "info",
        "doc.md",
        "--page",
        "nope",
        "--output",
        "readable-json",
    ])
    .expect_err("unused page should fail info");

    assert_eq!(error.exit_code().code(), DocnavExitCode::InputError.code());
    assert_diagnostic(error, "--page", "unsupported_argument");
}

#[test]
fn unknown_document_argument_is_rejected() {
    let error = parse(["outline", "--future", "doc.md"]).expect_err("unknown argument should fail");

    assert_eq!(error.exit_code().code(), DocnavExitCode::InputError.code());
    assert_diagnostic(error, "argv", "unknown_argument");
}

#[test]
fn extra_document_positional_is_rejected() {
    let error = parse(["outline", "doc.md", "extra.md"]).expect_err("extra positional should fail");

    assert_eq!(error.exit_code().code(), DocnavExitCode::InputError.code());
    assert_diagnostic(error, "argv", "extra_positional");
}

// @case WB-CORE-ARGS-REPAIR-001
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

fn assert_diagnostic(error: AppError, field: &str, reason_fragment: &str) {
    let details = error.diagnostic().details().to_value();
    assert_eq!(
        details.get("field").and_then(serde_json::Value::as_str),
        Some(field)
    );
    assert!(details
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|reason| reason.contains(reason_fragment)));
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
