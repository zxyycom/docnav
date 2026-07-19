use std::sync::Mutex;

use docnav_adapter_contracts::{
    Adapter, AdapterDefinition, AdapterError, AdapterResult, FindInput, InfoInput, OutlineInput,
    ReadInput,
};
use docnav_protocol::{
    AdapterIdentity, AutoReadReason, AutoReadResult, Cost, Entry, FindResult, FormatDescriptor,
    InfoResult, Manifest, OperationResult, OutlineResult, ProbeReason, ProbeReasonCode,
    ProbeResult, ProtocolResponse, ReadResult, UnstructuredOutlineReason,
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
}

impl RecordingAdapter {
    fn new(outline_result: OutlineResult, read_result: Option<ReadResult>) -> Self {
        Self {
            outline_result,
            find_result: FindResult::new(Vec::new(), None),
            read_result,
            read_inputs: Mutex::new(Vec::new()),
        }
    }

    fn with_find_result(mut self, find_result: FindResult) -> Self {
        self.find_result = find_result;
        self
    }

    fn read_inputs(&self) -> Vec<ReadInput> {
        self.read_inputs.lock().unwrap().clone()
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
        Ok(self.outline_result.clone())
    }

    fn read(&self, input: &ReadInput) -> AdapterResult<ReadResult> {
        self.read_inputs.lock().unwrap().push(input.clone());
        self.read_result
            .clone()
            .ok_or_else(|| AdapterError::internal("nested-read-failed"))
    }

    fn find(&self, _input: &FindInput) -> AdapterResult<FindResult> {
        Ok(self.find_result.clone())
    }

    fn info(&self, _input: &InfoInput) -> AdapterResult<InfoResult> {
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
            content_types: vec!["text/stub".to_owned()],
        }],
    }
}
