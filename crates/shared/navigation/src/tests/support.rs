use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use cli_config_resolution::{
    FieldIdentity, Source, SourceCandidate, SourceId, SourceKind, SourceLocator,
};
use docnav_adapter_contracts::{
    Adapter, AdapterDefinition, AdapterError, AdapterResult, FindInput, InfoInput, OutlineInput,
    ReadInput, StandardInputBinding, UnstructuredFullReadCapabilities,
};
use docnav_protocol::{
    AdapterIdentity, Entry, FindResult, FormatDescriptor, InfoResult, Manifest, Operation,
    OutlineResult, PagedOperation, ProbeReason, ProbeReasonCode, ProbeResult, ReadResult,
};
use docnav_typed_fields::{
    ExpectedFieldShape, FieldBound, FieldDef, FieldDefSet, FieldDefSetBuilder, FieldValidation,
    MergeStrategy, ProcessStrategy,
};
use serde_json::Value;

use crate::{
    config_source::LoadedNavigationConfigSource, AutoReadMode, DocumentParameterBinding,
    DocumentParameterCatalog, DocumentParameterEntry, NavigationAdapterRegistry, NavigationCommand,
    NavigationConfigSource, NavigationConfigSourceLevel, NavigationConfigSourceOrigin,
    NavigationConfigSources, NavigationOutputMode,
};

const PAGE_IDENTITY: &str = "docnav.document.page";
const LIMIT_IDENTITY: &str = "docnav.defaults.pagination.limit";
const PAGINATION_ENABLED_IDENTITY: &str = "docnav.defaults.pagination.enabled";
const OUTPUT_IDENTITY: &str = "docnav.defaults.output";
const AUTO_READ_IDENTITY: &str = "docnav.defaults.auto_read";
const MARKDOWN_MAX_HEADING_LEVEL_IDENTITY: &str =
    "docnav.adapters.docnav-markdown.options.max_heading_level";
const OTHER_MAX_HEADING_LEVEL_IDENTITY: &str =
    "docnav.adapters.docnav-other.options.max_heading_level";

pub(super) fn navigation_command(candidates: Vec<SourceCandidate>) -> NavigationCommand {
    NavigationCommand {
        operation: Operation::Outline,
        document_path: "docs/guide.stub".to_owned(),
        ref_id: None,
        query: None,
        cli_source: Source::new(
            SourceId::new("explicit").unwrap(),
            SourceKind::Cli,
            400,
            candidates,
        )
        .unwrap(),
    }
}

pub(super) fn cli_value_candidate(identity: &str, flag: &str, value: Value) -> SourceCandidate {
    SourceCandidate::value(
        FieldIdentity::new(identity).unwrap(),
        SourceLocator::CliFlag(flag.to_owned()),
        value,
    )
}

pub(super) fn cli_invalid_candidate(
    identity: &str,
    flag: &str,
    raw: Value,
    reason: &str,
) -> SourceCandidate {
    SourceCandidate::invalid(
        FieldIdentity::new(identity).unwrap(),
        SourceLocator::CliFlag(flag.to_owned()),
        raw,
        reason,
    )
}

pub(super) fn config_sources(project: Value, user: Value) -> NavigationConfigSources {
    NavigationConfigSources {
        project: NavigationConfigSource {
            level: NavigationConfigSourceLevel::Project,
            origin: NavigationConfigSourceOrigin::Default,
            path: "project/.docnav/docnav.json".to_owned(),
            loaded: LoadedNavigationConfigSource::from_value(project),
        },
        user: NavigationConfigSource {
            level: NavigationConfigSourceLevel::User,
            origin: NavigationConfigSourceOrigin::Default,
            path: "user/docnav.json".to_owned(),
            loaded: LoadedNavigationConfigSource::from_value(user),
        },
    }
}

pub(super) fn document_parameter_catalog() -> DocumentParameterCatalog {
    DocumentParameterCatalog::new(
        ["docnav-markdown", "docnav-other", "docnav-unsupported"],
        document_parameter_fields(),
        document_parameter_entries(),
    )
    .expect("test document parameter catalog must be valid")
}

