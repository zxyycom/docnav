use super::super::parse;
use crate::cli::{AdapterCommand, CliCommand};

#[test]
fn adapter_list_returns_static_registry_command() {
    let parsed = parse(["adapter", "list"]).expect("parse adapter list");

    match parsed.command {
        CliCommand::Adapter(AdapterCommand::List) => {}
        command => panic!("expected adapter list command, got {command:?}"),
    }
}

#[test]
fn dynamic_adapter_management_is_unsupported() {
    let error = parse(["adapter", "install", "source"])
        .expect_err("dynamic adapter management should be rejected");
    assert_unsupported_adapter_command(error);
}

fn assert_unsupported_adapter_command(error: crate::error::AppError) {
    let details = error.diagnostic().details().to_value();
    assert_eq!(
        details.get("field").and_then(serde_json::Value::as_str),
        Some("adapter")
    );
    assert!(details
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|reason| reason.contains("unsupported adapter command")));
}
