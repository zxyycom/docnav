use super::*;

pub(super) fn execute(
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

pub(super) fn outline_result(response: ProtocolResponse) -> OutlineResult {
    let ProtocolResponse::Success(success) = response else {
        panic!("expected success response");
    };
    let OperationResult::Outline(result) = success.result else {
        panic!("expected outline result");
    };
    result
}

pub(super) fn find_result(response: ProtocolResponse) -> FindResult {
    let ProtocolResponse::Success(success) = response else {
        panic!("expected success response");
    };
    let OperationResult::Find(result) = success.result else {
        panic!("expected find result");
    };
    result
}

pub(super) fn read_result(ref_id: &str) -> ReadResult {
    ReadResult {
        ref_id: ref_id.to_owned(),
        content: "selected content".to_owned(),
        content_type: "text/markdown".to_owned(),
        cost: read_cost(),
        page: None,
    }
}

pub(super) fn read_cost() -> Cost {
    Cost {
        measurements: vec![Measurement {
            unit: "bytes".to_owned(),
            value: 16,
            scope: None,
        }],
    }
}

pub(super) fn empty_cost() -> Cost {
    Cost {
        measurements: Vec::new(),
    }
}

pub(super) fn command(
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

pub(super) fn entry(ref_id: &str, label: &str) -> Entry {
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

pub(super) fn positive(value: u32) -> Option<docnav_protocol::PositiveInteger> {
    docnav_protocol::PositiveInteger::new(value)
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
        vec![
            AdapterDefinition::new(recording_manifest(), self.adapter, None)
                .expect("valid recording adapter definition"),
        ]
    }
}

pub(super) struct RecordingAdapter {
    outline_result: OutlineResult,
    find_result: FindResult,
    read_result: Option<ReadResult>,
    read_inputs: Mutex<Vec<ReadInput>>,
    shared_view_reads: Mutex<Vec<bool>>,
    pub(super) document_creations: AtomicUsize,
    pub(super) source_acquisitions: AtomicUsize,
    pub(super) source_decodes: AtomicUsize,
    pub(super) model_builds: AtomicUsize,
    pub(super) ref_productions: AtomicUsize,
    pub(super) read_resolutions: AtomicUsize,
    pub(super) live_documents: AtomicUsize,
    pub(super) peak_live_documents: AtomicUsize,
    pub(super) document_drops: AtomicUsize,
    pub(super) model_drops: AtomicUsize,
    panic_outline: bool,
}

impl RecordingAdapter {
    pub(super) fn new(outline_result: OutlineResult, read_result: Option<ReadResult>) -> Self {
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

    pub(super) fn with_find_result(mut self, find_result: FindResult) -> Self {
        self.find_result = find_result;
        self
    }

    pub(super) fn panicking_on_outline(mut self) -> Self {
        self.panic_outline = true;
        self
    }

    pub(super) fn read_inputs(&self) -> Vec<ReadInput> {
        self.read_inputs.lock().unwrap().clone()
    }

    pub(super) fn shared_view_reads(&self) -> Vec<bool> {
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