fn document_parameter_fields() -> FieldDefSetBuilder {
    FieldDefSet::builder()
        .field(
            FieldDef::builder(PAGE_IDENTITY)
                .process("cli", ProcessStrategy::cli_flag("--page"))
                .validation(FieldValidation::int().between(
                    FieldBound::closed(1),
                    FieldBound::closed(i64::from(u32::MAX)),
                ))
                .default_static(1)
                .merge(MergeStrategy::Replace),
            ExpectedFieldShape::required(),
        )
        .field(
            FieldDef::builder(LIMIT_IDENTITY)
                .process("cli", ProcessStrategy::cli_flag("--limit"))
                .process(
                    "config",
                    ProcessStrategy::config_path(["defaults", "pagination", "limit"]),
                )
                .validation(FieldValidation::int().between(
                    FieldBound::closed(1),
                    FieldBound::closed(i64::from(u32::MAX)),
                ))
                .default_static(6000)
                .merge(MergeStrategy::Replace),
            ExpectedFieldShape::required(),
        )
        .field(
            FieldDef::builder(PAGINATION_ENABLED_IDENTITY)
                .process("cli", ProcessStrategy::cli_flag("--pagination"))
                .process(
                    "config",
                    ProcessStrategy::config_path(["defaults", "pagination", "enabled"]),
                )
                .validation(FieldValidation::boolean())
                .default_static(true)
                .merge(MergeStrategy::Replace),
            ExpectedFieldShape::required(),
        )
        .field(
            FieldDef::builder(OUTPUT_IDENTITY)
                .process("cli", ProcessStrategy::cli_flag("--output"))
                .process(
                    "config",
                    ProcessStrategy::config_path(["defaults", "output"]),
                )
                .validation(FieldValidation::string_enum::<NavigationOutputMode>())
                .default_static(NavigationOutputMode::ReadableView)
                .merge(MergeStrategy::Replace),
            ExpectedFieldShape::required(),
        )
        .field(
            FieldDef::builder(AUTO_READ_IDENTITY)
                .process("cli", ProcessStrategy::cli_flag("--auto-read"))
                .process(
                    "config",
                    ProcessStrategy::config_path(["defaults", "auto_read"]),
                )
                .validation(FieldValidation::string_enum::<AutoReadMode>())
                .default_static(AutoReadMode::UniqueRef)
                .merge(MergeStrategy::Replace),
            ExpectedFieldShape::required(),
        )
        .field(
            FieldDef::builder(MARKDOWN_MAX_HEADING_LEVEL_IDENTITY)
                .process("cli", ProcessStrategy::cli_flag("--max-heading-level"))
                .process(
                    "config",
                    ProcessStrategy::config_path([
                        "options",
                        "docnav-markdown",
                        "max_heading_level",
                    ]),
                )
                .validation(
                    FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(6)),
                )
                .default_static(3)
                .merge(MergeStrategy::Replace),
            ExpectedFieldShape::optional(),
        )
        .field(
            FieldDef::builder(OTHER_MAX_HEADING_LEVEL_IDENTITY)
                .process(
                    "config",
                    ProcessStrategy::config_path(["options", "docnav-other", "max_heading_level"]),
                )
                .validation(
                    FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(6)),
                )
                .merge(MergeStrategy::Replace),
            ExpectedFieldShape::optional(),
        )
}

fn document_parameter_entries() -> Vec<DocumentParameterEntry> {
    vec![
        DocumentParameterEntry::new(
            FieldIdentity::new(PAGE_IDENTITY).expect("test field identity must be valid"),
            None,
            vec![
                DocumentParameterBinding::StandardInput(StandardInputBinding::OutlinePage),
                DocumentParameterBinding::StandardInput(StandardInputBinding::ReadPage),
                DocumentParameterBinding::StandardInput(StandardInputBinding::FindPage),
            ],
        ),
        DocumentParameterEntry::new(
            FieldIdentity::new(LIMIT_IDENTITY).expect("test field identity must be valid"),
            None,
            vec![
                DocumentParameterBinding::StandardInput(StandardInputBinding::OutlineLimit),
                DocumentParameterBinding::StandardInput(StandardInputBinding::ReadLimit),
                DocumentParameterBinding::StandardInput(StandardInputBinding::FindLimit),
            ],
        ),
        DocumentParameterEntry::new(
            FieldIdentity::new(PAGINATION_ENABLED_IDENTITY)
                .expect("test field identity must be valid"),
            None,
            vec![
                DocumentParameterBinding::PaginationEnabled(PagedOperation::Outline),
                DocumentParameterBinding::PaginationEnabled(PagedOperation::Read),
                DocumentParameterBinding::PaginationEnabled(PagedOperation::Find),
            ],
        ),
        DocumentParameterEntry::new(
            FieldIdentity::new(OUTPUT_IDENTITY).expect("test field identity must be valid"),
            None,
            vec![
                DocumentParameterBinding::OutputMode(Operation::Outline),
                DocumentParameterBinding::OutputMode(Operation::Read),
                DocumentParameterBinding::OutputMode(Operation::Find),
                DocumentParameterBinding::OutputMode(Operation::Info),
            ],
        ),
        DocumentParameterEntry::new(
            FieldIdentity::new(AUTO_READ_IDENTITY).expect("test field identity must be valid"),
            None,
            vec![
                DocumentParameterBinding::AutoReadMode(Operation::Outline),
                DocumentParameterBinding::AutoReadMode(Operation::Find),
            ],
        ),
        DocumentParameterEntry::new(
            FieldIdentity::new(MARKDOWN_MAX_HEADING_LEVEL_IDENTITY)
                .expect("test field identity must be valid"),
            Some("docnav-markdown".to_owned()),
            vec![
                DocumentParameterBinding::StandardInput(
                    StandardInputBinding::OutlineMaxHeadingLevel,
                ),
                DocumentParameterBinding::StandardInput(StandardInputBinding::FindMaxHeadingLevel),
            ],
        ),
        DocumentParameterEntry::new(
            FieldIdentity::new(OTHER_MAX_HEADING_LEVEL_IDENTITY)
                .expect("test field identity must be valid"),
            Some("docnav-other".to_owned()),
            vec![
                DocumentParameterBinding::StandardInput(
                    StandardInputBinding::OutlineMaxHeadingLevel,
                ),
                DocumentParameterBinding::StandardInput(StandardInputBinding::FindMaxHeadingLevel),
            ],
        ),
    ]
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
    fn adapters(&self) -> Vec<AdapterDefinition<'_>> {
        vec![adapter_definition(&StubAdapter, stub_manifest(), None)]
    }
}

