use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

use docnav_adapter_contracts::{
    Adapter, AdapterDefinition, AdapterError, AdapterResult, FindInput, InfoInput, OutlineInput,
    ReadInput, UnstructuredFullRead, UnstructuredFullReadCapabilities, UnstructuredFullReadFacts,
};
use docnav_protocol::{
    AdapterIdentity, Cost, Entry, FindResult, FormatDescriptor, InfoResult, Manifest, Measurement,
    OperationResult, OutlineResult, ProbeReason, ProbeReasonCode, ProbeResult, ProtocolResponse,
    ReadResult, RequestEnvelope,
};
use serde_json::{json, Value};

use crate::NavigationAdapterRegistry;

use super::super::super::support::navigation_command;

pub(super) fn command_for(path: impl Into<String>) -> crate::NavigationCommand {
    let mut command = navigation_command(Vec::new());
    command.document_path = path.into();
    command
}

pub(super) fn threshold_config(unit: &str, value: u64) -> Value {
    json!({
        "outline": {
            "auto_full_read": {
                "thresholds": [
                    {"adapter": "docnav-markdown", "unit": unit, "value": value}
                ]
            }
        }
    })
}

pub(super) fn success_outline(response: ProtocolResponse) -> OutlineResult {
    let ProtocolResponse::Success(success) = response else {
        panic!("expected success response");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };
    result
}

pub(super) fn measurement(unit: &str, value: u64) -> Measurement {
    Measurement {
        unit: unit.to_owned(),
        value,
        scope: Some("full_read".to_owned()),
    }
}

pub(super) struct SingleRegistry<'a> {
    adapter: &'a RecordingAdapter,
}

impl<'a> SingleRegistry<'a> {
    pub(super) fn new(adapter: &'a RecordingAdapter) -> Self {
        Self { adapter }
    }
}

impl NavigationAdapterRegistry for SingleRegistry<'_> {
    fn adapters(&self) -> Vec<AdapterDefinition<'_>> {
        vec![AdapterDefinition::new(
            recording_manifest(),
            self.adapter,
            self.adapter.full_read_capabilities(),
        )
        .expect("valid recording adapter definition")]
    }
}

pub(super) struct RecordingAdapter {
    capabilities: UnstructuredFullReadCapabilities,
    pub(super) outline_calls: AtomicUsize,
    pub(super) fail_outline: AtomicBool,
    pub(super) content_hook: AtomicBool,
    pub(super) result_facts_hook: AtomicBool,
    pub(super) cost_error: AtomicBool,
    pub(super) cost_requests: Mutex<Vec<Vec<String>>>,
    pub(super) cost_measurements: Vec<Measurement>,
    pub(super) full_read_content: String,
    pub(super) full_read_content_type: String,
    pub(super) facts_cost: Option<Cost>,
}

impl Default for RecordingAdapter {
    fn default() -> Self {
        Self {
            capabilities: UnstructuredFullReadCapabilities::default(),
            outline_calls: AtomicUsize::new(0),
            fail_outline: AtomicBool::new(false),
            content_hook: AtomicBool::new(false),
            result_facts_hook: AtomicBool::new(false),
            cost_error: AtomicBool::new(false),
            cost_requests: Mutex::new(Vec::new()),
            cost_measurements: Vec::new(),
            full_read_content: "full read".to_owned(),
            full_read_content_type: "text/plain".to_owned(),
            facts_cost: None,
        }
    }
}

impl RecordingAdapter {
    pub(super) fn with_cost_units(units: impl IntoIterator<Item = &'static str>) -> Self {
        Self {
            capabilities: UnstructuredFullReadCapabilities {
                content_hook: false,
                cost_measurement_units: units.into_iter().map(str::to_owned).collect(),
                result_facts_hook: false,
            },
            ..Self::default()
        }
    }

    fn full_read_capabilities(&self) -> Option<UnstructuredFullReadCapabilities> {
        let capabilities = UnstructuredFullReadCapabilities {
            content_hook: self.content_hook.load(Ordering::SeqCst)
                || self.capabilities.content_hook,
            cost_measurement_units: self.capabilities.cost_measurement_units.clone(),
            result_facts_hook: self.result_facts_hook.load(Ordering::SeqCst)
                || self.capabilities.result_facts_hook,
        };
        (capabilities != UnstructuredFullReadCapabilities::default()).then_some(capabilities)
    }
}

fn recording_manifest() -> Manifest {
    Manifest {
        manifest_version: docnav_protocol::MANIFEST_VERSION.to_owned(),
        adapter: AdapterIdentity {
            id: "docnav-markdown".to_owned(),
            name: "Recording".to_owned(),
            version: "0.1.0".to_owned(),
        },
        formats: vec![FormatDescriptor {
            id: "stub".to_owned(),
            extensions: vec![".stub".to_owned()],
            content_types: vec!["text/stub".to_owned()],
        }],
    }
}

impl Adapter for RecordingAdapter {
    fn probe(&self, path: &str) -> ProbeResult {
        ProbeResult {
            probe_version: docnav_protocol::PROBE_VERSION.to_owned(),
            adapter_id: "docnav-markdown".to_owned(),
            path: path.to_owned(),
            supported: true,
            format: Some("stub".to_owned()),
            confidence: 1.0,
            reasons: vec![ProbeReason {
                code: ProbeReasonCode::ContentMatch,
                detail: "recording adapter accepts test path".to_owned(),
            }],
        }
    }

    fn outline(&self, _input: &OutlineInput) -> AdapterResult<OutlineResult> {
        self.outline_calls.fetch_add(1, Ordering::SeqCst);
        if self.fail_outline.load(Ordering::SeqCst) {
            return Err(AdapterError::internal("outline-should-not-run"));
        }
        Ok(OutlineResult::structured(
            vec![Entry {
                ref_id: "stub:1".to_owned(),
                label: "structured outline".to_owned(),
                kind: None,
                location: None,
                summary: None,
                excerpt: None,
                rank: None,
                cost: None,
                metadata: None,
            }],
            None,
        ))
    }

    fn read(&self, _input: &ReadInput) -> AdapterResult<ReadResult> {
        Err(AdapterError::internal("read-unimplemented"))
    }

    fn find(&self, _input: &FindInput) -> AdapterResult<FindResult> {
        Err(AdapterError::internal("find-unimplemented"))
    }

    fn info(&self, _input: &InfoInput) -> AdapterResult<InfoResult> {
        Err(AdapterError::internal("info-unimplemented"))
    }

    fn unstructured_full_read(
        &self,
        _request: &RequestEnvelope,
    ) -> AdapterResult<UnstructuredFullRead> {
        Ok(UnstructuredFullRead::new(
            self.full_read_content.clone(),
            self.full_read_content_type.clone(),
        ))
    }

    fn measure_unstructured_full_read_cost(
        &self,
        _request: &RequestEnvelope,
        requested_units: &[String],
    ) -> AdapterResult<Cost> {
        self.cost_requests
            .lock()
            .unwrap()
            .push(requested_units.to_vec());
        if self.cost_error.load(Ordering::SeqCst) {
            return Err(AdapterError::internal("measurement-unavailable"));
        }
        Ok(Cost {
            measurements: self.cost_measurements.clone(),
        })
    }

    fn unstructured_full_read_facts(
        &self,
        _request: &RequestEnvelope,
    ) -> AdapterResult<UnstructuredFullReadFacts> {
        Ok(UnstructuredFullReadFacts {
            cost: self.facts_cost.clone(),
        })
    }
}
