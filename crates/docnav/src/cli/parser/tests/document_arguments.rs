use super::super::parse;
use crate::cli::CliCommand;
use crate::error::{AppError, DocnavExitCode};

#[test]
fn used_known_argument_stays_strict() {
    let error = parse(["outline", "doc.md", "--page", "0"]).expect_err("page is invalid");
    assert_diagnostic(error, "--page", "positive integer");
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
            assert_eq!(command.max_heading_level.map(|value| value.get()), Some(2));
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
    assert_diagnostic(error, "--max-heading-level", "not used by read command");
}

#[test]
fn invalid_pagination_value_returns_error() {
    let error = parse(["outline", "doc.md", "--pagination", "off"])
        .expect_err("invalid pagination value should be rejected");
    assert_diagnostic(error, "--pagination", "enabled or disabled");
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
    assert_diagnostic(error, "--page", "not used by info command");
}

#[test]
fn unknown_document_argument_is_rejected() {
    let error = parse(["outline", "--future", "doc.md"]).expect_err("unknown argument should fail");

    assert_eq!(error.exit_code().code(), DocnavExitCode::InputError.code());
    assert_diagnostic(error, "argv", "unknown argument --future");
}

#[test]
fn extra_document_positional_is_rejected() {
    let error = parse(["outline", "doc.md", "extra.md"]).expect_err("extra positional should fail");

    assert_eq!(error.exit_code().code(), DocnavExitCode::InputError.code());
    assert_diagnostic(error, "argv", "extra positional argument extra.md");
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
