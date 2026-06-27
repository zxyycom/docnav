use std::collections::BTreeMap;

use docnav_diagnostics::{DiagnosticDetails, DiagnosticEffect};
use docnav_typed_fields::{
    DefaultMetadata, FieldNumericRange, ProcessingId, TypedValue, ValidationReason, ValueKind,
};
use serde_json::json;

use super::*;

#[test]
fn processing_metadata_projection_includes_processing_path_and_schema_facts() {
    let definitions = Params::field_defs().unwrap();
    let metadata = definitions.processing_metadata(&ProcessingId::from(CONFIG_PROCESSING));
    let limit = metadata
        .iter()
        .find(|metadata| metadata.identity.as_str() == "docnav.defaults.limit_chars")
        .unwrap();

    assert_eq!(limit.processing_id.as_str(), CONFIG_PROCESSING);
    assert_eq!(limit.path.segments(), vec!["defaults", "limit_chars"]);
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
    let identity = identity("docnav.defaults.limit_chars");
    let mut dynamic_defaults = BTreeMap::new();
    dynamic_defaults.insert(identity.clone(), json!(400));
    let direct_input = json!({
        "limit_chars": 100,
        "output": "readable-json",
        "native_options": {"theme": "direct"}
    });
    let project_config = json!({
        "defaults": {"limit_chars": 200, "output": "readable-view"},
        "native_options": {"theme": "project"}
    });
    let user_config = json!({
        "defaults": {"limit_chars": 300, "output": "readable-view"}
    });

    let resolution = resolve_standard_parameters(
        entries,
        StandardParameterSources {
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
        StandardParameterSourceInfo::new(StandardParameterSourceKind::DirectInput)
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
fn catalog_derivation_rejects_conflicting_config_paths() {
    let definitions = ConflictingConfigPathParams::field_defs().unwrap();

    let error = derive_standard_parameter_catalog(
        &definitions,
        &ProcessingId::from(DIRECT_PROCESSING),
        &ProcessingId::from(CONFIG_PROCESSING),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        StandardParameterCatalogError::Conflict {
            kind: StandardParameterCatalogConflictKind::ConfigPath,
            path: ref conflict_path,
            ..
        } if conflict_path.as_ref() == Some(&path(["defaults", "shared"]))
    ));
}

#[test]
fn constructed_sources_report_validation_failures_through_diagnostic_handoff() {
    let catalog = parameter_catalog();
    let entries = catalog.entries();
    let identity = identity("docnav.defaults.limit_chars");
    let direct_input = json!({"limit_chars": 0});
    let project_config = json!({"defaults": {"limit_chars": 200}});

    let resolution = resolve_standard_parameters(
        entries,
        StandardParameterSources {
            direct_input: construct_direct_input_source(entries, Some(&direct_input)),
            project_config: construct_config_source(entries, Some(&project_config)),
            ..StandardParameterSources::default()
        },
        EntryPassthroughPolicy::Retain,
    );

    assert!(resolution.value(&identity).is_none());
    let diagnostic = resolution.diagnostics()[0].as_validation().unwrap();
    assert_eq!(diagnostic.identity, identity);
    assert_eq!(
        diagnostic.source,
        Some(StandardParameterSourceInfo::new(
            StandardParameterSourceKind::DirectInput
        ))
    );
    assert!(matches!(
        diagnostic.failure.reason,
        ValidationReason::BelowMinimum { .. }
    ));
}

#[test]
fn explicit_config_source_skip_returns_warning_event_and_continues_resolution() {
    let fields = Params::field_defs().unwrap();
    let missing = temp_path("missing-project-config.json");
    let user_config = temp_file(
        "source-skip-user-config.json",
        r#"{"defaults": {"limit_chars": 300, "output": "readable-view"}}"#,
    );
    let loaded_project =
        load_standard_parameter_config_source(&StandardParameterConfigSourceDescriptor::new(
            ConfigSourceLevel::Project,
            ConfigPathOrigin::Override,
            missing.clone(),
        ));
    let loaded_user =
        load_standard_parameter_config_source(&StandardParameterConfigSourceDescriptor::new(
            ConfigSourceLevel::User,
            ConfigPathOrigin::Override,
            user_config,
        ));

    assert_eq!(loaded_project.diagnostics().len(), 1);
    let warning = loaded_project.diagnostics()[0].as_warning().unwrap();
    assert_eq!(warning.effect(), DiagnosticEffect::OperationContinued);
    assert_eq!(
        warning.details(),
        &DiagnosticDetails::AdapterConfigSource {
            source_level: "project".to_owned(),
            path_origin: "override".to_owned(),
            path: missing.display().to_string(),
            reason_code: "missing_override".to_owned(),
        }
    );

    let resolution = StandardParameterPipeline::new(&fields)
        .with_direct_input_processing_id(DIRECT_PROCESSING)
        .with_config_processing_id(CONFIG_PROCESSING)
        .with_loaded_project_config(loaded_project)
        .with_loaded_user_config(loaded_user)
        .resolve(None::<JsonValue>)
        .unwrap();

    assert_eq!(
        resolution
            .value(&identity("docnav.defaults.limit_chars"))
            .unwrap()
            .value,
        TypedValue::Integer(300)
    );
    assert_eq!(resolution.diagnostics().len(), 1);
    assert!(resolution.diagnostics()[0].as_warning().is_some());
}

#[test]
fn default_missing_config_source_has_no_diagnostic_event() {
    let loaded =
        load_standard_parameter_config_source(&StandardParameterConfigSourceDescriptor::new(
            ConfigSourceLevel::User,
            ConfigPathOrigin::Default,
            temp_path("missing-user-config.json"),
        ));

    assert!(loaded.value().is_none());
    assert!(loaded.diagnostics().is_empty());
}

#[test]
fn invalid_config_sources_are_skipped_with_warning_events() {
    let invalid_json = temp_file("invalid-config.json", "{invalid");
    let non_object = temp_file("non-object-config.json", "[]");
    let not_file = temp_dir("not-file-config.json");

    let invalid =
        load_standard_parameter_config_source(&StandardParameterConfigSourceDescriptor::new(
            ConfigSourceLevel::Project,
            ConfigPathOrigin::Override,
            invalid_json,
        ));
    let non_object =
        load_standard_parameter_config_source(&StandardParameterConfigSourceDescriptor::new(
            ConfigSourceLevel::User,
            ConfigPathOrigin::Override,
            non_object,
        ));
    let not_file =
        load_standard_parameter_config_source(&StandardParameterConfigSourceDescriptor::new(
            ConfigSourceLevel::Project,
            ConfigPathOrigin::Override,
            not_file,
        ));

    assert_warning_reason(&invalid, "invalid_json");
    assert_warning_reason(&non_object, "non_object");
    assert_warning_reason(&not_file, "not_file");
}

#[test]
fn unreadable_config_source_is_skipped_with_warning_event() {
    let Some((path, _guard)) = unreadable_file("unreadable-config.json") else {
        return;
    };

    let loaded =
        load_standard_parameter_config_source(&StandardParameterConfigSourceDescriptor::new(
            ConfigSourceLevel::Project,
            ConfigPathOrigin::Override,
            path,
        ));

    assert_warning_reason(&loaded, "unreadable");
}

#[test]
fn constructed_passthrough_values_keep_source_processing_results_without_validation() {
    let catalog = parameter_catalog();
    let entries = catalog.entries();
    let direct_input = json!({"native": {"shared": 1}, "limit_chars": 100});
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

    let resolution = resolve_standard_parameters(
        entries,
        StandardParameterSources {
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
            ..StandardParameterSources::default()
        },
        EntryPassthroughPolicy::Delegate,
    );

    assert!(resolution.diagnostics().is_empty());
    assert_eq!(resolution.passthrough().len(), 3);
    let native = passthrough_from(&resolution, StandardParameterSourceKind::DirectInput);
    assert_eq!(native.value, direct_passthrough);
    assert_eq!(
        native.source,
        StandardParameterSourceInfo::new(StandardParameterSourceKind::DirectInput)
    );
    assert_eq!(native.disposition, PassthroughDisposition::Delegated);
    let project = passthrough_from(&resolution, StandardParameterSourceKind::ProjectConfig);
    assert_eq!(project.value, project_passthrough);
    let user = passthrough_from(&resolution, StandardParameterSourceKind::UserConfig);
    assert_eq!(user.value, user_config);
}

#[test]
fn empty_object_passthrough_is_preserved_for_entry_owner() {
    let catalog = parameter_catalog();
    let entries = catalog.entries();
    let direct_input = json!({"output": "readable-view", "native": {}});
    let direct_passthrough = json!({"native": {}});

    let resolution = resolve_standard_parameters(
        entries,
        StandardParameterSources {
            direct_input: construct_direct_input_source_with_passthrough(
                entries,
                Some(&direct_input),
                Some(&direct_passthrough),
            ),
            ..StandardParameterSources::default()
        },
        EntryPassthroughPolicy::Delegate,
    );

    assert!(resolution.diagnostics().is_empty());
    assert_eq!(resolution.passthrough().len(), 1);
    let passthrough = &resolution.passthrough()[0];
    assert_eq!(passthrough.value, direct_passthrough);
    assert_eq!(
        passthrough.source,
        StandardParameterSourceInfo::new(StandardParameterSourceKind::DirectInput)
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

    let resolution = resolve_standard_parameters(
        entries,
        StandardParameterSources {
            project_config: construct_config_source_with_passthrough(
                entries,
                Some(&project_config),
                Some(&project_passthrough),
            ),
            ..StandardParameterSources::default()
        },
        EntryPassthroughPolicy::Delegate,
    );

    assert!(resolution.diagnostics().is_empty());
    let output = resolution
        .value(&identity("docnav.defaults.output"))
        .unwrap();
    assert_eq!(output.value, TypedValue::String("readable-view".to_owned()));
    let passthrough = passthrough_from(&resolution, StandardParameterSourceKind::ProjectConfig);
    assert_eq!(passthrough.value, project_passthrough);
    assert_eq!(
        passthrough.source,
        StandardParameterSourceInfo::new(StandardParameterSourceKind::ProjectConfig)
    );
    assert_eq!(passthrough.disposition, PassthroughDisposition::Delegated);
}

fn assert_warning_reason(loaded: &LoadedStandardParameterConfigSource, reason_code: &str) {
    let warning = loaded.diagnostics()[0].as_warning().unwrap();
    let DiagnosticDetails::AdapterConfigSource {
        reason_code: actual,
        ..
    } = warning.details()
    else {
        panic!("expected adapter config source warning details");
    };
    assert_eq!(actual, reason_code);
}
