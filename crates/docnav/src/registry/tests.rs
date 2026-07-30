use super::*;

#[test]
fn static_registry_contains_built_in_markdown_adapter() {
    let registry = AdapterRegistry::builtin();
    let definitions = registry
        .adapters
        .iter()
        .map(|definition| definition())
        .collect::<Vec<_>>();

    assert_eq!(
        definitions
            .iter()
            .map(AdapterDefinition::id)
            .collect::<Vec<_>>(),
        ["docnav-markdown", "docnav-json"]
    );
    assert_eq!(definitions[0].manifest().formats[0].id, "markdown");
    let json_format = &definitions[1].manifest().formats[0];
    assert_eq!(json_format.id, "json");
    assert_eq!(json_format.extensions, [".json"]);
    assert_eq!(json_format.content_types, ["application/json"]);

    for definition in definitions {
        let probe = definition.probe("registry-metadata-probe");
        assert_eq!(probe.adapter_id, definition.id());
        assert_eq!(probe.path, "registry-metadata-probe");
        assert!(!probe.reasons.is_empty());
    }
}

#[test]
fn adapter_layer_check_reports_definition_metadata_and_core_source() {
    let registry = AdapterRegistry::builtin();
    let checks = adapter_layer_checks(&registry);
    let registry_check = registry_check(&registry);

    assert_eq!(registry_check.value()["status"], "pass");
    assert_eq!(registry_check.value()["adapter_count"], 2);
    assert_eq!(checks.len(), 2);
    assert_eq!(
        checks
            .iter()
            .map(|check| check.value()["adapter_id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["docnav-markdown", "docnav-json"]
    );
    assert_eq!(
        checks
            .iter()
            .map(|check| check.value()["formats"][0]["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["markdown", "json"]
    );
    for check in checks {
        let check = check.value();
        assert_eq!(check["status"], "pass");
        assert_eq!(
            check["message"],
            "built-in adapter layer metadata is available"
        );
        assert_eq!(check["implementation_source"], "core_static");
        assert_eq!(check["version"], env!("CARGO_PKG_VERSION"));
    }
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

    assert_eq!(exit_code, 0);
    assert!(stderr.is_empty());
    assert_eq!(
        output.get("registry").and_then(Value::as_str),
        Some("core_static")
    );
    assert_eq!(
        output["adapters"],
        json!([
            {
                "id": "docnav-markdown",
                "name": "Docnav Markdown Adapter",
                "version": env!("CARGO_PKG_VERSION"),
                "implementation_source": "core_static",
                "formats": [{
                    "id": "markdown",
                    "extensions": [".md", ".markdown"],
                    "content_types": ["text/markdown"],
                }],
            },
            {
                "id": "docnav-json",
                "name": "Docnav JSON Adapter",
                "version": env!("CARGO_PKG_VERSION"),
                "implementation_source": "core_static",
                "formats": [{
                    "id": "json",
                    "extensions": [".json"],
                    "content_types": ["application/json"],
                }],
            },
        ])
    );
}
