use crate::{
    Adapter, AdapterDefinition, AdapterDefinitionError, AdapterOptionProcessStrategy,
    AdapterOptionSpec, FieldValidation, UnstructuredFullReadCapabilities,
};
use docnav_protocol::Operation;

use super::support::{definition_builder, no_hook_option, NoHookAdapter};

#[test]
fn adapter_definition_requires_all_operation_handlers() {
    let adapter = NoHookAdapter;
    let error = AdapterDefinition::builder("no-hook")
        .adapter(&adapter)
        .manifest(adapter.manifest())
        .operation_handler(Operation::Outline)
        .operation_handler(Operation::Read)
        .operation_handler(Operation::Find)
        .build()
        .expect_err("missing info handler");

    assert!(matches!(
        error,
        AdapterDefinitionError::MissingRequiredHandlers { operations, .. }
            if operations == vec![Operation::Info]
    ));
}

#[test]
fn adapter_definition_rejects_invalid_and_duplicate_native_options() {
    let adapter = NoHookAdapter;
    let invalid = AdapterOptionSpec::builder("docnav.adapters.no-hook.options.bad")
        .owner("no-hook")
        .operations(&[Operation::Outline])
        .path(["invalid", "bad"])
        .validation(FieldValidation::int())
        .build();
    let error = definition_builder(&adapter)
        .native_option(invalid)
        .build()
        .expect_err("invalid option path");
    assert!(matches!(
        error,
        AdapterDefinitionError::InvalidNativeOption { .. }
    ));

    let shared = no_hook_option("docnav.adapters.no-hook.options.shared", "shared");
    let error = definition_builder(&adapter)
        .native_option(shared.clone())
        .native_option(shared)
        .build()
        .expect_err("duplicate native option identity");
    assert!(matches!(
        error,
        AdapterDefinitionError::DuplicateNativeOptionDeclaration { .. }
    ));

    let error = definition_builder(&adapter)
        .native_option(no_hook_option(
            "docnav.adapters.no-hook.options.first",
            "shared",
        ))
        .native_option(no_hook_option(
            "docnav.adapters.no-hook.options.second",
            "shared",
        ))
        .build()
        .expect_err("duplicate native option path");
    assert!(matches!(
        error,
        AdapterDefinitionError::DuplicateNativeOptionPath { .. }
    ));
}

#[test]
fn adapter_definition_rejects_incomplete_native_option_field_declarations() {
    let adapter = NoHookAdapter;
    let missing_processing =
        AdapterOptionSpec::builder("docnav.adapters.no-hook.options.missing_processing")
            .owner("no-hook")
            .operations(&[Operation::Outline])
            .path(["options", "missing_processing"])
            .validation(FieldValidation::int())
            .build();
    let missing_validation =
        AdapterOptionSpec::builder("docnav.adapters.no-hook.options.missing_validation")
            .owner("no-hook")
            .operations(&[Operation::Outline])
            .path(["options", "missing_validation"])
            .process(
                "config",
                AdapterOptionProcessStrategy::config_path([
                    "options",
                    "no-hook",
                    "missing_validation",
                ]),
            )
            .build();

    for (option, identity, reason) in [
        (
            missing_processing,
            "docnav.adapters.no-hook.options.missing_processing",
            "field processing strategy is missing",
        ),
        (
            missing_validation,
            "docnav.adapters.no-hook.options.missing_validation",
            "field validation is missing",
        ),
    ] {
        let error = definition_builder(&adapter)
            .native_option(option)
            .build()
            .expect_err("incomplete typed-field declaration");

        assert_eq!(
            error,
            AdapterDefinitionError::InvalidNativeOption {
                id: "no-hook".to_owned(),
                option: identity.to_owned(),
                reason: format!("adapter option {identity} field declaration is invalid: {reason}"),
            }
        );
    }
}

#[test]
fn adapter_definition_rejects_native_option_owner_mismatch() {
    let adapter = NoHookAdapter;
    let mismatched_owner = AdapterOptionSpec::builder("docnav.adapters.other.options.shared")
        .owner("other-adapter")
        .operations(&[Operation::Outline])
        .path(["options", "shared"])
        .process(
            "config",
            AdapterOptionProcessStrategy::config_path(["options", "other-adapter", "shared"]),
        )
        .validation(FieldValidation::int())
        .build();

    let error = definition_builder(&adapter)
        .native_option(mismatched_owner)
        .build()
        .expect_err("native option owner mismatch");

    assert!(matches!(
        error,
        AdapterDefinitionError::NativeOptionOwnerMismatch {
            id,
            owner,
            ..
        } if id == "no-hook" && owner == "other-adapter"
    ));
}

#[test]
fn adapter_definition_rejects_duplicate_handlers_and_capability_groups() {
    let adapter = NoHookAdapter;
    let error = definition_builder(&adapter)
        .operation_handler(Operation::Outline)
        .build()
        .expect_err("duplicate outline handler");
    assert!(matches!(
        error,
        AdapterDefinitionError::DuplicateOperationHandler {
            operation: Operation::Outline,
            ..
        }
    ));

    let capabilities = UnstructuredFullReadCapabilities {
        content_hook: true,
        cost_measurement_units: Vec::new(),
        result_facts_hook: false,
    };
    let error = definition_builder(&adapter)
        .full_read_capability_group(capabilities.clone())
        .full_read_capability_group(capabilities)
        .build()
        .expect_err("duplicate full-read group");
    assert!(matches!(
        error,
        AdapterDefinitionError::DuplicateCapabilityGroup {
            capability: "full_read",
            ..
        }
    ));
}

#[test]
fn adapter_definition_rejects_unsupported_full_read_capability_group() {
    let adapter = NoHookAdapter;
    let error = definition_builder(&adapter)
        .full_read_capability_group(UnstructuredFullReadCapabilities::default())
        .build()
        .expect_err("empty full-read group");

    assert!(matches!(
        error,
        AdapterDefinitionError::UnsupportedCapabilityCombination {
            capability: "full_read",
            ..
        }
    ));
}
