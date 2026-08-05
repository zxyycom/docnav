use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

use docnav_adapter_contracts::{
    Adapter, AdapterDefinition, AdapterDocument, AdapterError, AdapterResult, FindInput, InfoInput,
    OutlineInput, ReadInput, UnstructuredFullRead, UnstructuredFullReadCapabilities,
    UnstructuredFullReadFacts,
};
use docnav_protocol::{
    AdapterIdentity, Cost, Entry, FindResult, FormatDescriptor, InfoResult, Manifest, Measurement,
    OperationResult, OutlineResult, ProtocolResponse, ReadResult, RequestEnvelope,
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
    pub(super) document_creations: AtomicUsize,
    pub(super) source_acquisitions: AtomicUsize,
    pub(super) source_decodes: AtomicUsize,
    pub(super) model_builds: AtomicUsize,
    pub(super) live_documents: AtomicUsize,
    pub(super) peak_live_documents: AtomicUsize,
    pub(super) document_drops: AtomicUsize,
    pub(super) model_drops: AtomicUsize,
    stage_documents: Mutex<Vec<(&'static str, usize)>>,
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
            document_creations: AtomicUsize::new(0),
            source_acquisitions: AtomicUsize::new(0),
            source_decodes: AtomicUsize::new(0),
            model_builds: AtomicUsize::new(0),
            live_documents: AtomicUsize::new(0),
            peak_live_documents: AtomicUsize::new(0),
            document_drops: AtomicUsize::new(0),
            model_drops: AtomicUsize::new(0),
            stage_documents: Mutex::new(Vec::new()),
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

    pub(super) fn stage_documents(&self) -> Vec<(&'static str, usize)> {
        self.stage_documents.lock().unwrap().clone()
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
            filenames: vec![],
            content_types: vec!["text/stub".to_owned()],
        }],
    }
}

impl Adapter for RecordingAdapter {
    fn create_document(&self, _document_path: String) -> Box<dyn AdapterDocument + '_> {
        let id = self.document_creations.fetch_add(1, Ordering::SeqCst) + 1;
        let live = self.live_documents.fetch_add(1, Ordering::SeqCst) + 1;
        self.peak_live_documents.fetch_max(live, Ordering::SeqCst);
        Box::new(RecordingDocument {
            adapter: self,
            id,
            prepared: false,
        })
    }
}

struct RecordingDocument<'a> {
    adapter: &'a RecordingAdapter,
    id: usize,
    prepared: bool,
}

impl RecordingDocument<'_> {
    fn record(&mut self, stage: &'static str) {
        if !self.prepared {
            self.adapter
                .source_acquisitions
                .fetch_add(1, Ordering::SeqCst);
            self.adapter.source_decodes.fetch_add(1, Ordering::SeqCst);
            self.adapter.model_builds.fetch_add(1, Ordering::SeqCst);
            self.prepared = true;
        }
        self.adapter
            .stage_documents
            .lock()
            .unwrap()
            .push((stage, self.id));
    }
}

impl Drop for RecordingDocument<'_> {
    fn drop(&mut self) {
        self.adapter.document_drops.fetch_add(1, Ordering::SeqCst);
        self.adapter.live_documents.fetch_sub(1, Ordering::SeqCst);
        if self.prepared {
            self.adapter.model_drops.fetch_add(1, Ordering::SeqCst);
        }
    }
}

impl AdapterDocument for RecordingDocument<'_> {
    fn outline(&mut self, _input: &OutlineInput) -> AdapterResult<OutlineResult> {
        self.record("outline");
        self.adapter.outline_calls.fetch_add(1, Ordering::SeqCst);
        if self.adapter.fail_outline.load(Ordering::SeqCst) {
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

    fn read(&mut self, _input: &ReadInput) -> AdapterResult<ReadResult> {
        Err(AdapterError::internal("read-unimplemented"))
    }

    fn find(&mut self, _input: &FindInput) -> AdapterResult<FindResult> {
        Err(AdapterError::internal("find-unimplemented"))
    }

    fn info(&mut self, _input: &InfoInput) -> AdapterResult<InfoResult> {
        Err(AdapterError::internal("info-unimplemented"))
    }

    fn unstructured_full_read(
        &mut self,
        _request: &RequestEnvelope,
    ) -> AdapterResult<UnstructuredFullRead> {
        self.record("content");
        Ok(UnstructuredFullRead::new(
            self.adapter.full_read_content.clone(),
            self.adapter.full_read_content_type.clone(),
        ))
    }

    fn measure_unstructured_full_read_cost(
        &mut self,
        _request: &RequestEnvelope,
        requested_units: &[String],
    ) -> AdapterResult<Cost> {
        self.record("cost");
        self.adapter
            .cost_requests
            .lock()
            .unwrap()
            .push(requested_units.to_vec());
        if self.adapter.cost_error.load(Ordering::SeqCst) {
            return Err(AdapterError::internal("measurement-unavailable"));
        }
        Ok(Cost {
            measurements: self.adapter.cost_measurements.clone(),
        })
    }

    fn unstructured_full_read_facts(
        &mut self,
        _request: &RequestEnvelope,
    ) -> AdapterResult<UnstructuredFullReadFacts> {
        self.record("facts");
        Ok(UnstructuredFullReadFacts {
            cost: self.adapter.facts_cost.clone(),
        })
    }
}
