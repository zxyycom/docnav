use docnav_adapter_contracts::{
    AdapterDefinition, AdapterOptionProcessStrategy, AdapterOptionSpec, FieldValidation,
};
use docnav_protocol::{positive_result, Operation, OperationResult, ProtocolResponse};
use serde_json::{json, Value};

use crate::{
    execute_loaded_navigation_command, NavigationAdapterRef, NavigationAdapterRegistry,
    NavigationNativeOptionInput, NavigationOutputMode,
};

use super::super::support::{config_sources, navigation_command, StubRegistry};

struct TypedOnlyConstraintRegistry;

impl NavigationAdapterRegistry for TypedOnlyConstraintRegistry {
    fn adapters(&self) -> Vec<NavigationAdapterRef<'_>> {
        let base = StubRegistry
            .adapters()
            .into_iter()
            .next()
            .expect("stub adapter definition");
        let definition = AdapterDefinition::builder(base.id())
            .adapter(base.definition.adapter())
            .manifest(base.definition.manifest().clone())
            .required_operation_handlers()
            .native_options(typed_only_constraint_options())
            .build()
            .expect("typed-only constraint adapter definition");
        vec![NavigationAdapterRef::new(definition)]
    }
}

fn typed_only_constraint_options() -> Vec<AdapterOptionSpec> {
    vec![
        AdapterOptionSpec::builder("docnav.adapters.docnav-markdown.options.pattern")
            .owner("docnav-markdown")
            .operations(&[Operation::Outline])
            .path(["options", "pattern"])
            .process(
                "config",
                AdapterOptionProcessStrategy::json_path(["options", "docnav-markdown", "pattern"]),
            )
            .validation(FieldValidation::string().regex("^valid$"))
            .build(),
        AdapterOptionSpec::builder("docnav.adapters.docnav-markdown.options.tags")
            .owner("docnav-markdown")
            .operations(&[Operation::Outline])
            .path(["options", "tags"])
            .process(
                "config",
                AdapterOptionProcessStrategy::json_path(["options", "docnav-markdown", "tags"]),
            )
            .validation(FieldValidation::array().unique_items())
            .build(),
    ]
}

// @case WB-NAVIGATION-HARD-CUTOVER-001
#[test]
fn hard_cutover_preserves_common_and_native_option_source_priority() {
    let mut command = navigation_command(vec![NavigationNativeOptionInput {
        flag: "--max-heading-level".to_owned(),
        value: "4".to_owned(),
    }]);
    command.limit = Some(positive_result(42).unwrap());
    command.output = Some(NavigationOutputMode::ProtocolJson);

    let outcome = execute_loaded_navigation_command(
        command,
        config_sources(
            json!({
                "defaults": {
                    "pagination": { "limit": 120 },
                    "output": "readable-json"
                },
                "options": {
                    "docnav-markdown": { "max_heading_level": 2 }
                }
            }),
            json!({
                "defaults": {
                    "pagination": { "limit": 240 },
                    "output": "readable-view"
                },
                "options": {
                    "docnav-markdown": { "max_heading_level": 1 }
                }
            }),
        ),
        &StubRegistry,
    )
    .expect("hard cutover priority outcome");

    assert_eq!(outcome.output, NavigationOutputMode::ProtocolJson);
    let ProtocolResponse::Success(success) = outcome.response else {
        panic!("expected success");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };
    let result = result.as_structured().expect("structured outline result");
    assert_eq!(result.entries[0].label, "Max 4");
}

#[test]
fn hard_cutover_preserves_field_declaration_order_for_primary_diagnostic() {
    let error = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "defaults": {
                    "output": "invalid-output"
                },
                "options": {
                    "docnav-markdown": {
                        "max_heading_level": 9
                    }
                }
            }),
            Value::Null,
        ),
        &StubRegistry,
    )
    .expect_err("mixed invalid common and native option fields");
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error
            .details()
            .get("field")
            .and_then(Value::as_str),
        Some("defaults.output")
    );
}

#[test]
fn hard_cutover_preserves_declaration_order_for_typed_only_constraints() {
    for (constraint, invalid_options) in [
        ("regex", json!({ "pattern": "invalid" })),
        ("unique_items", json!({ "tags": ["same", "same"] })),
    ] {
        let error = execute_loaded_navigation_command(
            navigation_command(Vec::new()),
            config_sources(
                json!({
                    "defaults": {
                        "output": "invalid-output"
                    },
                    "options": {
                        "docnav-markdown": invalid_options
                    }
                }),
                Value::Null,
            ),
            &TypedOnlyConstraintRegistry,
        )
        .expect_err("mixed invalid common and typed-only native option fields");
        let protocol_error = super::protocol_error(error.diagnostic());

        assert_eq!(
            protocol_error
                .details()
                .get("field")
                .and_then(Value::as_str),
            Some("defaults.output"),
            "typed-only {constraint} diagnostic must follow earlier common fields"
        );
    }
}
