use std::path::PathBuf;

use docnav_adapter_contracts::{Adapter, AdapterResult};
use docnav_diagnostics::DiagnosticStack;
use docnav_protocol::{
    AdapterIdentity, FindArguments, FindResult, FormatDescriptor, InfoArguments, InfoResult,
    Manifest, OutlineArguments, OutlineResult, ProbeReason, ProbeReasonCode, ProbeResult,
    ProtocolDiagnosticCode, ProtocolError, ReadArguments, ReadResult, RequestEnvelope,
    MANIFEST_VERSION, PROBE_VERSION,
};

use super::*;

static REGISTRY_FIRST: RegistryFirstAdapter = RegistryFirstAdapter;
static MARKDOWNISH: MarkdownishAdapter = MarkdownishAdapter;
static UNSUPPORTED_PROBE: UnsupportedProbeAdapter = UnsupportedProbeAdapter;

// @case WB-CORE-ADAPTER-SOURCE-001
#[test]
fn explicit_missing_adapter_guidance_stays_on_static_registry() {
    let registry = AdapterRegistry { adapters: &[] };
    let document = NormalizedDocumentPath {
        adapter_path: "docs/guide.md".to_owned(),
        absolute_path: PathBuf::from("docs/guide.md"),
    };

    let error = select_adapter(AdapterSelectionRequest {
        registry: &registry,
        document: &document,
        preselected_adapter_id: Some("custom-local-adapter"),
        preselected_adapter_source: "cli",
    })
    .expect_err("missing explicit adapter should fail");

    let mut diagnostics = DiagnosticStack::new();
    let id = diagnostics
        .push(error.diagnostic().clone())
        .expect("routing diagnostic should be valid");
    let record = diagnostics.get(id).expect("diagnostic record");
    let protocol_error =
        ProtocolError::from_diagnostic_record(record).expect("protocol projection");

    assert_eq!(
        protocol_error.code(),
        ProtocolDiagnosticCode::AdapterUnavailable
    );
    assert_eq!(protocol_error.owner(), "adapter_selection");
    let guidance = protocol_error
        .guidance()
        .and_then(|items| items.first())
        .expect("adapter selection guidance");
    assert!(guidance.contains("current core release static registry"));
    for removed_term in ["install", "register", "executable", "artifact"] {
        assert!(
            !guidance.contains(removed_term),
            "guidance should not mention {removed_term}: {guidance}"
        );
    }
}

#[test]
fn automatic_discovery_probes_static_registry_order_without_extension_preselection() {
    let adapters = [
        AdapterRecord::from_adapter(&REGISTRY_FIRST),
        AdapterRecord::from_adapter(&MARKDOWNISH),
    ];
    let registry = AdapterRegistry {
        adapters: Box::leak(Box::new(adapters)),
    };
    let document = normalized_document("docs/guide.md");

    let selection = select_adapter(AdapterSelectionRequest {
        registry: &registry,
        document: &document,
        preselected_adapter_id: None,
        preselected_adapter_source: "none",
    })
    .expect("first registry adapter probe should be selected");

    assert_eq!(selection.record.id(), "registry-first");
    assert!(selection.evidence.is_empty());
}

#[test]
fn automatic_discovery_candidate_failure_comes_from_probe_result() {
    let adapters = [AdapterRecord::from_adapter(&UNSUPPORTED_PROBE)];
    let registry = AdapterRegistry {
        adapters: Box::leak(Box::new(adapters)),
    };
    let document = normalized_document("docs/guide.missing");

    let error = select_adapter(AdapterSelectionRequest {
        registry: &registry,
        document: &document,
        preselected_adapter_id: None,
        preselected_adapter_source: "none",
    })
    .expect_err("unsupported probe should produce format unknown");

    let mut diagnostics = DiagnosticStack::new();
    let id = diagnostics
        .push(error.diagnostic().clone())
        .expect("routing diagnostic should be valid");
    let record = diagnostics.get(id).expect("diagnostic record");
    let protocol_error =
        ProtocolError::from_diagnostic_record(record).expect("protocol projection");
    let candidates = protocol_error
        .details()
        .get("candidates")
        .and_then(|value| value.as_array())
        .expect("candidate failure details");
    let candidate = candidates.first().expect("candidate failure");

    assert_eq!(protocol_error.code(), ProtocolDiagnosticCode::FormatUnknown);
    assert_eq!(
        candidate.get("reason").and_then(|value| value.as_str()),
        Some("PROBE_UNSUPPORTED")
    );
}

