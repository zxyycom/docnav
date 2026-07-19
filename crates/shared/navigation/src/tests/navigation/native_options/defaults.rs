use super::*;

#[test]
fn navigation_includes_adapter_native_option_default() {
    let outcome = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(Value::Null, Value::Null),
        &crate::tests::support::document_parameter_catalog(),
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
fn navigation_accepts_max_heading_level_range_boundaries() {
    for level in [1, 6] {
        let outcome = execute_loaded_navigation_command(
            navigation_command(vec![cli_value_candidate(
                "docnav.adapters.docnav-markdown.options.max_heading_level",
                "--max-heading-level",
                json!(level),
            )]),
            config_sources(Value::Null, Value::Null),
            &crate::tests::support::document_parameter_catalog(),
            &StubRegistry,
        )
        .expect("range boundary should resolve");

        let ProtocolResponse::Success(success) = outcome.response else {
            panic!("expected success");
        };
        let OperationResult::Outline(result) = success.result else {
            panic!("expected outline result");
        };
        let result = result.as_structured().expect("structured outline result");
        assert_eq!(result.entries[0].label, format!("Max {level}"));
    }
}

#[test]
fn optional_non_json_config_null_suppresses_default_projections() {
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
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect("config null suppresses the default projections");

    let ProtocolResponse::Success(success) = outcome.response else {
        panic!("expected success");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };
    let result = result.as_structured().expect("structured outline result");
    assert_eq!(result.entries[0].label, "Stub");
}
