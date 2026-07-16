mod validation;

use docnav_protocol::{OperationResult, ProtocolResponse};
use serde_json::{json, Value};

use crate::{execute_loaded_navigation_command, NavigationOutputMode};

use super::super::support::{
    cli_value_candidate, config_sources, navigation_command, MultiAdapterRegistry, StubRegistry,
};

#[test]
// @case WB-NAV-INPUT-RESOLUTION-001
fn navigation_resolves_selected_adapter_options_and_dispatches() {
    let outcome = execute_loaded_navigation_command(
        navigation_command(vec![cli_value_candidate(
            "docnav.adapters.docnav-markdown.options.max_heading_level",
            "--max-heading-level",
            json!(4),
        )]),
        config_sources(
            json!({
                "defaults": {
                    "pagination": { "limit": 120 },
                    "output": "protocol-json"
                },
                "options": {
                    "docnav-markdown": {
                        "max_heading_level": 2
                    }
                }
            }),
            json!({
                "options": {
                    "docnav-markdown": {
                        "max_heading_level": 1
                    }
                }
            }),
        ),
        &StubRegistry,
    )
    .expect("navigation outcome");

    assert_eq!(outcome.output, NavigationOutputMode::ProtocolJson);
    match outcome.response {
        ProtocolResponse::Success(success) => {
            assert_eq!(success.operation, docnav_protocol::Operation::Outline);
            assert!(success.ok);
            let OperationResult::Outline(result) = success.result else {
                panic!("expected outline result");
            };
            let result = result.as_structured().expect("structured outline result");
            assert_eq!(result.entries[0].label, "Max 4");
        }
        ProtocolResponse::Failure(failure) => panic!("expected success, got {failure:?}"),
    }
}

#[test]
fn navigation_includes_adapter_native_option_default() {
    let outcome = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(Value::Null, Value::Null),
        &StubRegistry,
    )
    .expect("navigation outcome");

    let ProtocolResponse::Success(success) = outcome.response else {
        panic!("expected success");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };
    let result = result.as_structured().expect("structured outline result");
    assert_eq!(result.entries[0].label, "Max 3");
}

#[test]
fn optional_non_json_config_null_suppresses_default_and_handoff() {
    let outcome = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "options": {
                    "docnav-markdown": {
                        "max_heading_level": null
                    }
                }
            }),
            Value::Null,
        ),
        &StubRegistry,
    )
    .expect("config null suppresses the default without entering handoff");

    let ProtocolResponse::Success(success) = outcome.response else {
        panic!("expected success");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };
    let result = result.as_structured().expect("structured outline result");
    assert_eq!(result.entries[0].label, "Stub");
}

#[test]
fn navigation_resolves_json_native_option_through_typed_fields() {
    let outcome = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "options": {
                    "docnav-markdown": {
                        "payload": {"source": "project"}
                    }
                }
            }),
            Value::Null,
        ),
        &StubRegistry,
    )
    .expect("navigation outcome");

    let ProtocolResponse::Success(success) = outcome.response else {
        panic!("expected success");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };
    let result = result.as_structured().expect("structured outline result");

    assert_eq!(result.entries[0].label, "Payload {\"source\":\"project\"}");
}

#[test]
fn navigation_accepts_config_option_applicable_to_operation() {
    let outcome = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "options": {
                    "docnav-markdown": {
                        "max_heading_level": 2
                    }
                }
            }),
            Value::Null,
        ),
        &StubRegistry,
    )
    .expect("applicable native option");

    let ProtocolResponse::Success(success) = outcome.response else {
        panic!("expected success");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };
    let result = result.as_structured().expect("structured outline result");
    assert_eq!(result.entries[0].label, "Max 2");
}

#[test]
fn navigation_does_not_forward_other_known_adapter_namespace() {
    let outcome = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "options": {
                    "docnav-markdown": {
                        "max_heading_level": 2
                    },
                    "docnav-other": {
                        "payload": {"source": "other"}
                    }
                }
            }),
            Value::Null,
        ),
        &MultiAdapterRegistry,
    )
    .expect("other adapter namespace remains non-selected source facts");

    let ProtocolResponse::Success(success) = outcome.response else {
        panic!("expected success");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };
    let result = result.as_structured().expect("structured outline result");
    assert_eq!(result.entries[0].label, "Max 2");
}

#[test]
fn navigation_keeps_same_option_key_distinct_by_adapter_namespace() {
    let outcome = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "options": {
                    "docnav-markdown": {
                        "max_heading_level": 2
                    },
                    "docnav-other": {
                        "max_heading_level": 6
                    }
                }
            }),
            Value::Null,
        ),
        &MultiAdapterRegistry,
    )
    .expect("same option key in other adapter namespace should coexist");

    let ProtocolResponse::Success(success) = outcome.response else {
        panic!("expected success");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };
    let result = result.as_structured().expect("structured outline result");
    assert_eq!(result.entries[0].label, "Max 2");
}
