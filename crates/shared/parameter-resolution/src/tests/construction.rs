use std::collections::BTreeMap;

use docnav_typed_fields::{
    DefaultMetadata, FieldDefSetBuildError, FieldNumericRange, ProcessingId, TypedValue,
    ValidationReason, ValueKind,
};
use serde_json::json;

use super::*;

#[test]
fn processing_metadata_projection_includes_processing_path_and_schema_facts() {
    let definitions = Params::field_defs().unwrap();
    let metadata = definitions.processing_metadata(&ProcessingId::from(CONFIG_PROCESSING));
    let limit = metadata
        .iter()
        .find(|metadata| metadata.identity.as_str() == "docnav.defaults.pagination.limit")
        .unwrap();

    assert_eq!(limit.processing_id.as_str(), CONFIG_PROCESSING);
    assert_eq!(
        limit.path.segments(),
        vec!["defaults", "pagination", "limit"]
    );
    assert_eq!(limit.value_kind, ValueKind::Integer);
    let FieldNumericRange::Integer(range) = limit.constraints.numeric_range else {
        panic!("expected integer range");
    };
    assert_eq!(range.minimum.unwrap().value, 1);
    assert_eq!(limit.default, DefaultMetadata::Static(json!(20_000)));
}

#[test]
fn source_construction_maps_direct_config_and_default_values() {
    let catalog = parameter_catalog();
    let entries = catalog.entries();
    let identity = identity("docnav.defaults.pagination.limit");
    let mut dynamic_defaults = BTreeMap::new();
    dynamic_defaults.insert(identity.clone(), json!(400));
    let direct_input = json!({
        "limit": 100,
        "output": "readable-json",
        "native_options": {"theme": "direct"}
    });
    let project_config = json!({
        "defaults": {"pagination": {"limit": 200}, "output": "readable-view"},
        "native_options": {"theme": "project"}
    });
    let user_config = json!({
        "defaults": {"pagination": {"limit": 300}, "output": "readable-view"}
    });

    let resolution = resolve_parameters(
        entries,
        ParameterSources {
            direct_input: construct_direct_input_source(entries, Some(&direct_input)),
            project_config: construct_config_source(entries, Some(&project_config)),
            user_config: construct_config_source(entries, Some(&user_config)),
            default: construct_default_source(entries, &dynamic_defaults),
        },
        EntryPassthroughPolicy::Retain,
    );

    let resolved = resolution.value(&identity).unwrap();
    assert_eq!(resolved.value, TypedValue::Integer(100));
    assert_eq!(
        resolved.source,
        ParameterSourceInfo::new(ParameterSourceKind::DirectInput)
    );
    assert!(resolution.diagnostics().is_empty());
}

#[derive(Debug, FieldDefs)]
struct ConflictingConfigPathParams {
    #[field(
        FieldDef::builder("docnav.defaults.left")
            .process(DIRECT_PROCESSING, config_json_path(["left"]))
            .process(CONFIG_PROCESSING, config_json_path(["defaults", "shared"]))
            .validation(FieldValidation::int().between(
                FieldBound::closed(1),
                FieldBound::closed(100_000),
            ))
    )]
    left: Option<i64>,

    #[field(
        FieldDef::builder("docnav.defaults.right")
            .process(DIRECT_PROCESSING, config_json_path(["right"]))
            .process(CONFIG_PROCESSING, config_json_path(["defaults", "shared"]))
            .validation(FieldValidation::int().between(
                FieldBound::closed(1),
                FieldBound::closed(100_000),
            ))
    )]
    right: Option<i64>,
}

#[test]
fn catalog_derivation_conflicting_config_paths_are_rejected_by_field_set_build() {
    let error = ConflictingConfigPathParams::field_defs()
        .expect_err("typed-field set rejects duplicate config path");

    assert!(matches!(
        error,
        FieldDefSetBuildError::DuplicateProcessingPath(ref error)
            if error.processing_id.as_str() == CONFIG_PROCESSING
                && error.path.segments() == vec!["defaults", "shared"]
                && error.previous_identity.as_str() == "docnav.defaults.left"
                && error.current_identity.as_str() == "docnav.defaults.right"
    ));
}

#[test]
fn constructed_sources_report_validation_failures_through_diagnostic_handoff() {
    let catalog = parameter_catalog();
    let entries = catalog.entries();
    let identity = identity("docnav.defaults.pagination.limit");
    let direct_input = json!({"limit": 0});
    let project_config = json!({"defaults": {"pagination": {"limit": 200}}});

    let resolution = resolve_parameters(
        entries,
        ParameterSources {
            direct_input: construct_direct_input_source(entries, Some(&direct_input)),
            project_config: construct_config_source(entries, Some(&project_config)),
            ..ParameterSources::default()
        },
        EntryPassthroughPolicy::Retain,
    );

    assert!(resolution.value(&identity).is_none());
    let diagnostic = resolution.diagnostics()[0].as_validation().unwrap();
    assert_eq!(diagnostic.identity, identity);
    assert_eq!(
        diagnostic.source,
        Some(ParameterSourceInfo::new(ParameterSourceKind::DirectInput))
    );
    assert!(matches!(
        diagnostic.failure.reason,
        ValidationReason::BelowMinimum { .. }
    ));
}

