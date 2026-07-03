use docnav_adapter_contracts::AdapterResult;
use docnav_protocol::{
    AdapterIdentity, FindArguments, FindResult, FormatDescriptor, InfoArguments, InfoResult,
    OutlineArguments, OutlineResult, ProbeReason, ProbeReasonCode, ProbeResult, ReadArguments,
    ReadResult, RequestEnvelope, MANIFEST_VERSION, PROBE_VERSION,
};

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
    assert!(manifest
        .formats
        .iter()
        .any(|format| format.id == "markdown"));
}

#[test]
fn adapter_layer_check_fails_when_manifest_id_mismatches_registry_id() {
    static MISMATCHED: MismatchedManifestAdapter = MismatchedManifestAdapter;
    let adapters = [AdapterRecord::from_adapter(&MISMATCHED)];
    let registry = AdapterRegistry {
        adapters: Box::leak(Box::new(adapters)),
    };

    let checks = adapter_layer_checks(&registry);
    let check = checks.first().expect("adapter layer check");

    assert_eq!(check.get("status").and_then(Value::as_str), Some("fail"));
    assert_eq!(
        check.get("message").and_then(Value::as_str),
        Some("built-in adapter layer metadata is invalid")
    );
}

#[test]
fn static_registry_exposes_full_native_option_specs() {
    let registry = AdapterRegistry { adapters: ADAPTERS };
    let native_options = registry.native_options_for(Operation::Outline);

    assert!(registry.has_native_option_config_key("options.max_heading_level"));
    assert!(registry
        .native_option_config_keys()
        .contains(&"options.max_heading_level".to_owned()));
    assert!(native_options.iter().any(|option| {
        option.owner == "docnav-markdown"
            && option.namespace == "options"
            && option.key == "max_heading_level"
    }));
}

struct MismatchedManifestAdapter;

impl Adapter for MismatchedManifestAdapter {
    fn adapter_id(&self) -> &str {
        "registry-id"
    }

    fn manifest(&self) -> Manifest {
        Manifest {
            manifest_version: MANIFEST_VERSION.to_owned(),
            adapter: AdapterIdentity {
                id: "manifest-id".to_owned(),
                name: "Mismatched".to_owned(),
                version: "0.1.0".to_owned(),
            },
            formats: vec![FormatDescriptor {
                id: "mismatched".to_owned(),
                extensions: vec![".mismatch".to_owned()],
                content_types: vec!["text/mismatched".to_owned()],
            }],
        }
    }

    fn probe(&self, path: &str) -> ProbeResult {
        ProbeResult {
            probe_version: PROBE_VERSION.to_owned(),
            adapter_id: self.adapter_id().to_owned(),
            path: path.to_owned(),
            supported: true,
            format: Some("partial".to_owned()),
            confidence: 1.0,
            reasons: vec![ProbeReason {
                code: ProbeReasonCode::ContentMatch,
                detail: "test adapter".to_owned(),
            }],
        }
    }

    fn outline(
        &self,
        _request: &RequestEnvelope,
        _arguments: &OutlineArguments,
    ) -> AdapterResult<OutlineResult> {
        unreachable!("registry tests only inspect manifest metadata")
    }

    fn read(
        &self,
        _request: &RequestEnvelope,
        _arguments: &ReadArguments,
    ) -> AdapterResult<ReadResult> {
        unreachable!("registry tests only inspect manifest metadata")
    }

    fn find(
        &self,
        _request: &RequestEnvelope,
        _arguments: &FindArguments,
    ) -> AdapterResult<FindResult> {
        unreachable!("registry tests only inspect manifest metadata")
    }

    fn info(
        &self,
        _request: &RequestEnvelope,
        _arguments: &InfoArguments,
    ) -> AdapterResult<InfoResult> {
        unreachable!("registry tests only inspect manifest metadata")
    }
}
