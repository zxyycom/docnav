use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use docnav_adapter_contracts::{
    Adapter, AdapterError, AdapterOptionProcessStrategy, AdapterOptionSpec, AdapterResult,
    FieldBound, FieldValidation,
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
    diagnostic.clone().into_record().expect("valid diagnostic")
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

pub(super) struct UnsupportedRegistry;

impl NavigationAdapterRegistry for UnsupportedRegistry {
    fn adapters(&self) -> Vec<NavigationAdapterRef<'_>> {
        vec![NavigationAdapterRef {
            id: "docnav-unsupported",
            adapter: &UnsupportedAdapter,
        }]
    }
}

pub(super) struct InvalidOptionRegistry;

impl NavigationAdapterRegistry for InvalidOptionRegistry {
    fn adapters(&self) -> Vec<NavigationAdapterRef<'_>> {
        vec![NavigationAdapterRef {
            id: "docnav-invalid-options",
            adapter: &InvalidOptionAdapter,
        }]
    }
}

pub(super) struct InvalidOptionConfigPathRegistry;

impl NavigationAdapterRegistry for InvalidOptionConfigPathRegistry {
    fn adapters(&self) -> Vec<NavigationAdapterRef<'_>> {
        vec![NavigationAdapterRef {
            id: "docnav-invalid-option-config-path",
            adapter: &InvalidOptionConfigPathAdapter,
        }]
    }
}

fn stub_adapter_options() -> Vec<AdapterOptionSpec> {
    vec![
        AdapterOptionSpec::builder("docnav.adapters.docnav-markdown.options.max_heading_level")
            .owner("docnav-markdown")
            .operations(MAX_HEADING_LEVEL_OPERATIONS)
            .path(["options", "max_heading_level"])
            .process(
                "cli",
                AdapterOptionProcessStrategy::cli_flag("--max-heading-level"),
            )
            .process(
                "config",
                AdapterOptionProcessStrategy::json_path(["options", "max_heading_level"]),
            )
            .validation(
                FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(6)),
            )
            .default_static(3)
            .build(),
        AdapterOptionSpec::builder("docnav.adapters.docnav-markdown.options.payload")
            .owner("docnav-markdown")
            .operations(MAX_HEADING_LEVEL_OPERATIONS)
            .path(["options", "payload"])
            .process("cli", AdapterOptionProcessStrategy::cli_flag("--payload"))
            .process(
                "config",
                AdapterOptionProcessStrategy::json_path(["options", "payload"]),
            )
            .validation(FieldValidation::json())
            .build(),
    ]
}

fn invalid_adapter_options() -> Vec<AdapterOptionSpec> {
    vec![
        AdapterOptionSpec::builder("docnav.adapters.invalid.options.bad_path")
            .owner("docnav-invalid-options")
            .operations(MAX_HEADING_LEVEL_OPERATIONS)
            .path(["invalid", "bad_path"])
            .process(
                "config",
                AdapterOptionProcessStrategy::json_path(["options", "bad_path"]),
            )
            .validation(FieldValidation::int())
            .build(),
    ]
}

fn invalid_adapter_option_config_paths() -> Vec<AdapterOptionSpec> {
    vec![
        AdapterOptionSpec::builder("docnav.adapters.invalid.options.bad_path")
            .owner("docnav-invalid-option-config-path")
            .operations(MAX_HEADING_LEVEL_OPERATIONS)
            .path(["options", "bad_path"])
            .process(
                "config",
                AdapterOptionProcessStrategy::json_path(["invalid", "bad_path"]),
            )
            .validation(FieldValidation::int())
            .build(),
    ]
}

struct StubAdapter;

struct UnsupportedAdapter;

struct InvalidOptionAdapter;