#[test]
fn explicit_config_source_failure_returns_issue_and_continues_resolution() {
    let fields = Params::field_defs().unwrap();
    let missing = temp_path("missing-project-config.json");
    let user_config = temp_file(
        "source-skip-user-config.json",
        r#"{"defaults": {"pagination": {"limit": 300}, "output": "readable-view"}}"#,
    );
    let loaded_project = load_parameter_config_source(&ParameterConfigSourceDescriptor::new(
        ConfigSourceLevel::Project,
        ConfigPathOrigin::ExplicitCli,
        missing.clone(),
    ));
    let loaded_user = load_parameter_config_source(&ParameterConfigSourceDescriptor::new(
        ConfigSourceLevel::User,
        ConfigPathOrigin::ExplicitCli,
        user_config,
    ));

    assert_eq!(loaded_project.diagnostics().len(), 1);
    let issue = loaded_project.diagnostics()[0].as_config_source().unwrap();
    assert_config_source_issue(
        issue,
        "project",
        "explicit_cli",
        &missing,
        "missing_explicit_cli",
    );

    let resolution = ParameterResolutionPipeline::new(&fields)
        .with_direct_input_processing_id(DIRECT_PROCESSING)
        .with_config_processing_id(CONFIG_PROCESSING)
        .with_loaded_project_config(loaded_project)
        .with_loaded_user_config(loaded_user)
        .resolve(None::<JsonValue>)
        .unwrap();

    assert_eq!(
        resolution
            .value(&identity("docnav.defaults.pagination.limit"))
            .unwrap()
            .value,
        TypedValue::Integer(300)
    );
    assert_eq!(resolution.diagnostics().len(), 1);
    assert!(resolution.diagnostics()[0].as_config_source().is_some());
}

#[test]
fn default_missing_config_source_has_no_diagnostic_event() {
    let loaded = load_parameter_config_source(&ParameterConfigSourceDescriptor::new(
        ConfigSourceLevel::User,
        ConfigPathOrigin::Default,
        temp_path("missing-user-config.json"),
    ));

    assert!(loaded.value().is_none());
    assert!(loaded.diagnostics().is_empty());
}

#[test]
fn override_missing_config_source_uses_override_diagnostic_event() {
    let missing = temp_path("missing-override-config.json");

    let loaded = load_parameter_config_source(&ParameterConfigSourceDescriptor::new(
        ConfigSourceLevel::Project,
        ConfigPathOrigin::Override,
        missing.clone(),
    ));

    assert_eq!(loaded.diagnostics().len(), 1);
    let issue = loaded.diagnostics()[0].as_config_source().unwrap();
    assert_config_source_issue(issue, "project", "override", &missing, "missing_override");
}

#[test]
fn invalid_config_sources_are_reported_as_config_source_issues() {
    let invalid_json = temp_file("invalid-config.json", "{invalid");
    let non_object = temp_file("non-object-config.json", "[]");
    let not_file = temp_dir("not-file-config.json");

    let invalid = load_parameter_config_source(&ParameterConfigSourceDescriptor::new(
        ConfigSourceLevel::Project,
        ConfigPathOrigin::ExplicitCli,
        invalid_json,
    ));
    let non_object = load_parameter_config_source(&ParameterConfigSourceDescriptor::new(
        ConfigSourceLevel::User,
        ConfigPathOrigin::ExplicitCli,
        non_object,
    ));
    let not_file = load_parameter_config_source(&ParameterConfigSourceDescriptor::new(
        ConfigSourceLevel::Project,
        ConfigPathOrigin::ExplicitCli,
        not_file,
    ));

    assert_config_source_reason(&invalid, "invalid_json");
    assert_config_source_reason(&non_object, "non_object");
    assert_config_source_reason(&not_file, "not_file");
}

#[test]
fn unreadable_config_source_is_reported_as_config_source_issue() {
    let Some((path, _guard)) = unreadable_file("unreadable-config.json") else {
        return;
    };

    let loaded = load_parameter_config_source(&ParameterConfigSourceDescriptor::new(
        ConfigSourceLevel::Project,
        ConfigPathOrigin::ExplicitCli,
        path,
    ));

    assert_config_source_reason(&loaded, "unreadable");
}

