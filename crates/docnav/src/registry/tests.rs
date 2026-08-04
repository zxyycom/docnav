use super::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use docnav_adapter_contracts::{
    Adapter, AdapterError, AdapterResult, FindInput, InfoInput, OutlineInput, ReadInput,
    StandardOperationInput,
};
use docnav_navigation::{select_adapter, AdapterSelectionRequest};
use docnav_protocol::{
    positive_result, FindResult, InfoResult, Manifest, OutlineResult, ProtocolDiagnosticCode,
    ProtocolError, ReadResult,
};

#[test]
fn static_registry_contains_built_in_routing_metadata() {
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
    assert_eq!(
        serde_json::to_value(&definitions[0].manifest().formats).unwrap(),
        json!([{
            "id": "markdown",
            "extensions": [".md", ".markdown"],
            "filenames": [],
            "content_types": ["text/markdown"]
        }])
    );
    assert_eq!(
        serde_json::to_value(&definitions[1].manifest().formats).unwrap(),
        json!([{
            "id": "json",
            "extensions": [".json", ".code-workspace", ".jsonc"],
            "filenames": [".prettierrc", ".watchmanconfig"],
            "content_types": ["application/json", "application/jsonc"]
        }])
    );
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
                    "filenames": [],
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
                    "extensions": [".json", ".code-workspace", ".jsonc"],
                    "filenames": [".prettierrc", ".watchmanconfig"],
                    "content_types": ["application/json", "application/jsonc"],
                }],
            },
        ])
    );
}

#[test]
fn registry_check_rejects_duplicate_format_identity() {
    assert_registry_rejected(
        DUPLICATE_FORMAT_ADAPTERS,
        "registry-format-identity-conflict",
    );
    assert_explicit_registry_rejected(
        &AdapterRegistry::new(DUPLICATE_FORMAT_ADAPTERS),
        "docnav-json",
        "registry-format-identity-conflict",
    );
}

#[test]
fn registry_check_rejects_ascii_normalized_duplicate_suffix() {
    assert_registry_rejected(DUPLICATE_SUFFIX_ADAPTERS, "registry-path-hint-conflict");
}

#[test]
fn registry_check_rejects_duplicate_exact_filename() {
    assert_registry_rejected(DUPLICATE_FILENAME_ADAPTERS, "registry-path-hint-conflict");
}

#[test]
fn explicit_json_selection_bypasses_markdown_pathname_and_still_parses_document() {
    let mut path = std::env::temp_dir();
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("docnav-explicit-json-{suffix}.md"));
    fs::write(&path, "# not JSON\n").unwrap();

    let registry = AdapterRegistry::builtin();
    let path_string = path.display().to_string();
    let selection = select_adapter(AdapterSelectionRequest {
        registry: &registry,
        document_path: &path_string,
        preselected_adapter_id: Some("docnav-json"),
        preselected_adapter_source: "explicit",
    });
    let selected_id = selection
        .as_ref()
        .ok()
        .map(|selection| selection.adapter.id().to_owned());
    let input = StandardOperationInput::Outline(OutlineInput {
        document_path: path_string.clone(),
        page: positive_result(1).unwrap(),
        limit: positive_result(80).unwrap(),
        max_heading_level: None,
    });
    let operation = selection
        .as_ref()
        .ok()
        .map(|selection| selection.adapter.execute_operation(&input));

    fs::remove_file(path).unwrap();

    assert_eq!(selected_id.as_deref(), Some("docnav-json"));
    assert!(
        matches!(operation, Some(Err(_))),
        "lookup success must still execute the selected JSON parser"
    );

    let invalid_registry = AdapterRegistry::new(INVALID_HINT_ADAPTERS);
    assert_registry_rejected(INVALID_HINT_ADAPTERS, "registry-manifest-invalid");
    assert_explicit_registry_rejected(
        &invalid_registry,
        "docnav-invalid-hint",
        "registry-manifest-invalid",
    );
}

fn assert_registry_rejected(
    adapters: &'static [fn() -> AdapterDefinition<'static>],
    expected_error_id: &str,
) {
    let check = registry_check(&AdapterRegistry::new(adapters));
    assert_eq!(check.value()["status"], "fail");
    assert_eq!(check.value()["error_id"], expected_error_id);
    assert!(check.failure_exit_code().is_some());
}

