use super::*;

// @case WB-CORE-ADAPTER-001
#[test]
fn static_registry_contains_built_in_markdown_adapter() {
    let registry = AdapterRegistry::builtin();
    let definition = registry
        .adapters
        .iter()
        .map(|definition| definition())
        .find(|definition| definition.id() == "docnav-markdown")
        .expect("built-in markdown adapter");

    let manifest = definition.manifest();

    assert_eq!(definition.id(), "docnav-markdown");
    assert_eq!(manifest.adapter.id, "docnav-markdown");
    assert!(manifest
        .formats
        .iter()
        .any(|format| format.id == "markdown"));
}

#[test]
fn adapter_layer_check_reports_definition_metadata_and_core_source() {
    let registry = AdapterRegistry::builtin();
    let checks = adapter_layer_checks(&registry);
    let check = checks.first().expect("adapter layer check").value();

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
fn adapter_list_preserves_static_registry_projection() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit_code = crate::output::write_outcome(
        adapter_list().expect("adapter list"),
        &mut stdout,
        &mut stderr,
    );
    let output: Value = serde_json::from_slice(&stdout).expect("adapter list json");
    let adapters = output
        .get("adapters")
        .and_then(Value::as_array)
        .expect("adapters");

    assert_eq!(exit_code, 0);
    assert!(stderr.is_empty());
    assert_eq!(
        output.get("registry").and_then(Value::as_str),
        Some("core_static")
    );
    assert_eq!(adapters.len(), 1);
    assert_eq!(
        adapters[0].get("id").and_then(Value::as_str),
        Some("docnav-markdown")
    );
    assert_eq!(
        adapters[0]
            .get("implementation_source")
            .and_then(Value::as_str),
        Some("core_static")
    );
}
