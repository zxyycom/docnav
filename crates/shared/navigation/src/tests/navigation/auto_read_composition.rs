use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use docnav_adapter_contracts::{
    Adapter, AdapterDefinition, AdapterDocument, AdapterError, AdapterResult, FindInput, InfoInput,
    OutlineInput, ReadInput,
};
use docnav_protocol::{
    AdapterIdentity, AutoReadReason, AutoReadResult, Cost, Entry, FindResult, FormatDescriptor,
    InfoResult, Manifest, OperationResult, OutlineResult, ProtocolResponse, ReadResult,
    UnstructuredOutlineReason,
};
use serde_json::json;

use crate::{
    execute_loaded_navigation_command, NavigationAdapterRegistry, NavigationCommand,
    NavigationFailureLayer,
};

use super::super::support::{
    cli_value_candidate, config_sources, document_parameter_catalog, navigation_command,
};

mod find;
mod outline;

fn execute(
    adapter: &RecordingAdapter,
    command: NavigationCommand,
) -> crate::NavigationCommandOutcome {
    execute_loaded_navigation_command(
        command,
        config_sources(json!({}), json!({})),
        &document_parameter_catalog(),
        &SingleRegistry::new(adapter),
    )
    .expect("navigation success")
}

fn outline_result(response: ProtocolResponse) -> OutlineResult {
    let ProtocolResponse::Success(success) = response else {
        panic!("expected success response");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };
    result
}

fn find_result(response: ProtocolResponse) -> FindResult {
    let ProtocolResponse::Success(success) = response else {
        panic!("expected success response");
    };
    let OperationResult::Find(result) = success.result else {
        panic!("expected find result");
    };
    result
}

fn read_result(ref_id: &str) -> ReadResult {
    ReadResult {
        ref_id: ref_id.to_owned(),
        content: "selected content".to_owned(),
        content_type: "text/markdown".to_owned(),
        cost: empty_cost(),
        page: None,
    }
}

fn empty_cost() -> Cost {
    Cost {
        measurements: Vec::new(),
    }
}

fn command(
    operation: docnav_protocol::Operation,
    candidates: Vec<cli_config_resolution::SourceCandidate>,
) -> NavigationCommand {
    let mut command = navigation_command(candidates);
    command.operation = operation;
    if operation == docnav_protocol::Operation::Find {
        command.query = Some("needle".to_owned());
    }
    command
}

fn entry(ref_id: &str, label: &str) -> Entry {
    Entry {
        ref_id: ref_id.to_owned(),
        label: label.to_owned(),
        kind: None,
        location: None,
        summary: None,
        excerpt: None,
        rank: None,
        cost: None,
        metadata: None,
    }
}

fn positive(value: u32) -> Option<docnav_protocol::PositiveInteger> {
    docnav_protocol::PositiveInteger::new(value)
}

struct SingleRegistry<'a> {
    adapter: &'a RecordingAdapter,
}

impl<'a> SingleRegistry<'a> {
    fn new(adapter: &'a RecordingAdapter) -> Self {
        Self { adapter }
    }
}

impl NavigationAdapterRegistry for SingleRegistry<'_> {
    fn adapters(&self) -> Vec<AdapterDefinition<'_>> {
        vec![
            AdapterDefinition::new(recording_manifest(), self.adapter, None)
                .expect("valid recording adapter definition"),
        ]
    }
}

struct RecordingAdapter {
    outline_result: OutlineResult,
    find_result: FindResult,
    read_result: Option<ReadResult>,
    read_inputs: Mutex<Vec<ReadInput>>,
    shared_view_reads: Mutex<Vec<bool>>,
    document_creations: AtomicUsize,
    source_acquisitions: AtomicUsize,
    source_decodes: AtomicUsize,
    model_builds: AtomicUsize,
    ref_productions: AtomicUsize,
    read_resolutions: AtomicUsize,
    live_documents: AtomicUsize,
    peak_live_documents: AtomicUsize,
    document_drops: AtomicUsize,
    model_drops: AtomicUsize,
    panic_outline: bool,
}

impl RecordingAdapter {
    fn new(outline_result: OutlineResult, read_result: Option<ReadResult>) -> Self {
        Self {
            outline_result,
            find_result: FindResult::new(Vec::new(), None),
            read_result,
            read_inputs: Mutex::new(Vec::new()),
            shared_view_reads: Mutex::new(Vec::new()),
            document_creations: AtomicUsize::new(0),
            source_acquisitions: AtomicUsize::new(0),
            source_decodes: AtomicUsize::new(0),
            model_builds: AtomicUsize::new(0),
            ref_productions: AtomicUsize::new(0),
            read_resolutions: AtomicUsize::new(0),
            live_documents: AtomicUsize::new(0),
            peak_live_documents: AtomicUsize::new(0),
            document_drops: AtomicUsize::new(0),
            model_drops: AtomicUsize::new(0),
            panic_outline: false,
        }
    }