#[test]
fn constructed_passthrough_values_keep_source_processing_results_without_validation() {
    let catalog = parameter_catalog();
    let entries = catalog.entries();
    let direct_input = json!({"native": {"shared": 1}, "limit": 100});
    let direct_passthrough = json!({"native": {"shared": 1}});
    let project_config = json!({
        "defaults": {"output": "readable-view"},
        "native": {"shared": 2, "project": true}
    });
    let project_passthrough = json!({
        "native": {"shared": 2, "project": true}
    });
    let user_config = json!({
        "native": {"shared": 3, "user": true}
    });

    let resolution = resolve_parameters(
        entries,
        ParameterSources {
            direct_input: construct_direct_input_source_with_passthrough(
                entries,
                Some(&direct_input),
                Some(&direct_passthrough),
            ),
            project_config: construct_config_source_with_passthrough(
                entries,
                Some(&project_config),
                Some(&project_passthrough),
            ),
            user_config: construct_config_source(entries, Some(&user_config)),
            ..ParameterSources::default()
        },
        EntryPassthroughPolicy::Delegate,
    );

    assert!(resolution.diagnostics().is_empty());
    assert_eq!(resolution.passthrough().len(), 3);
    let native = passthrough_from(&resolution, ParameterSourceKind::DirectInput);
    assert_eq!(native.value, direct_passthrough);
    assert_eq!(
        native.source,
        ParameterSourceInfo::new(ParameterSourceKind::DirectInput)
    );
    assert_eq!(native.disposition, PassthroughDisposition::Delegated);
    let project = passthrough_from(&resolution, ParameterSourceKind::ProjectConfig);
    assert_eq!(project.value, project_passthrough);
    let user = passthrough_from(&resolution, ParameterSourceKind::UserConfig);
    assert_eq!(user.value, user_config);
}

#[test]
fn empty_object_passthrough_is_preserved_for_entry_owner() {
    let catalog = parameter_catalog();
    let entries = catalog.entries();
    let direct_input = json!({"output": "readable-view", "native": {}});
    let direct_passthrough = json!({"native": {}});

    let resolution = resolve_parameters(
        entries,
        ParameterSources {
            direct_input: construct_direct_input_source_with_passthrough(
                entries,
                Some(&direct_input),
                Some(&direct_passthrough),
            ),
            ..ParameterSources::default()
        },
        EntryPassthroughPolicy::Delegate,
    );

    assert!(resolution.diagnostics().is_empty());
    assert_eq!(resolution.passthrough().len(), 1);
    let passthrough = &resolution.passthrough()[0];
    assert_eq!(passthrough.value, direct_passthrough);
    assert_eq!(
        passthrough.source,
        ParameterSourceInfo::new(ParameterSourceKind::DirectInput)
    );
    assert_eq!(passthrough.disposition, PassthroughDisposition::Delegated);
}

#[test]
fn nested_processed_passthrough_preserves_raw_structure() {
    let catalog = parameter_catalog();
    let entries = catalog.entries();
    let project_config = json!({
        "defaults": {
            "output": "readable-view",
            "native": {
                "theme": "dark",
                "strict": true
            }
        }
    });
    let project_passthrough = json!({
        "defaults": {
            "native": {
                "theme": "dark",
                "strict": true
            }
        }
    });

    let resolution = resolve_parameters(
        entries,
        ParameterSources {
            project_config: construct_config_source_with_passthrough(
                entries,
                Some(&project_config),
                Some(&project_passthrough),
            ),
            ..ParameterSources::default()
        },
        EntryPassthroughPolicy::Delegate,
    );

    assert!(resolution.diagnostics().is_empty());
    let output = resolution
        .value(&identity("docnav.defaults.output"))
        .unwrap();
    assert_eq!(output.value, TypedValue::String("readable-view".to_owned()));
    let passthrough = passthrough_from(&resolution, ParameterSourceKind::ProjectConfig);
    assert_eq!(passthrough.value, project_passthrough);
    assert_eq!(
        passthrough.source,
        ParameterSourceInfo::new(ParameterSourceKind::ProjectConfig)
    );
    assert_eq!(passthrough.disposition, PassthroughDisposition::Delegated);
}

fn assert_config_source_reason(loaded: &LoadedParameterConfigSource, reason_code: &str) {
    let issue = loaded.diagnostics()[0].as_config_source().unwrap();
    assert_eq!(issue.reason_code, reason_code);
}

fn assert_config_source_issue(
    issue: &ParameterConfigSourceIssue,
    source_level: &str,
    path_origin: &str,
    path: &std::path::Path,
    reason_code: &str,
) {
    assert_eq!(issue.source_level, source_level);
    assert_eq!(issue.path_origin, path_origin);
    assert_eq!(issue.path, path.display().to_string());
    assert_eq!(issue.reason_code, reason_code);
}
