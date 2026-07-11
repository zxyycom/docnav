use crate::{AdapterError, NativeOptionHandoff, NativeOptionIssue};
use docnav_protocol::ProtocolDiagnosticCode;

#[test]
fn adapter_error_constructors_project_protocol_error_details() {
    let not_found = AdapterError::document_not_found("missing.md").protocol_error();

    assert_eq!(not_found.code(), ProtocolDiagnosticCode::DocumentNotFound);
    assert_eq!(not_found.owner(), "adapter");
    assert_eq!(
        not_found.location(),
        Some(&serde_json::json!({ "path": "missing.md" }))
    );
    assert_eq!(
        not_found.guidance().unwrap()[0],
        "Check the document path and retry."
    );

    let issue = NativeOptionIssue {
        owner: "docnav-markdown".to_owned(),
        namespace: "options".to_owned(),
        key: "max_heading_level".to_owned(),
        source: "cli".to_owned(),
        reason_code: "above_maximum".to_owned(),
        field: "--max-heading-level".to_owned(),
        received: Some("7".to_owned()),
        expected: Some("integer in range 1..6".to_owned()),
        type_variant: Some("integer".to_owned()),
        config_source: None,
    };
    let invalid = AdapterError::native_option_invalid(
        "invalid max heading level",
        issue,
        ["Use --max-heading-level between 1 and 6.".to_owned()],
    )
    .protocol_error();

    assert_eq!(invalid.code(), ProtocolDiagnosticCode::InvalidRequest);
    assert_eq!(invalid.owner(), "adapter_options");
    assert_eq!(invalid.received(), Some(&serde_json::json!("7")));
    assert_eq!(
        invalid.expected(),
        Some(&serde_json::json!("integer in range 1..6"))
    );
    assert_eq!(
        invalid
            .details()
            .get("reason")
            .and_then(serde_json::Value::as_str),
        Some("above_maximum")
    );
    let option_issue = invalid
        .details()
        .get("option_issues")
        .and_then(serde_json::Value::as_array)
        .and_then(|issues| issues.first())
        .expect("option issue is projected");
    assert_eq!(option_issue["owner"], "docnav-markdown");
    assert_eq!(
        invalid.guidance().unwrap()[0],
        "Use --max-heading-level between 1 and 6."
    );
}

#[test]
fn native_option_handoff_preserves_handler_facing_typed_metadata() {
    let mut options = docnav_protocol::Options::new();
    options.insert_entry(docnav_protocol::OptionEntry {
        identity: "docnav.adapters.no-hook.options.max".to_owned(),
        owner: "no-hook".to_owned(),
        namespace: "options".to_owned(),
        key: "max".to_owned(),
        source: "project".to_owned(),
        type_variant: "integer".to_owned(),
        value: serde_json::json!(4),
    });

    let handoff = NativeOptionHandoff::from_options(Some(&options));
    let value = handoff
        .get("no-hook", "options", "max")
        .expect("typed option value");

    assert_eq!(value.identity, "docnav.adapters.no-hook.options.max");
    assert_eq!(value.owner, "no-hook");
    assert_eq!(value.namespace, "options");
    assert_eq!(value.key, "max");
    assert_eq!(value.source, "project");
    assert_eq!(value.type_variant, "integer");
    assert_eq!(value.value, serde_json::json!(4));
}