pub(super) struct MultiAdapterRegistry;

impl NavigationAdapterRegistry for MultiAdapterRegistry {
    fn adapters(&self) -> Vec<AdapterDefinition<'_>> {
        vec![
            adapter_definition(&StubAdapter, stub_manifest(), None),
            adapter_definition(&OtherAdapter, other_manifest(), None),
        ]
    }
}

pub(super) struct UnsupportedRegistry;

impl NavigationAdapterRegistry for UnsupportedRegistry {
    fn adapters(&self) -> Vec<AdapterDefinition<'_>> {
        vec![adapter_definition(
            &UnsupportedAdapter,
            unsupported_manifest(),
            None,
        )]
    }
}

fn adapter_definition(
    adapter: &dyn Adapter,
    manifest: Manifest,
    capabilities: Option<UnstructuredFullReadCapabilities>,
) -> AdapterDefinition<'_> {
    AdapterDefinition::new(manifest, adapter, capabilities).expect("valid test adapter definition")
}

struct StubAdapter;

struct OtherAdapter;

struct UnsupportedAdapter;

fn stub_manifest() -> Manifest {
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

impl Adapter for StubAdapter {
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

    fn outline(&self, input: &OutlineInput) -> AdapterResult<OutlineResult> {
        let label = input
            .max_heading_level
            .map(|value| format!("Max {value}"))
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

    fn read(&self, _input: &ReadInput) -> AdapterResult<ReadResult> {
        Err(AdapterError::ref_not_found("missing"))
    }

    fn find(&self, _input: &FindInput) -> AdapterResult<FindResult> {
        Err(AdapterError::invalid_request(
            "arguments.query",
            "query is not indexed",
        ))
    }

    fn info(&self, _input: &InfoInput) -> AdapterResult<InfoResult> {
        Err(AdapterError::internal("stub-info-unimplemented"))
    }
}

fn other_manifest() -> Manifest {
    Manifest {
        manifest_version: "0.1".to_owned(),
        adapter: AdapterIdentity {
            id: "docnav-other".to_owned(),
            name: "Other".to_owned(),
            version: "0.1.0".to_owned(),
        },
        formats: vec![FormatDescriptor {
            id: "other".to_owned(),
            extensions: vec![".other".to_owned()],
            content_types: vec!["text/other".to_owned()],
        }],
    }
}

impl Adapter for OtherAdapter {
    fn probe(&self, path: &str) -> ProbeResult {
        ProbeResult {
            probe_version: docnav_protocol::PROBE_VERSION.to_owned(),
            adapter_id: "docnav-other".to_owned(),
            path: path.to_owned(),
            supported: false,
            format: None,
            confidence: 0.0,
            reasons: vec![ProbeReason {
                code: ProbeReasonCode::ContentMatch,
                detail: "other test probe rejected".to_owned(),
            }],
        }
    }

    fn outline(&self, _input: &OutlineInput) -> AdapterResult<OutlineResult> {
        Err(AdapterError::internal("other-outline-unreachable"))
    }

    fn read(&self, _input: &ReadInput) -> AdapterResult<ReadResult> {
        Err(AdapterError::internal("other-read-unreachable"))
    }

    fn find(&self, _input: &FindInput) -> AdapterResult<FindResult> {
        Err(AdapterError::internal("other-find-unreachable"))
    }

    fn info(&self, _input: &InfoInput) -> AdapterResult<InfoResult> {
        Err(AdapterError::internal("other-info-unreachable"))
    }
}

fn unsupported_manifest() -> Manifest {
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

impl Adapter for UnsupportedAdapter {
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

    fn outline(&self, _input: &OutlineInput) -> AdapterResult<OutlineResult> {
        Err(AdapterError::internal("unsupported-outline-unreachable"))
    }

    fn read(&self, _input: &ReadInput) -> AdapterResult<ReadResult> {
        Err(AdapterError::internal("unsupported-read-unreachable"))
    }

    fn find(&self, _input: &FindInput) -> AdapterResult<FindResult> {
        Err(AdapterError::internal("unsupported-find-unreachable"))
    }

    fn info(&self, _input: &InfoInput) -> AdapterResult<InfoResult> {
        Err(AdapterError::internal("unsupported-info-unreachable"))
    }
}