fn normalized_document(path: &str) -> NormalizedDocumentPath {
    NormalizedDocumentPath {
        adapter_path: path.to_owned(),
        absolute_path: PathBuf::from(path),
    }
}

fn manifest(id: &str, extension: &str) -> Manifest {
    Manifest {
        manifest_version: MANIFEST_VERSION.to_owned(),
        adapter: AdapterIdentity {
            id: id.to_owned(),
            name: id.to_owned(),
            version: "0.1.0".to_owned(),
        },
        formats: vec![FormatDescriptor {
            id: id.to_owned(),
            extensions: vec![extension.to_owned()],
            content_types: vec![format!("text/{id}")],
        }],
    }
}

fn probe(adapter_id: &str, path: &str, supported: bool) -> ProbeResult {
    ProbeResult {
        probe_version: PROBE_VERSION.to_owned(),
        adapter_id: adapter_id.to_owned(),
        path: path.to_owned(),
        supported,
        format: supported.then(|| adapter_id.to_owned()),
        confidence: if supported { 1.0 } else { 0.0 },
        reasons: vec![ProbeReason {
            code: ProbeReasonCode::ContentMatch,
            detail: "test probe result".to_owned(),
        }],
    }
}

struct RegistryFirstAdapter;
struct MarkdownishAdapter;
struct UnsupportedProbeAdapter;

macro_rules! adapter_operation_stubs {
    () => {
        fn outline(
            &self,
            _request: &RequestEnvelope,
            _arguments: &OutlineArguments,
        ) -> AdapterResult<OutlineResult> {
            unreachable!("routing tests only probe adapter candidates")
        }

        fn read(
            &self,
            _request: &RequestEnvelope,
            _arguments: &ReadArguments,
        ) -> AdapterResult<ReadResult> {
            unreachable!("routing tests only probe adapter candidates")
        }

        fn find(
            &self,
            _request: &RequestEnvelope,
            _arguments: &FindArguments,
        ) -> AdapterResult<FindResult> {
            unreachable!("routing tests only probe adapter candidates")
        }

        fn info(
            &self,
            _request: &RequestEnvelope,
            _arguments: &InfoArguments,
        ) -> AdapterResult<InfoResult> {
            unreachable!("routing tests only probe adapter candidates")
        }
    };
}

impl Adapter for RegistryFirstAdapter {
    fn adapter_id(&self) -> &str {
        "registry-first"
    }

    fn manifest(&self) -> Manifest {
        manifest(self.adapter_id(), ".first")
    }

    fn probe(&self, path: &str) -> ProbeResult {
        probe(self.adapter_id(), path, true)
    }

    adapter_operation_stubs!();
}

impl Adapter for MarkdownishAdapter {
    fn adapter_id(&self) -> &str {
        "markdownish"
    }

    fn manifest(&self) -> Manifest {
        manifest(self.adapter_id(), ".md")
    }

    fn probe(&self, path: &str) -> ProbeResult {
        probe(self.adapter_id(), path, true)
    }

    adapter_operation_stubs!();
}

impl Adapter for UnsupportedProbeAdapter {
    fn adapter_id(&self) -> &str {
        "unsupported-probe"
    }

    fn manifest(&self) -> Manifest {
        manifest(self.adapter_id(), ".unsupported")
    }

    fn probe(&self, path: &str) -> ProbeResult {
        probe(self.adapter_id(), path, false)
    }

    adapter_operation_stubs!();
}