struct InvalidOptionConfigPathAdapter;

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

    fn adapter_options(&self) -> Vec<AdapterOptionSpec> {
        stub_adapter_options()
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
        Ok(OutlineResult::structured(
            vec![Entry {
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
            None,
        ))
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

impl Adapter for UnsupportedAdapter {
    fn adapter_id(&self) -> &str {
        "docnav-unsupported"
    }

    fn manifest(&self) -> Manifest {
        Manifest {
            manifest_version: "0.1".to_owned(),
            adapter: AdapterIdentity {
                id: "docnav-unsupported".to_owned(),
                name: "Unsupported".to_owned(),
                version: "0.1.0".to_owned(),
            },
            formats: vec![FormatDescriptor {
                id: "unsupported".to_owned(),
                extensions: vec![".unsupported".to_owned()],
                content_types: vec!["application/x-unsupported".to_owned()],
            }],
        }
    }

    fn probe(&self, path: &str) -> ProbeResult {
        ProbeResult {
            probe_version: docnav_protocol::PROBE_VERSION.to_owned(),
            adapter_id: "docnav-unsupported".to_owned(),
            path: path.to_owned(),
            supported: false,
            format: None,
            confidence: 0.0,
            reasons: vec![ProbeReason {
                code: ProbeReasonCode::ContentMatch,
                detail: "unsupported test probe rejected".to_owned(),
            }],
        }
    }

    fn outline(
        &self,
        _request: &RequestEnvelope,
        _arguments: &OutlineArguments,
    ) -> AdapterResult<OutlineResult> {
        Err(AdapterError::internal("unsupported-outline-unreachable"))
    }

    fn read(
        &self,
        _request: &RequestEnvelope,
        _arguments: &ReadArguments,
    ) -> AdapterResult<ReadResult> {
        Err(AdapterError::internal("unsupported-read-unreachable"))
    }

    fn find(
        &self,
        _request: &RequestEnvelope,
        _arguments: &FindArguments,
    ) -> AdapterResult<FindResult> {
        Err(AdapterError::internal("unsupported-find-unreachable"))
    }

    fn info(
        &self,
        _request: &RequestEnvelope,
        _arguments: &InfoArguments,
    ) -> AdapterResult<InfoResult> {
        Err(AdapterError::internal("unsupported-info-unreachable"))
    }
}

impl Adapter for InvalidOptionAdapter {
    fn adapter_id(&self) -> &str {
        "docnav-invalid-options"
    }

    fn manifest(&self) -> Manifest {
        Manifest {
            manifest_version: "0.1".to_owned(),
            adapter: AdapterIdentity {
                id: "docnav-invalid-options".to_owned(),
                name: "Invalid Options".to_owned(),
                version: "0.1.0".to_owned(),
            },
            formats: vec![FormatDescriptor {
                id: "invalid-options".to_owned(),
                extensions: vec![".stub".to_owned()],
                content_types: vec!["text/stub".to_owned()],
            }],
        }
    }

    fn adapter_options(&self) -> Vec<AdapterOptionSpec> {
        invalid_adapter_options()
    }

    fn probe(&self, path: &str) -> ProbeResult {
        ProbeResult {
            probe_version: docnav_protocol::PROBE_VERSION.to_owned(),
            adapter_id: "docnav-invalid-options".to_owned(),
            path: path.to_owned(),
            supported: true,
            format: Some("invalid-options".to_owned()),
            confidence: 1.0,
            reasons: vec![ProbeReason {
                code: ProbeReasonCode::ContentMatch,
                detail: "invalid option test probe accepted".to_owned(),
            }],
        }
    }

    fn outline(
        &self,
        _request: &RequestEnvelope,
        _arguments: &OutlineArguments,
    ) -> AdapterResult<OutlineResult> {
        Err(AdapterError::internal("invalid-option-outline-unreachable"))
    }

    fn read(
        &self,
        _request: &RequestEnvelope,
        _arguments: &ReadArguments,
    ) -> AdapterResult<ReadResult> {
        Err(AdapterError::internal("invalid-option-read-unreachable"))
    }

    fn find(
        &self,
        _request: &RequestEnvelope,
        _arguments: &FindArguments,
    ) -> AdapterResult<FindResult> {
        Err(AdapterError::internal("invalid-option-find-unreachable"))
    }

    fn info(
        &self,
        _request: &RequestEnvelope,
        _arguments: &InfoArguments,
    ) -> AdapterResult<InfoResult> {
        Err(AdapterError::internal("invalid-option-info-unreachable"))
    }
}

impl Adapter for InvalidOptionConfigPathAdapter {
    fn adapter_id(&self) -> &str {
        "docnav-invalid-option-config-path"
    }

    fn manifest(&self) -> Manifest {
        Manifest {
            manifest_version: "0.1".to_owned(),
            adapter: AdapterIdentity {
                id: "docnav-invalid-option-config-path".to_owned(),
                name: "Invalid Option Config Path".to_owned(),
                version: "0.1.0".to_owned(),
            },
            formats: vec![FormatDescriptor {
                id: "invalid-option-config-path".to_owned(),
                extensions: vec![".stub".to_owned()],
                content_types: vec!["text/stub".to_owned()],
            }],
        }
    }

    fn adapter_options(&self) -> Vec<AdapterOptionSpec> {
        invalid_adapter_option_config_paths()
    }

    fn probe(&self, path: &str) -> ProbeResult {
        ProbeResult {
            probe_version: docnav_protocol::PROBE_VERSION.to_owned(),
            adapter_id: "docnav-invalid-option-config-path".to_owned(),
            path: path.to_owned(),
            supported: true,
            format: Some("invalid-option-config-path".to_owned()),
            confidence: 1.0,
            reasons: vec![ProbeReason {
                code: ProbeReasonCode::ContentMatch,
                detail: "invalid option config path test probe accepted".to_owned(),
            }],
        }
    }

    fn outline(
        &self,
        _request: &RequestEnvelope,
        _arguments: &OutlineArguments,
    ) -> AdapterResult<OutlineResult> {
        Err(AdapterError::internal(
            "invalid-option-config-path-outline-unreachable",
        ))
    }

    fn read(
        &self,
        _request: &RequestEnvelope,
        _arguments: &ReadArguments,
    ) -> AdapterResult<ReadResult> {
        Err(AdapterError::internal(
            "invalid-option-config-path-read-unreachable",
        ))
    }

    fn find(
        &self,
        _request: &RequestEnvelope,
        _arguments: &FindArguments,
    ) -> AdapterResult<FindResult> {
        Err(AdapterError::internal(
            "invalid-option-config-path-find-unreachable",
        ))
    }

    fn info(
        &self,
        _request: &RequestEnvelope,
        _arguments: &InfoArguments,
    ) -> AdapterResult<InfoResult> {
        Err(AdapterError::internal(
            "invalid-option-config-path-info-unreachable",
        ))
    }
}
