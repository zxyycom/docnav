use cli_config_resolution::{
    FieldIdentity, Source, SourceCandidate, SourceId, SourceKind, SourceLocator,
};
use docnav_protocol::Operation;
use docnav_typed_fields::{
    ExpectedFieldShape, FieldDef, FieldDefSet, FieldValidation, MergeStrategy, ProcessStrategy,
};
use serde_json::{json, Value};

use super::{
    fields, resolve_command_with_fields, AutoReadMode, DocumentParameterBinding,
    DocumentParameterCatalog, DocumentParameterEntry,
};
use crate::{
    config_source::LoadedNavigationConfigSource, NavigationCommand, NavigationConfigSource,
    NavigationConfigSourceLevel, NavigationConfigSourceOrigin, NavigationConfigSources,
    DOCUMENT_CLI_SOURCE_ID, DOCUMENT_CLI_SOURCE_PRIORITY,
};

const AUTO_READ_IDENTITY: &str = "docnav.defaults.auto_read";
const SELECTED_ADAPTER_ID: &str = "test-adapter";

#[test]
fn auto_read_replace_trace_keeps_selected_overridden_and_builtin_provenance() {
    let catalog = auto_read_catalog();
    let fields = fields::operation_fields(Operation::Outline, SELECTED_ADAPTER_ID, &catalog)
        .expect("operation fields");

    let cli_overrides_config = resolve_command_with_fields(
        fields.as_ref(),
        &command(Some("disabled")),
        &config_sources(
            json!({"defaults": {"auto_read": "unique-ref"}}),
            json!({"defaults": {"auto_read": "disabled"}}),
        ),
    )
    .expect("CLI precedence resolution");
    assert_trace(
        &cli_overrides_config,
        "explicit",
        &["project", "user"],
        "unique-ref",
    );

    let project_overrides_user = resolve_command_with_fields(
        fields.as_ref(),
        &command(None),
        &config_sources(
            json!({"defaults": {"auto_read": "unique-ref"}}),
            json!({"defaults": {"auto_read": "disabled"}}),
        ),
    )
    .expect("project precedence resolution");
    assert_trace(&project_overrides_user, "project", &["user"], "unique-ref");

    let user_overrides_builtin = resolve_command_with_fields(
        fields.as_ref(),
        &command(None),
        &config_sources(json!({}), json!({"defaults": {"auto_read": "disabled"}})),
    )
    .expect("user precedence resolution");
    assert_trace(&user_overrides_builtin, "user", &[], "unique-ref");
}

fn assert_trace(
    resolution: &cli_config_resolution::ResolutionResult,
    selected: &str,
    overridden: &[&str],
    fallback: &str,
) {
    let identity = FieldIdentity::new(AUTO_READ_IDENTITY).expect("valid identity");
    let trace = resolution.trace(&identity).expect("auto-read trace");
    assert_eq!(
        trace
            .selected
            .as_ref()
            .map(|candidate| candidate.source_id.as_str()),
        Some(selected)
    );
    assert_eq!(
        trace
            .overridden
            .iter()
            .map(|candidate| candidate.source_id.as_str())
            .collect::<Vec<_>>(),
        overridden
    );
    assert_eq!(
        trace
            .default_fallback
            .as_ref()
            .map(|candidate| &candidate.raw),
        Some(&json!(fallback))
    );
}

fn auto_read_catalog() -> DocumentParameterCatalog {
    let fields = FieldDefSet::builder().field(
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
    );
    let entry = DocumentParameterEntry::new(
        FieldIdentity::new(AUTO_READ_IDENTITY).expect("valid identity"),
        None,
        vec![
            DocumentParameterBinding::AutoReadMode(Operation::Outline),
            DocumentParameterBinding::AutoReadMode(Operation::Find),
        ],
    );
    DocumentParameterCatalog::new([SELECTED_ADAPTER_ID], fields, vec![entry])
        .expect("valid test catalog")
}

fn command(auto_read: Option<&str>) -> NavigationCommand {
    let candidates = auto_read
        .map(|value| {
            vec![SourceCandidate::value(
                FieldIdentity::new(AUTO_READ_IDENTITY).expect("valid identity"),
                SourceLocator::CliFlag("--auto-read".to_owned()),
                json!(value),
            )]
        })
        .unwrap_or_default();
    NavigationCommand {
        operation: Operation::Outline,
        document_path: "docs/guide.md".to_owned(),
        ref_id: None,
        query: None,
        cli_source: Source::new(
            SourceId::new(DOCUMENT_CLI_SOURCE_ID).expect("valid source id"),
            SourceKind::Cli,
            DOCUMENT_CLI_SOURCE_PRIORITY,
            candidates,
        )
        .expect("valid CLI source"),
    }
}

fn config_sources(project: Value, user: Value) -> NavigationConfigSources {
    NavigationConfigSources {
        project: config_source(NavigationConfigSourceLevel::Project, project),
        user: config_source(NavigationConfigSourceLevel::User, user),
    }
}

fn config_source(level: NavigationConfigSourceLevel, value: Value) -> NavigationConfigSource {
    NavigationConfigSource {
        level,
        origin: NavigationConfigSourceOrigin::Default,
        path: format!("{}.json", level.as_str()),
        loaded: LoadedNavigationConfigSource::from_value(value),
    }
}
