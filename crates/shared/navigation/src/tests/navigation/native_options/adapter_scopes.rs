use super::*;

#[test]
fn navigation_rejects_option_missing_from_core_catalog() {
    let error = execute_loaded_navigation_command(
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
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect_err("the adapter cannot add product parameters outside the core catalog");
    let protocol_error = super::super::protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error
            .details()
            .get("field")
            .and_then(Value::as_str),
        Some("arguments.options.payload")
    );
    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some("unsupported")
    );
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
        &crate::tests::support::document_parameter_catalog(),
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
                    "docnav-other": {
                        "max_heading_level": 6
                    }
                }
            }),
            Value::Null,
        ),
        &crate::tests::support::document_parameter_catalog(),
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
    assert_eq!(result.entries[0].label, "Max 3");
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
        &crate::tests::support::document_parameter_catalog(),
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
