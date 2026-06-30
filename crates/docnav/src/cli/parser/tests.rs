use super::*;
use crate::cli::{CliCommand, OutputMode};
use crate::error::DocnavExitCode;

// @case WB-CORE-HELP-001
#[test]
fn help_returns_typed_help_command() {
    let parsed = parse(["outline", "--help"]).expect("parse help");

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
            assert_eq!(command.pagination_enabled, None);
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

    assert_eq!(error.exit_code().code(), DocnavExitCode::InputError.code());
    let details = error.diagnostic().details().to_value();
    let reason = details
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

    let details = error.diagnostic().details().to_value();
    let reason = details
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
    let details = error.diagnostic().details().to_value();

    assert_eq!(
        details.get("field").and_then(serde_json::Value::as_str),
        Some("--page")
    );
    assert!(details
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|reason| reason.contains("positive integer")));
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
fn invalid_pagination_value_returns_error() {
    let error = parse(["outline", "doc.md", "--pagination", "off"])
        .expect_err("invalid pagination value should be rejected");
    let details = error.diagnostic().details().to_value();

    assert_eq!(
        details.get("field").and_then(serde_json::Value::as_str),
        Some("--pagination")
    );
    assert!(details
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|reason| reason.contains("enabled or disabled")));
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
    let details = error.diagnostic().details().to_value();
    assert_eq!(
        details.get("field").and_then(serde_json::Value::as_str),
        Some("--page")
    );
    assert!(details
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|reason| reason.contains("not used by info command")));
}

#[test]
fn unknown_document_argument_is_rejected() {
    let error = parse(["outline", "--future", "doc.md"]).expect_err("unknown argument should fail");

    assert_eq!(error.exit_code().code(), DocnavExitCode::InputError.code());
    let details = error.diagnostic().details().to_value();
    assert_eq!(
        details.get("field").and_then(serde_json::Value::as_str),
        Some("argv")
    );
    assert!(details
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|reason| reason.contains("unknown argument --future")));
}

#[test]
fn extra_document_positional_is_rejected() {
    let error = parse(["outline", "doc.md", "extra.md"]).expect_err("extra positional should fail");

    assert_eq!(error.exit_code().code(), DocnavExitCode::InputError.code());
    let details = error.diagnostic().details().to_value();
    assert_eq!(
        details.get("field").and_then(serde_json::Value::as_str),
        Some("argv")
    );
    assert!(details
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|reason| reason.contains("extra positional argument extra.md")));
}
