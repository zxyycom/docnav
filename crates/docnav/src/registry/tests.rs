use super::*;

// @case WB-CORE-ADAPTER-001
#[test]
fn static_registry_contains_built_in_markdown_adapter() {
    let registry = AdapterRegistry { adapters: ADAPTERS };
    let record = registry
        .adapters
        .iter()
        .find(|adapter| adapter.id() == "docnav-markdown")
        .expect("built-in markdown adapter");

    let manifest = record.manifest();

    assert_eq!(record.id(), "docnav-markdown");
    assert_eq!(manifest.adapter.id, "docnav-markdown");
    assert_eq!(record.implementation_source(), "core_static");
    assert!(manifest
        .formats
        .iter()
        .any(|format| format.id == "markdown"));
}

#[test]
fn adapter_layer_check_reports_definition_metadata_and_core_source() {
    let registry = AdapterRegistry { adapters: ADAPTERS };
    let checks = adapter_layer_checks(&registry);
    let check = checks.first().expect("adapter layer check");

    assert_eq!(check.get("status").and_then(Value::as_str), Some("pass"));
    assert_eq!(
        check.get("message").and_then(Value::as_str),
        Some("built-in adapter layer metadata is available")
    );
    assert_eq!(
        check.get("implementation_source").and_then(Value::as_str),
        Some("core_static")
    );
}

#[test]
fn static_registry_exposes_full_native_option_specs() {
    let registry = AdapterRegistry { adapters: ADAPTERS };
    let native_options = registry.native_options_for(Operation::Outline);

    assert!(native_options.iter().any(|option| {
        option.owner == "docnav-markdown"
            && option.namespace() == "options"
            && option.key() == "max_heading_level"
    }));
}
