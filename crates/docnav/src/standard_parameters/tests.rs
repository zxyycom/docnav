// @case WB-CORE-STDPARAMS-001
use std::path::PathBuf;

use docnav_protocol::Operation;
use serde_json::json;

use super::*;
use crate::cli::DocumentCommand;
use crate::config::CoreConfig;
use crate::project_context::ProjectContext;
use crate::registry::AdapterRegistry;

#[test]
fn core_document_parameters_use_standard_resolution_sources() {
    let command = DocumentCommand {
        operation: Operation::Outline,
        path: "doc.md".to_owned(),
        ref_id: None,
        query: None,
        page: None,
        pagination_enabled: None,
        limit: None,
        max_heading_level: None,
        output: None,
        adapter: Some("direct-adapter".to_owned()),
    };
    let context = ConfigContext {
        project: project_context(),
        project_config: serde_json::from_value(json!({
            "defaults": {
                "adapter": "project-adapter",
                "pagination": {
                    "enabled": true,
                    "limit": 321
                },
                "output": "protocol-json"
            }
        }))
        .unwrap(),
        user_config: serde_json::from_value(json!({
            "defaults": {
                "pagination": {
                    "enabled": false,
                    "limit": 111
                },
                "output": "readable-json"
            }
        }))
        .unwrap(),
    };

    let resolved = resolve_core_document_parameters(&command, &context).unwrap();

    assert_eq!(resolved.path, "doc.md");
    assert_eq!(resolved.adapter.as_deref(), Some("direct-adapter"));
    assert_eq!(resolved.limit.unwrap().get(), 321);
    assert_eq!(resolved.page.unwrap().get(), 1);
    assert_eq!(resolved.output, OutputMode::ProtocolJson);
    assert_eq!(resolved.defaults.adapter.source, "explicit");
    let pagination = resolved.defaults.pagination.unwrap();
    assert_eq!(pagination.enabled.source, "project");
    assert_eq!(pagination.limit.source, "project");
    assert_eq!(resolved.defaults.output.source, "project");
    assert_eq!(resolved.defaults.page.unwrap().source, "built_in");
}

#[test]
fn core_document_parameters_merge_registered_native_options_without_selected_adapter() {
    let command = DocumentCommand {
        operation: Operation::Outline,
        path: "doc.md".to_owned(),
        ref_id: None,
        query: None,
        page: None,
        pagination_enabled: None,
        limit: None,
        max_heading_level: Some(docnav_protocol::try_positive(4).unwrap()),
        output: None,
        adapter: None,
    };
    let context = ConfigContext {
        project: project_context(),
        project_config: serde_json::from_value(json!({
            "options": {
                "max_heading_level": "wide"
            }
        }))
        .unwrap(),
        user_config: CoreConfig::default(),
    };

    let resolved = resolve_core_document_parameters(&command, &context).unwrap();
    let registry = AdapterRegistry::load(&context.project).unwrap();
    let native_options = registry.native_options_for(Operation::Outline);
    let options = resolve_registered_native_options(&command, &context, &native_options).unwrap();

    assert!(resolved.options.is_none());
    assert_eq!(
        options
            .as_ref()
            .and_then(|options| options.get("max_heading_level"))
            .and_then(serde_json::Value::as_i64),
        Some(4)
    );
}

#[test]
fn registered_native_options_preserve_config_json_value_when_cli_is_absent() {
    let command = DocumentCommand {
        operation: Operation::Outline,
        path: "doc.md".to_owned(),
        ref_id: None,
        query: None,
        page: None,
        pagination_enabled: None,
        limit: None,
        max_heading_level: None,
        output: None,
        adapter: None,
    };
    let context = ConfigContext {
        project: project_context(),
        project_config: serde_json::from_value(json!({
            "options": {
                "max_heading_level": "wide"
            }
        }))
        .unwrap(),
        user_config: CoreConfig::default(),
    };
    let registry = AdapterRegistry::load(&context.project).unwrap();
    let native_options = registry.native_options_for(Operation::Outline);

    let options = resolve_registered_native_options(&command, &context, &native_options).unwrap();

    assert_eq!(
        options
            .as_ref()
            .and_then(|options| options.get("max_heading_level"))
            .and_then(serde_json::Value::as_str),
        Some("wide")
    );
}

