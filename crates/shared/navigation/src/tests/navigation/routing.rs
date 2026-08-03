use std::sync::atomic::{AtomicUsize, Ordering};

use docnav_adapter_contracts::{
    Adapter, AdapterDefinition, AdapterError, AdapterResult, FindInput, InfoInput, OutlineInput,
    ReadInput,
};
use docnav_protocol::{
    AutoReadResult, Cost, FindResult, InfoResult, Manifest, OutlineResult, ProtocolDiagnosticCode,
    ProtocolResponse, ReadResult, StructuredOutlineResult,
};
use serde_json::{json, Value};

use crate::{
    execute_loaded_navigation_command, select_adapter, select_navigation_context,
    AdapterSelectionRequest, NavigationAdapterRegistry, NavigationFailureLayer, RegistryRouting,
    RegistryRoutingError,
};

use super::super::support::{
    cli_value_candidate, config_sources, document_parameter_catalog, navigation_command,
};

#[test]
fn automatic_routing_prefers_case_sensitive_exact_filename_over_suffix() {
    let generic = RecordingAdapter::new();
    let exact = RecordingAdapter::new();
    let non_ascii = RecordingAdapter::new();
    let registry = TestRegistry::new(vec![
        entry(
            &generic,
            manifest("docnav-markdown", "generic-json", &[".json"], None),
        ),
        entry(
            &exact,
            manifest(
                "docnav-other",
                "settings-json",
                &[".settings"],
                Some(&["settings.json"]),
            ),
        ),
        entry(
            &non_ascii,
            manifest("docnav-non-ascii", "non-ascii", &[".配置+V1"], None),
        ),
    ]);

    assert_eq!(
        selected_id(&registry, "config/settings.json"),
        "docnav-other"
    );
    assert_eq!(
        selected_id(&registry, "config/SETTINGS.JSON"),
        "docnav-markdown"
    );
    assert_eq!(
        selected_id(&registry, "config/文档.配置+v1"),
        "docnav-non-ascii"
    );
    let context = select_navigation_context(&registry, "config/settings.json", None, "built_in")
        .expect("automatic pathname routing should produce navigation context");
    assert_eq!(context.source, "automatic_discovery");
}

#[test]
fn automatic_routing_uses_longest_compound_suffix_independent_of_registry_order() {
    let generic = RecordingAdapter::new();
    let compound = RecordingAdapter::new();
    let generic_manifest = manifest("docnav-markdown", "json", &[".json"], None);
    let compound_manifest = manifest("docnav-other", "json-schema", &[".schema.json"], None);
    let generic_first = TestRegistry::new(vec![
        entry(&generic, generic_manifest.clone()),
        entry(&compound, compound_manifest.clone()),
    ]);
    let compound_first = TestRegistry::new(vec![
        entry(&compound, compound_manifest),
        entry(&generic, generic_manifest),
    ]);

    assert_eq!(
        [
            selected_id(&generic_first, "models/model.schema.JSON"),
            selected_id(&compound_first, "models/model.schema.JSON"),
        ],
        ["docnav-other", "docnav-other"]
    );
}

#[test]
fn automatic_suffix_routing_is_anchored_to_the_complete_basename_end() {
    let adapter = RecordingAdapter::new();
    let registry = TestRegistry::new(vec![entry(
        &adapter,
        manifest("docnav-markdown", "json", &[".json"], None),
    )]);

    let error = select_adapter(AdapterSelectionRequest {
        registry: &registry,
        document_path: "settings.json.backup",
        preselected_adapter_id: None,
        preselected_adapter_source: "built_in",
    })
    .expect_err("an interior suffix must not route");
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_eq!(protocol_error.code(), ProtocolDiagnosticCode::FormatUnknown);
    assert_eq!(adapter.outline_calls.load(Ordering::Relaxed), 0);
}

#[test]
fn explicit_adapter_bypasses_pathname_routing_and_executes_selected_strategy() {
    let automatic = RecordingAdapter::new();
    let explicit = RecordingAdapter::new();
    let registry = TestRegistry::new(vec![
        entry(
            &automatic,
            manifest("docnav-markdown", "automatic", &[".automatic"], None),
        ),
        entry(
            &explicit,
            manifest("docnav-other", "explicit", &[".explicit"], None),
        ),
    ]);
    let mut command = navigation_command(vec![cli_value_candidate(
        "docnav.defaults.adapter",
        "--adapter",
        json!("docnav-other"),
    )]);
    command.document_path = "docs/document.automatic".to_owned();

    let outcome = execute_loaded_navigation_command(
        command,
        config_sources(Value::Null, Value::Null),
        &document_parameter_catalog(),
        &registry,
    )
    .expect("explicit lookup should advance to selected strategy execution");

    assert_eq!(
        outcome.trace.selected_adapter_id.as_deref(),
        Some("docnav-other")
    );
    assert!(matches!(outcome.response, ProtocolResponse::Success(_)));
    assert_eq!(explicit.outline_calls.load(Ordering::Relaxed), 1);
    assert_eq!(automatic.outline_calls.load(Ordering::Relaxed), 0);
    let context = select_navigation_context(
        &registry,
        "docs/document.automatic",
        Some("docnav-other"),
        "project",
    )
    .expect("explicit lookup should produce navigation context");
    assert_eq!(context.source, "project");
}