fn assert_explicit_registry_rejected(
    registry: &AdapterRegistry,
    adapter_id: &str,
    expected_error_id: &str,
) {
    let error = select_adapter(AdapterSelectionRequest {
        registry,
        document_path: "ignored.unknown",
        preselected_adapter_id: Some(adapter_id),
        preselected_adapter_source: "explicit",
    })
    .expect_err("explicit lookup must not bypass registry invariants");
    let record = error
        .into_diagnostic()
        .into_record()
        .expect("registry invariant diagnostic must be valid");
    let protocol_error =
        ProtocolError::from_diagnostic_record(&record).expect("registry invariant projects");

    assert_eq!(protocol_error.code(), ProtocolDiagnosticCode::InternalError);
    assert_eq!(
        protocol_error.details().get("error_id"),
        Some(&json!(expected_error_id))
    );
}

static DUPLICATE_FORMAT_ADAPTERS: &[fn() -> AdapterDefinition<'static>] =
    &[json_adapter_definition, duplicate_json_format_definition];
static DUPLICATE_SUFFIX_ADAPTERS: &[fn() -> AdapterDefinition<'static>] =
    &[json_adapter_definition, duplicate_json_suffix_definition];
static DUPLICATE_FILENAME_ADAPTERS: &[fn() -> AdapterDefinition<'static>] = &[
    duplicate_filename_first_definition,
    duplicate_filename_second_definition,
];
static INVALID_HINT_ADAPTERS: &[fn() -> AdapterDefinition<'static>] = &[invalid_hint_definition];

fn duplicate_json_format_definition() -> AdapterDefinition<'static> {
    registry_test_definition("docnav-json-duplicate", "json", &[".duplicate-json"], None)
}

fn duplicate_json_suffix_definition() -> AdapterDefinition<'static> {
    registry_test_definition(
        "docnav-json-suffix-alternative",
        "json-alternative",
        &[".JSON"],
        None,
    )
}

fn duplicate_filename_first_definition() -> AdapterDefinition<'static> {
    registry_test_definition(
        "docnav-exact-first",
        "exact-first",
        &[".exact-first"],
        Some(&[".prettierrc"]),
    )
}

fn duplicate_filename_second_definition() -> AdapterDefinition<'static> {
    registry_test_definition(
        "docnav-exact-second",
        "exact-second",
        &[".exact-second"],
        Some(&[".prettierrc"]),
    )
}

fn invalid_hint_definition() -> AdapterDefinition<'static> {
    registry_test_definition("docnav-invalid-hint", "invalid-hint", &["."], None)
}

fn registry_test_definition(
    adapter_id: &str,
    format_id: &str,
    extensions: &[&str],
    filenames: Option<&[&str]>,
) -> AdapterDefinition<'static> {
    let format = json!({
        "id": format_id,
        "extensions": extensions,
        "filenames": filenames.unwrap_or_default(),
        "content_types": [format!("application/x-{format_id}")]
    });
    let manifest: Manifest = serde_json::from_value(json!({
        "manifest_version": docnav_protocol::MANIFEST_VERSION,
        "adapter": {
            "id": adapter_id,
            "name": adapter_id,
            "version": "0.1.0"
        },
        "formats": [format]
    }))
    .expect("registry test manifest must decode");
    AdapterDefinition::new(manifest, &REGISTRY_TEST_ADAPTER, None)
        .expect("registry test definition")
}

static REGISTRY_TEST_ADAPTER: RegistryTestAdapter = RegistryTestAdapter;

struct RegistryTestAdapter;

impl Adapter for RegistryTestAdapter {
    fn outline(&self, _input: &OutlineInput) -> AdapterResult<OutlineResult> {
        Err(AdapterError::internal("registry-test-outline-unreachable"))
    }

    fn read(&self, _input: &ReadInput) -> AdapterResult<ReadResult> {
        Err(AdapterError::internal("registry-test-read-unreachable"))
    }

    fn find(&self, _input: &FindInput) -> AdapterResult<FindResult> {
        Err(AdapterError::internal("registry-test-find-unreachable"))
    }

    fn info(&self, _input: &InfoInput) -> AdapterResult<InfoResult> {
        Err(AdapterError::internal("registry-test-info-unreachable"))
    }
}
