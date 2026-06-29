use crate::{Adapter, AdapterError, AdapterResult};
use docnav_protocol::{
    try_positive, AdapterIdentity, Entry, FormatDescriptor, InfoArguments, InfoResult, Manifest,
    Operation, OutlineArguments, OutlineResult, ProbeReason, ProbeReasonCode, ProbeResult,
    ReadArguments, ReadResult, RequestEnvelope, PROBE_VERSION,
};

pub(super) fn positive(value: u32) -> docnav_protocol::PositiveInteger {
    try_positive(value).expect("test positive integer")
}

pub(super) struct StubAdapter;

impl Adapter for StubAdapter {
    fn adapter_id(&self) -> &str {
        "stub"
    }

    fn manifest(&self) -> Manifest {
        Manifest {
            manifest_version: docnav_protocol::MANIFEST_VERSION.to_owned(),
            adapter: AdapterIdentity {
                id: "stub".to_owned(),
                name: "Stub Adapter".to_owned(),
                version: "0.1.0".to_owned(),
            },
            formats: vec![FormatDescriptor {
                id: "stub".to_owned(),
                extensions: vec![".stub".to_owned()],
                content_types: vec!["text/stub".to_owned()],
            }],
            capabilities: vec![Operation::Outline, Operation::Info],
        }
    }

    fn probe(&self, path: &str) -> ProbeResult {
        ProbeResult {
            probe_version: PROBE_VERSION.to_owned(),
            adapter_id: "stub".to_owned(),
            path: path.to_owned(),
            supported: true,
            format: Some("stub".to_owned()),
            confidence: 1.0,
            reasons: vec![ProbeReason {
                code: ProbeReasonCode::ExtensionMatch,
                detail: "stub extension".to_owned(),
            }],
        }
    }

    fn outline(
        &self,
        _request: &RequestEnvelope,
        _arguments: &OutlineArguments,
    ) -> AdapterResult<OutlineResult> {
        Ok(OutlineResult {
            entries: vec![Entry {
                ref_id: "L1:Stub".to_owned(),
                label: "Stub".to_owned(),
                kind: None,
                location: None,
                summary: None,
                excerpt: None,
                rank: None,
                cost: None,
                metadata: None,
            }],
            page: None,
        })
    }

    fn info(
        &self,
        _request: &RequestEnvelope,
        _arguments: &InfoArguments,
    ) -> AdapterResult<InfoResult> {
        Ok(InfoResult {
            capabilities: vec![Operation::Outline, Operation::Info],
            document: None,
            adapter: None,
            metadata: None,
        })
    }
}

pub(super) struct InvalidManifestAdapter;

impl Adapter for InvalidManifestAdapter {
    fn adapter_id(&self) -> &str {
        "bad-manifest"
    }

    fn manifest(&self) -> Manifest {
        let mut manifest = StubAdapter.manifest();
        manifest.adapter.id.clear();
        manifest
    }

    fn probe(&self, path: &str) -> ProbeResult {
        StubAdapter.probe(path)
    }
}

pub(super) struct ManifestAdapterIdDriftAdapter;

impl Adapter for ManifestAdapterIdDriftAdapter {
    fn adapter_id(&self) -> &str {
        "stub"
    }

    fn manifest(&self) -> Manifest {
        let mut manifest = StubAdapter.manifest();
        manifest.adapter.id = "drift".to_owned();
        manifest
    }

    fn probe(&self, path: &str) -> ProbeResult {
        StubAdapter.probe(path)
    }
}

pub(super) struct ManifestShapeErrorAdapter;

impl Adapter for ManifestShapeErrorAdapter {
    fn adapter_id(&self) -> &str {
        "stub"
    }

    fn manifest(&self) -> Manifest {
        let mut manifest = StubAdapter.manifest();
        manifest.formats[0].extensions.clear();
        manifest
    }

    fn probe(&self, path: &str) -> ProbeResult {
        StubAdapter.probe(path)
    }
}

pub(super) struct EmptyReasonsProbeAdapter;

impl Adapter for EmptyReasonsProbeAdapter {
    fn adapter_id(&self) -> &str {
        "stub"
    }

    fn manifest(&self) -> Manifest {
        StubAdapter.manifest()
    }

    fn probe(&self, path: &str) -> ProbeResult {
        let mut probe = StubAdapter.probe(path);
        probe.reasons.clear();
        probe
    }
}

pub(super) struct BadConfidenceProbeAdapter;

impl Adapter for BadConfidenceProbeAdapter {
    fn adapter_id(&self) -> &str {
        "stub"
    }

    fn manifest(&self) -> Manifest {
        StubAdapter.manifest()
    }

    fn probe(&self, path: &str) -> ProbeResult {
        let mut probe = StubAdapter.probe(path);
        probe.confidence = 1.5;
        probe
    }
}

pub(super) struct ProbeAdapterIdDriftAdapter;

impl Adapter for ProbeAdapterIdDriftAdapter {
    fn adapter_id(&self) -> &str {
        "stub"
    }

    fn manifest(&self) -> Manifest {
        StubAdapter.manifest()
    }

    fn probe(&self, path: &str) -> ProbeResult {
        let mut probe = StubAdapter.probe(path);
        probe.adapter_id = "drift".to_owned();
        probe
    }
}

pub(super) struct HandlerErrorAdapter;

impl Adapter for HandlerErrorAdapter {
    fn adapter_id(&self) -> &str {
        "stub"
    }

    fn manifest(&self) -> Manifest {
        StubAdapter.manifest()
    }

    fn probe(&self, path: &str) -> ProbeResult {
        StubAdapter.probe(path)
    }

    fn read(
        &self,
        _request: &RequestEnvelope,
        _arguments: &ReadArguments,
    ) -> AdapterResult<ReadResult> {
        Err(AdapterError::ref_not_found("L1:Stub"))
    }
}