#[test]
fn selected_diagnostic_or_invalid_result_never_dispatches_later_adapter() {
    for behavior in [OutlineBehavior::Diagnostic, OutlineBehavior::InvalidResult] {
        let selected = RecordingAdapter::with_outline_behavior(behavior);
        let later = RecordingAdapter::new();
        let registry = TestRegistry::new(vec![
            entry(
                &selected,
                manifest("docnav-markdown", "selected", &[".selected"], None),
            ),
            entry(&later, manifest("docnav-other", "later", &[".later"], None)),
        ]);
        let mut command = navigation_command(Vec::new());
        command.document_path = "docs/document.selected".to_owned();

        let execution = execute_loaded_navigation_command(
            command,
            config_sources(Value::Null, Value::Null),
            &document_parameter_catalog(),
            &registry,
        );

        assert_eq!(
            selected.outline_calls.load(Ordering::Relaxed),
            1,
            "{behavior:?}"
        );
        assert_eq!(
            later.outline_calls.load(Ordering::Relaxed),
            0,
            "{behavior:?}"
        );
        match behavior {
            OutlineBehavior::Diagnostic => {
                let outcome = execution.expect("adapter diagnostic is a protocol response");
                assert!(matches!(outcome.response, ProtocolResponse::Failure(_)));
            }
            OutlineBehavior::InvalidResult => {
                let error = execution.expect_err("invalid result blocks the invocation");
                assert_eq!(
                    error.failure_layer(),
                    Some(NavigationFailureLayer::ResultValidation)
                );
            }
            OutlineBehavior::Success => unreachable!(),
        }
    }
}

fn selected_id(registry: &TestRegistry<'_>, path: &str) -> String {
    select_adapter(AdapterSelectionRequest {
        registry,
        document_path: path,
        preselected_adapter_id: None,
        preselected_adapter_source: "built_in",
    })
    .expect("pathname should select one adapter")
    .adapter
    .id()
    .to_owned()
}

fn entry<'a>(adapter: &'a RecordingAdapter, manifest: Manifest) -> RegistryEntry<'a> {
    RegistryEntry { adapter, manifest }
}

fn manifest(
    adapter_id: &str,
    format_id: &str,
    extensions: &[&str],
    filenames: Option<&[&str]>,
) -> Manifest {
    let format = json!({
        "id": format_id,
        "extensions": extensions,
        "filenames": filenames.unwrap_or_default(),
        "content_types": [format!("application/x-{format_id}")]
    });
    serde_json::from_value(json!({
        "manifest_version": docnav_protocol::MANIFEST_VERSION,
        "adapter": {
            "id": adapter_id,
            "name": adapter_id,
            "version": "0.1.0"
        },
        "formats": [format]
    }))
    .expect("routing manifest must decode")
}

struct RegistryEntry<'a> {
    adapter: &'a RecordingAdapter,
    manifest: Manifest,
}

struct TestRegistry<'a> {
    entries: Vec<RegistryEntry<'a>>,
    routing: Result<RegistryRouting, RegistryRoutingError>,
}

impl<'a> TestRegistry<'a> {
    fn new(entries: Vec<RegistryEntry<'a>>) -> Self {
        let definitions = entries
            .iter()
            .map(|entry| {
                AdapterDefinition::new(entry.manifest.clone(), entry.adapter, None)
                    .expect("routing test adapter definition")
            })
            .collect::<Vec<_>>();
        Self {
            entries,
            routing: RegistryRouting::from_adapters(&definitions),
        }
    }
}

impl NavigationAdapterRegistry for TestRegistry<'_> {
    fn adapters(&self) -> Vec<AdapterDefinition<'_>> {
        self.entries
            .iter()
            .map(|entry| {
                AdapterDefinition::new(entry.manifest.clone(), entry.adapter, None)
                    .expect("routing test adapter definition")
            })
            .collect()
    }

    fn routing(&self) -> Result<RegistryRouting, RegistryRoutingError> {
        self.routing.clone()
    }
}

struct RecordingAdapter {
    outline_behavior: OutlineBehavior,
    outline_calls: AtomicUsize,
}

impl RecordingAdapter {
    fn new() -> Self {
        Self::with_outline_behavior(OutlineBehavior::Success)
    }

    fn with_outline_behavior(outline_behavior: OutlineBehavior) -> Self {
        Self {
            outline_behavior,
            outline_calls: AtomicUsize::new(0),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum OutlineBehavior {
    Success,
    Diagnostic,
    InvalidResult,
}

impl Adapter for RecordingAdapter {
    fn outline(&self, _input: &OutlineInput) -> AdapterResult<OutlineResult> {
        self.outline_calls.fetch_add(1, Ordering::Relaxed);
        match self.outline_behavior {
            OutlineBehavior::Success => Ok(OutlineResult::structured(Vec::new(), None)),
            OutlineBehavior::Diagnostic => {
                Err(AdapterError::internal("selected-adapter-operation-failed"))
            }
            OutlineBehavior::InvalidResult => {
                Ok(OutlineResult::Structured(StructuredOutlineResult {
                    entries: Vec::new(),
                    page: None,
                    auto_read: Some(AutoReadResult::unique_ref(ReadResult {
                        ref_id: "unexpected".to_owned(),
                        content: "unexpected adapter-owned auto read".to_owned(),
                        content_type: "text/plain".to_owned(),
                        cost: Cost {
                            measurements: Vec::new(),
                        },
                        page: None,
                    })),
                }))
            }
        }
    }

    fn read(&self, _input: &ReadInput) -> AdapterResult<ReadResult> {
        Err(AdapterError::internal("routing-test-read-unreachable"))
    }

    fn find(&self, _input: &FindInput) -> AdapterResult<FindResult> {
        Err(AdapterError::internal("routing-test-find-unreachable"))
    }

    fn info(&self, _input: &InfoInput) -> AdapterResult<InfoResult> {
        Err(AdapterError::internal("routing-test-info-unreachable"))
    }
}
