use super::super::parse;
use crate::cli::{CliCommand, OutputMode};
use crate::error::DocnavExitCode;

#[test]
fn parse_without_output_has_none() {
    let parsed = parse(["outline", "doc.md"]).expect("parse with default output");

    match parsed.command {
        CliCommand::Document(command) => {
            assert_eq!(command.output, None);
            assert_eq!(command.pagination_enabled, None);
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
    assert_eq!(reason, "invalid_value");
    assert_eq!(details["received"], "text");
    assert!(
        details["accepted"]
            .as_array()
            .is_some_and(|accepted| accepted
                .iter()
                .any(|value| value.as_str() == Some("readable-view"))),
        "error should list readable-view, got: {details}"
    );
    assert!(
        details["accepted"]
            .as_array()
            .is_some_and(|accepted| accepted
                .iter()
                .any(|value| value.as_str() == Some("protocol-json"))),
        "error should list protocol-json, got: {details}"
    );
}
