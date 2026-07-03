use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use docnav_adapter_contracts::{
    Adapter, AdapterError, AdapterResult, NativeOptionSpec, NativeOptionValueSpec,
};
use docnav_parameter_resolution::LoadedParameterConfigSource;
use docnav_protocol::{
    positive_result, AdapterIdentity, Entry, FindArguments, FindResult, FormatDescriptor,
    InfoArguments, InfoResult, Manifest, Operation, OutlineArguments, OutlineResult, ProbeReason,
    ProbeReasonCode, ProbeResult, ReadArguments, ReadResult, RequestEnvelope,
};
use serde_json::Value;

use crate::{
    NavigationAdapterRef, NavigationAdapterRegistry, NavigationCommand, NavigationConfigSource,
    NavigationConfigSources, NavigationNativeOptionInput,
};

const MAX_HEADING_LEVEL_OPERATIONS: &[Operation] = &[Operation::Outline];
const MAX_HEADING_LEVEL_OPTION: NativeOptionSpec =
    NativeOptionSpec::builder("docnav.adapters.docnav-markdown.options.max_heading_level")
        .owner("docnav-markdown")
        .namespace("options")
        .key("max_heading_level")
        .operations(MAX_HEADING_LEVEL_OPERATIONS)
        .cli_flag("--max-heading-level")
        .value(NativeOptionValueSpec::Integer { min: 1, max: 6 })
        .default_integer(3)
        .build();
const PAYLOAD_OPTION: NativeOptionSpec =
    NativeOptionSpec::builder("docnav.adapters.docnav-markdown.options.payload")
        .owner("docnav-markdown")
        .namespace("options")
        .key("payload")
        .operations(MAX_HEADING_LEVEL_OPERATIONS)
        .cli_flag("--payload")
        .value(NativeOptionValueSpec::Json)
        .build();
const NATIVE_OPTIONS: &[NativeOptionSpec] = &[MAX_HEADING_LEVEL_OPTION, PAYLOAD_OPTION];

pub(super) fn navigation_command(
    native_options: Vec<NavigationNativeOptionInput>,
) -> NavigationCommand {
    NavigationCommand {
        operation: Operation::Outline,
        document_path: "docs/guide.stub".to_owned(),
        ref_id: None,
        query: None,
        page: Some(positive_result(1).unwrap()),
        pagination_enabled: None,
        limit: None,
        output: None,
        adapter: None,
        native_options,
    }
}

pub(super) fn config_sources(project: Value, user: Value) -> NavigationConfigSources {
    NavigationConfigSources {
        project: NavigationConfigSource {
            level: "project",
            path: "project/.docnav/docnav.json".to_owned(),
            loaded: LoadedParameterConfigSource::from_value(project),
        },
        user: NavigationConfigSource {
            level: "user",
            path: "user/docnav.json".to_owned(),
            loaded: LoadedParameterConfigSource::from_value(user),
        },
    }
}

pub(super) fn write_config_file(path: &Path, value: Value) {
    fs::create_dir_all(path.parent().expect("config parent")).unwrap();
    fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

pub(super) fn diagnostic_record(
    diagnostic: &docnav_diagnostics::DiagnosticRecordDraft,
) -> docnav_diagnostics::DiagnosticRecord {
    let mut stack = docnav_diagnostics::DiagnosticStack::new();
    let id = stack.push(diagnostic.clone()).expect("valid diagnostic");
    stack.get(id).expect("diagnostic record").clone()
}

pub(super) fn temp_workspace_path(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push("docnav-navigation-tests");
    path.push(format!("{name}-{suffix}"));
    fs::create_dir_all(&path).unwrap();
    path
}

pub(super) struct StubRegistry;

impl NavigationAdapterRegistry for StubRegistry {
    fn adapters(&self) -> Vec<NavigationAdapterRef<'_>> {
        vec![NavigationAdapterRef {
            id: "docnav-markdown",
            adapter: &StubAdapter,
        }]
    }
}

struct StubAdapter;

impl Adapter for StubAdapter {
    fn adapter_id(&self) -> &str {
        "docnav-markdown"
    }

    fn manifest(&self) -> Manifest {
        Manifest {
            manifest_version: "0.1".to_owned(),
            adapter: AdapterIdentity {
                id: "docnav-markdown".to_owned(),
                name: "Stub".to_owned(),
                version: "0.1.0".to_owned(),
            },
            formats: vec![FormatDescriptor {
                id: "stub".to_owned(),
                extensions: vec![".stub".to_owned()],
                content_types: vec!["text/stub".to_owned()],
            }],
        }
    }

    fn native_options(&self) -> &'static [NativeOptionSpec] {
        NATIVE_OPTIONS
    }

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
                detail: "stub probe accepted".to_owned(),
            }],
        }
    }

    fn outline(
        &self,
        _request: &RequestEnvelope,
        arguments: &OutlineArguments,
    ) -> AdapterResult<OutlineResult> {
        let label = arguments
            .options
            .as_ref()
            .and_then(|options| options.get("payload"))
            .map(|value| format!("Payload {value}"))
            .or_else(|| {
                arguments
                    .options
                    .as_ref()
                    .and_then(|options| options.get("max_heading_level"))
                    .map(|value| format!("Max {value}"))
            })
            .unwrap_or_else(|| "Stub".to_owned());
        Ok(OutlineResult {
            entries: vec![Entry {
                ref_id: "stub:1".to_owned(),
                label,
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

    fn read(
        &self,
        _request: &RequestEnvelope,
        _arguments: &ReadArguments,
    ) -> AdapterResult<ReadResult> {
        Err(AdapterError::ref_not_found("missing"))
    }

    fn find(
        &self,
        _request: &RequestEnvelope,
        _arguments: &FindArguments,
    ) -> AdapterResult<FindResult> {
        Err(AdapterError::invalid_request(
            "arguments.query",
            "query is not indexed",
        ))
    }

    fn info(
        &self,
        _request: &RequestEnvelope,
        _arguments: &InfoArguments,
    ) -> AdapterResult<InfoResult> {
        Err(AdapterError::internal("stub-info-unimplemented"))
    }
}