    fn with_find_result(mut self, find_result: FindResult) -> Self {
        self.find_result = find_result;
        self
    }

    fn panicking_on_outline(mut self) -> Self {
        self.panic_outline = true;
        self
    }

    fn read_inputs(&self) -> Vec<ReadInput> {
        self.read_inputs.lock().unwrap().clone()
    }

    fn shared_view_reads(&self) -> Vec<bool> {
        self.shared_view_reads.lock().unwrap().clone()
    }
}

impl Adapter for RecordingAdapter {
    fn create_document(&self, _document_path: String) -> Box<dyn AdapterDocument + '_> {
        self.document_creations.fetch_add(1, Ordering::SeqCst);
        let live = self.live_documents.fetch_add(1, Ordering::SeqCst) + 1;
        self.peak_live_documents.fetch_max(live, Ordering::SeqCst);
        Box::new(RecordingDocument {
            adapter: self,
            base_operation_seen: false,
            prepared: false,
        })
    }
}

struct RecordingDocument<'a> {
    adapter: &'a RecordingAdapter,
    base_operation_seen: bool,
    prepared: bool,
}

impl RecordingDocument<'_> {
    fn prepare(&mut self) {
        if !self.prepared {
            self.adapter
                .source_acquisitions
                .fetch_add(1, Ordering::SeqCst);
            self.adapter.source_decodes.fetch_add(1, Ordering::SeqCst);
            self.adapter.model_builds.fetch_add(1, Ordering::SeqCst);
            self.prepared = true;
        }
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
        self.prepare();
        assert!(!self.adapter.panic_outline, "fixture outline panic");
        self.adapter.ref_productions.fetch_add(1, Ordering::SeqCst);
        self.base_operation_seen = true;
        Ok(self.adapter.outline_result.clone())
    }

    fn read(&mut self, input: &ReadInput) -> AdapterResult<ReadResult> {
        self.prepare();
        self.adapter.read_resolutions.fetch_add(1, Ordering::SeqCst);
        self.adapter.read_inputs.lock().unwrap().push(input.clone());
        self.adapter
            .shared_view_reads
            .lock()
            .unwrap()
            .push(self.base_operation_seen);
        self.adapter
            .read_result
            .clone()
            .ok_or_else(|| AdapterError::internal("nested-read-failed"))
    }

    fn find(&mut self, _input: &FindInput) -> AdapterResult<FindResult> {
        self.prepare();
        self.adapter.ref_productions.fetch_add(1, Ordering::SeqCst);
        self.base_operation_seen = true;
        Ok(self.adapter.find_result.clone())
    }

    fn info(&mut self, _input: &InfoInput) -> AdapterResult<InfoResult> {
        self.prepare();
        Err(AdapterError::internal("info-unimplemented"))
    }
}

#[test]
fn request_construction_failure_does_not_create_an_adapter_document() {
    let adapter = RecordingAdapter::new(OutlineResult::structured(Vec::new(), None), None);
    let command = command(docnav_protocol::Operation::Read, Vec::new());

    let error = execute_loaded_navigation_command(
        command,
        config_sources(json!({}), json!({})),
        &document_parameter_catalog(),
        &SingleRegistry::new(&adapter),
    )
    .expect_err("missing ref fails request construction");

    assert_eq!(
        error.failure_layer(),
        Some(NavigationFailureLayer::RequestConstruction)
    );
    assert_eq!(adapter.document_creations.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.source_acquisitions.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.source_decodes.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.model_builds.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.ref_productions.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.read_resolutions.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.peak_live_documents.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.live_documents.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.document_drops.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.model_drops.load(Ordering::SeqCst), 0);
}

#[test]
fn adapter_document_is_dropped_during_handler_unwind() {
    let adapter = RecordingAdapter::new(OutlineResult::structured(Vec::new(), None), None)
        .panicking_on_outline();

    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = execute(
            &adapter,
            command(docnav_protocol::Operation::Outline, Vec::new()),
        );
    }));

    assert!(unwind.is_err());
    assert_eq!(adapter.document_creations.load(Ordering::SeqCst), 1);
    assert_eq!(adapter.source_acquisitions.load(Ordering::SeqCst), 1);
    assert_eq!(adapter.source_decodes.load(Ordering::SeqCst), 1);
    assert_eq!(adapter.model_builds.load(Ordering::SeqCst), 1);
    assert_eq!(adapter.ref_productions.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.read_resolutions.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.peak_live_documents.load(Ordering::SeqCst), 1);
    assert_eq!(adapter.live_documents.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.document_drops.load(Ordering::SeqCst), 1);
    assert_eq!(adapter.model_drops.load(Ordering::SeqCst), 1);
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
