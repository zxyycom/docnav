use super::super::warning::{CliWarningDetails, CliWarningEffect, CLI_ARGV_IGNORED};
use super::*;
use crate::cli::{CliCommand, OutputMode};
use crate::error::exit_code_for_error;

// @case WB-CORE-HELP-001
#[test]
fn help_returns_typed_help_command() {
    let parsed = parse(["outline", "--help"]).expect("parse help");

    assert!(parsed.warnings.is_empty());
    match parsed.command {
        CliCommand::Help(text) => {
            assert!(text.contains("Usage:"));
            assert!(text.contains("--output"));
            assert!(text.contains("outline"));
        }
        command => panic!("expected help command, got {command:?}"),
    }
}

// ── 2.8: Output mode help text shows only three accepted values ────────

#[test]
fn help_text_shows_three_output_modes() {
    let parsed = parse(["outline", "--help"]).expect("parse help");

    match parsed.command {
        CliCommand::Help(text) => {
            // Help must show only the three accepted document output modes.
            assert!(
                text.contains("readable-view"),
                "help should list readable-view; got:\n{text}"
            );
            assert!(
                text.contains("readable-json"),
                "help should list readable-json; got:\n{text}"
            );
            assert!(
                text.contains("protocol-json"),
                "help should list protocol-json; got:\n{text}"
            );
            // Legacy text output mode must not appear.
            assert!(
                !text.contains("text|readable-json|protocol-json"),
                "help should not show legacy 'text' output value"
            );
        }
        command => panic!("expected help command, got {command:?}"),
    }
}

#[test]
fn help_command_has_no_output_mode() {
    let parsed = parse(["--help"]).expect("parse --help");
    match parsed.command {
        CliCommand::Help(_) => {} // Help is not a document command
        command => panic!("expected help command, got {command:?}"),
    }
}

#[test]
fn version_command_has_no_output_mode() {
    let parsed = parse(["version"]).expect("parse version");
    match parsed.command {
        CliCommand::Version => {} // Version is not a document command
        command => panic!("expected version command, got {command:?}"),
    }
}

// ── 2.8: Default output mode is readable-view ─────────────────────────

// @case WB-CORE-OUTPUTMODE-001
#[test]
fn default_arg_output_is_readable_view() {
    // clap default_value for --output should be "readable-view".
    assert_eq!(spec::defaults::OUTPUT, "readable-view");
}

#[test]
fn parse_without_output_has_none() {
    // When no --output is given, the parsed command has output=None.
    // The default (ReadableView) is resolved later by DocumentRequest::from_command
    // via the config chain (CLI > project config > user config > built-in).
    let parsed = parse(["outline", "doc.md"]).expect("parse with default output");

    match parsed.command {
        CliCommand::Document(command) => {
            assert_eq!(command.output, None);
        }
        command => panic!("expected document command, got {command:?}"),
    }
}

// ── 2.8: Explicit output mode parsing ──────────────────────────────────

#[test]
fn parse_explicit_readable_view() {
    let parsed =
        parse(["outline", "doc.md", "--output", "readable-view"]).expect("parse readable-view");

    match parsed.command {
        CliCommand::Document(command) => {
            assert_eq!(command.output, Some(OutputMode::ReadableView));
        }
        command => panic!("expected document command, got {command:?}"),
    }
}

#[test]
fn parse_explicit_readable_json() {
    let parsed =
        parse(["outline", "doc.md", "--output", "readable-json"]).expect("parse readable-json");

    match parsed.command {
        CliCommand::Document(command) => {
            assert_eq!(command.output, Some(OutputMode::ReadableJson));
        }
        command => panic!("expected document command, got {command:?}"),
    }
}

#[test]
fn parse_explicit_protocol_json() {
    let parsed =
        parse(["outline", "doc.md", "--output", "protocol-json"]).expect("parse protocol-json");

    match parsed.command {
        CliCommand::Document(command) => {
            assert_eq!(command.output, Some(OutputMode::ProtocolJson));
        }
        command => panic!("expected document command, got {command:?}"),
    }
}

// ── 2.8: Invalid output value ──────────────────────────────────────────

#[test]
fn invalid_output_value_returns_error() {
    let error =
        parse(["outline", "doc.md", "--output", "text"]).expect_err("text should be rejected");

    assert_eq!(
        error.exit_code().code(),
        exit_code_for_error(docnav_protocol::StableErrorCode::InvalidRequest).code()
    );
    let reason = error
        .error()
        .details
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    // Error message in document.rs: "invalid --output: <FromStr error>"
    // FromStr error: "invalid output value \"text\", accepted values: ..."
    assert!(
        reason.contains("invalid --output"),
        "error should mention --output, got: {reason}"
    );
    assert!(
        reason.contains("readable-view"),
        "error should list accepted values, got: {reason}"
    );
    assert!(
        reason.contains("protocol-json"),
        "error should list protocol-json, got: {reason}"
    );
}

#[test]
fn bogus_output_value_returns_error() {
    let error = parse(["outline", "doc.md", "--output", "bogus"])
        .expect_err("bogus output should be rejected");

    let reason = error
        .error()
        .details
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    assert!(reason.contains("invalid --output"));
    assert!(reason.contains("bogus"));
}

// ── existing tests ─────────────────────────────────────────────────────

// @case WB-CORE-ARGS-001
#[test]
fn used_known_argument_stays_strict() {
    let error = parse(["outline", "doc.md", "--page", "0"]).expect_err("page is invalid");

    assert_eq!(
        error
            .error()
            .details
            .get("field")
            .and_then(serde_json::Value::as_str),
        Some("--page")
    );
    assert!(error
        .error()
        .details
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|reason| reason.contains("positive integer")));
}

#[test]
fn unused_known_argument_value_is_not_eagerly_typed() {
    let parsed = parse([
        "info",
        "doc.md",
        "--page",
        "nope",
        "--output",
        "readable-json",
    ])
    .expect("unused page should not fail info");

    match parsed.command {
        CliCommand::Document(command) => {
            assert_eq!(command.operation, Operation::Info);
            assert_eq!(command.output, Some(OutputMode::ReadableJson));
            assert!(command.page.is_none());
            assert!(command.limit_chars.is_none());
        }
        command => panic!("expected document command, got {command:?}"),
    }
    assert_eq!(parsed.warnings.len(), 1);
    let warning = &parsed.warnings[0];
    assert_eq!(warning.id, CLI_ARGV_IGNORED);
    assert_eq!(warning.effect, CliWarningEffect::OperationContinued);
    match &warning.details {
        CliWarningDetails::CliArgv { tokens } => {
            assert!(tokens.contains(&"--page".to_owned()));
            assert!(tokens.contains(&"nope".to_owned()));
        }
        details => panic!("expected CLI argv details, got {details:?}"),
    }
}