#[test]
fn native_option_specs_with_same_key_are_not_folded_by_key() {
    let integer_variant = NativeOptionSpec {
        identity: "docnav.adapters.integer.options.shared",
        owner: "integer-adapter",
        namespace: "options",
        key: "shared",
        operations: &[Operation::Outline],
        value: docnav_adapter_contracts::NativeOptionValueSpec::Integer { min: 1, max: 3 },
    };
    let string_variant = NativeOptionSpec {
        identity: "docnav.adapters.string.options.shared",
        owner: "string-adapter",
        namespace: "options",
        key: "shared",
        operations: &[Operation::Outline],
        value: docnav_adapter_contracts::NativeOptionValueSpec::String,
    };

    let options =
        native_option_keys_for_operation(Operation::Outline, &[integer_variant, string_variant]);

    assert_eq!(options.len(), 2);
    assert_eq!(options[0].owner, "integer-adapter");
    assert_eq!(options[1].owner, "string-adapter");
    assert_ne!(options[0].value_kind(), options[1].value_kind());
}

#[test]
fn registered_native_option_handoff_preserves_same_key_variant_metadata() {
    let integer_variant = NativeOptionSpec {
        identity: "docnav.adapters.integer.options.shared",
        owner: "integer-adapter",
        namespace: "options",
        key: "shared",
        operations: &[Operation::Outline],
        value: docnav_adapter_contracts::NativeOptionValueSpec::Integer { min: 1, max: 3 },
    };
    let string_variant = NativeOptionSpec {
        identity: "docnav.adapters.string.options.shared",
        owner: "string-adapter",
        namespace: "options",
        key: "shared",
        operations: &[Operation::Outline],
        value: docnav_adapter_contracts::NativeOptionValueSpec::String,
    };
    let command = DocumentCommand {
        operation: Operation::Outline,
        path: "doc.md".to_owned(),
        ref_id: None,
        query: None,
        page: None,
        pagination_enabled: None,
        limit: None,
        max_heading_level: None,
        output: None,
        adapter: None,
    };
    let context = ConfigContext {
        project: project_context(),
        project_config: serde_json::from_value(json!({
            "options": {
                "shared": 2
            }
        }))
        .unwrap(),
        user_config: CoreConfig::default(),
    };

    let options =
        resolve_registered_native_options(&command, &context, &[integer_variant, string_variant])
            .unwrap()
            .expect("native options");

    assert_eq!(options.len(), 1);
    assert_eq!(options.entries().len(), 2);
    assert_eq!(options.entries()[0].owner, "integer-adapter");
    assert_eq!(options.entries()[0].source, "project_config");
    assert_eq!(options.entries()[0].type_variant, "integer");
    assert_eq!(options.entries()[1].owner, "string-adapter");
    assert_eq!(options.entries()[1].source, "project_config");
    assert_eq!(options.entries()[1].type_variant, "string");
}

#[test]
fn core_pagination_disabled_finalizes_operation_limit() {
    let command = DocumentCommand {
        operation: Operation::Outline,
        path: "doc.md".to_owned(),
        ref_id: None,
        query: None,
        page: None,
        pagination_enabled: Some(false),
        limit: Some(docnav_protocol::try_positive(12).unwrap()),
        max_heading_level: None,
        output: None,
        adapter: None,
    };
    let context = ConfigContext {
        project: project_context(),
        project_config: CoreConfig::default(),
        user_config: CoreConfig::default(),
    };

    let resolved = resolve_core_document_parameters(&command, &context).unwrap();

    assert_eq!(resolved.limit.unwrap().get(), u32::MAX);
    let pagination = resolved.defaults.pagination.unwrap();
    assert_eq!(pagination.enabled.source, "explicit");
    assert_eq!(pagination.limit.source, "explicit");
}

fn project_context() -> ProjectContext {
    let root = PathBuf::from("D:/docnav-test");
    ProjectContext {
        cwd: root.clone(),
        project_root: root.clone(),
        project_config_path: root.join(".docnav").join("docnav.json"),
        user_config_path: root.join("user").join("docnav.json"),
    }
}

fn _core_config_type_is_reexported_for_tests(_: CoreConfig) {}
