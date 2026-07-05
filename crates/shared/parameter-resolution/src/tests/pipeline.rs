use docnav_typed_fields::{ProcessingBuild, TypedValue};
use serde_json::json;

use super::*;

#[test]
fn pipeline_derives_catalog_from_field_defs_without_manual_catalog_assembly() {
    let fields = Params::field_defs().unwrap();

    let resolution = ParameterResolutionPipeline::new(&fields)
        .with_direct_input_processing_id(DIRECT_PROCESSING)
        .with_config_processing_id(CONFIG_PROCESSING)
        .resolve(json!({"output": "readable-json"}))
        .unwrap();

    let output = resolution
        .value(&identity("docnav.defaults.output"))
        .unwrap();
    assert_eq!(
        output.source,
        ParameterSourceInfo::new(ParameterSourceKind::DirectInput)
    );
    assert_eq!(output.value, TypedValue::String("readable-json".to_owned()));

    let limit = resolution
        .value(&identity("docnav.defaults.pagination.limit"))
        .unwrap();
    assert_eq!(
        limit.source,
        ParameterSourceInfo::new(ParameterSourceKind::Default)
    );
    assert_eq!(limit.value, TypedValue::Integer(20_000));
}

#[test]
fn pipeline_resolves_paths_defaults_diagnostics_and_passthrough_through_facade() {
    let fields = Params::field_defs().unwrap();
    let missing_project = temp_path("pipeline-missing-project-config.json");
    let user_config = temp_file(
        "pipeline-user-config.json",
        r#"{
            "defaults": {"output": "readable-json"},
            "native": {"shared": "user"}
        }"#,
    );

    let resolution = ParameterResolutionPipeline::new(&fields)
        .with_direct_input_processing_id(DIRECT_PROCESSING)
        .with_config_processing_id(CONFIG_PROCESSING)
        .with_config_source_descriptor(ParameterConfigSourceDescriptor::new(
            ConfigSourceLevel::Project,
            ConfigPathOrigin::ExplicitCli,
            missing_project.clone(),
        ))
        .with_user_config_path(user_config)
        .with_dynamic_default(identity("docnav.defaults.pagination.limit"), json!(500))
        .with_passthrough_policy(EntryPassthroughPolicy::Delegate)
        .resolve(json!({"native": {"shared": "direct"}}))
        .unwrap();

    let limit = resolution
        .value(&identity("docnav.defaults.pagination.limit"))
        .unwrap();
    assert_eq!(limit.value, TypedValue::Integer(500));
    assert_eq!(
        limit.source,
        ParameterSourceInfo::new(ParameterSourceKind::Default)
    );

    let output = resolution
        .value(&identity("docnav.defaults.output"))
        .unwrap();
    assert_eq!(output.value, TypedValue::String("readable-json".to_owned()));
    assert_eq!(
        output.source,
        ParameterSourceInfo::new(ParameterSourceKind::UserConfig)
    );

    let issue = resolution
        .diagnostics()
        .iter()
        .find_map(ParameterResolutionHandoff::as_config_source)
        .unwrap();
    assert_config_source_issue(
        issue,
        "project",
        "explicit_cli",
        &missing_project,
        "missing_explicit_cli",
    );

    let passthrough = passthrough_from(&resolution, ParameterSourceKind::DirectInput);
    assert_eq!(passthrough.value, json!({"native": {"shared": "direct"}}));
    assert_eq!(
        passthrough.source,
        ParameterSourceInfo::new(ParameterSourceKind::DirectInput)
    );
    assert_eq!(passthrough.disposition, PassthroughDisposition::Delegated);
}

#[test]
fn pipeline_config_path_facade_uses_override_origin() {
    let fields = Params::field_defs().unwrap();
    let missing_project = temp_path("pipeline-missing-override-project-config.json");

    let resolution = ParameterResolutionPipeline::new(&fields)
        .with_direct_input_processing_id(DIRECT_PROCESSING)
        .with_config_processing_id(CONFIG_PROCESSING)
        .with_project_config_path(missing_project.clone())
        .resolve(None::<JsonValue>)
        .unwrap();

    let issue = resolution
        .diagnostics()
        .iter()
        .find_map(ParameterResolutionHandoff::as_config_source)
        .unwrap();
    assert_config_source_issue(
        issue,
        "project",
        "override",
        &missing_project,
        "missing_override",
    );
}

#[test]
fn pipeline_uses_direct_input_passthrough_processing_result_as_is() {
    let fields = Params::field_defs().unwrap();
    let passthrough_processing = ProcessingBuild::new(
        DIRECT_PROCESSING,
        |raw: JsonValue| json!({"native": raw.get("native").cloned().unwrap()}),
    )
    .unwrap();

    let resolution = ParameterResolutionPipeline::new(&fields)
        .with_direct_input_processing_id(DIRECT_PROCESSING)
        .with_config_processing_id(CONFIG_PROCESSING)
        .with_direct_input_passthrough_processing(passthrough_processing)
        .with_passthrough_policy(EntryPassthroughPolicy::Delegate)
        .resolve(json!({"output": "readable-view", "native": {}}))
        .unwrap();

    let output = resolution
        .value(&identity("docnav.defaults.output"))
        .unwrap();
    assert_eq!(output.value, TypedValue::String("readable-view".to_owned()));
    let passthrough = passthrough_from(&resolution, ParameterSourceKind::DirectInput);
    assert_eq!(passthrough.value, json!({"native": {}}));
    assert_eq!(passthrough.disposition, PassthroughDisposition::Delegated);
}

#[test]
fn pipeline_uses_config_passthrough_processing_result_as_is() {
    let fields = Params::field_defs().unwrap();
    let project_config = json!({
        "defaults": {
            "output": "readable-view",
            "native": {
                "theme": "dark",
                "strict": true
            }
        }
    });
    let passthrough_processing = ProcessingBuild::new(
        CONFIG_PROCESSING,
        |raw: JsonValue| json!({"defaults": {"native": raw["defaults"]["native"].clone()}}),
    )
    .unwrap();

    let resolution = ParameterResolutionPipeline::new(&fields)
        .with_direct_input_processing_id(DIRECT_PROCESSING)
        .with_config_processing_id(CONFIG_PROCESSING)
        .with_loaded_project_config(LoadedParameterConfigSource::from_value(project_config))
        .with_config_passthrough_processing(passthrough_processing)
        .with_passthrough_policy(EntryPassthroughPolicy::Delegate)
        .resolve(None::<JsonValue>)
        .unwrap();

    let output = resolution
        .value(&identity("docnav.defaults.output"))
        .unwrap();
    assert_eq!(output.value, TypedValue::String("readable-view".to_owned()));
    let passthrough = passthrough_from(&resolution, ParameterSourceKind::ProjectConfig);
    assert_eq!(
        passthrough.value,
        json!({"defaults": {"native": {"theme": "dark", "strict": true}}})
    );
    assert_eq!(passthrough.disposition, PassthroughDisposition::Delegated);
}

#[test]
fn pipeline_reuses_loaded_config_sources_from_standard_loader() {
    let fields = Params::field_defs().unwrap();
    let invalid_project = temp_file("pipeline-loaded-invalid-project.json", "{invalid");
    let user_config = temp_file(
        "pipeline-loaded-user.json",
        r#"{"defaults": {"pagination": {"limit": 700}, "output": "readable-view"}}"#,
    );
    let loaded_project = load_parameter_config_source(&ParameterConfigSourceDescriptor::new(
        ConfigSourceLevel::Project,
        ConfigPathOrigin::ExplicitCli,
        invalid_project.clone(),
    ));
    let loaded_user = load_parameter_config_source(&ParameterConfigSourceDescriptor::new(
        ConfigSourceLevel::User,
        ConfigPathOrigin::ExplicitCli,
        user_config,
    ));

    assert!(loaded_project.value().is_none());
    assert_eq!(loaded_project.diagnostics().len(), 1);
    assert!(loaded_user.value().is_some());

    let resolution = ParameterResolutionPipeline::new(&fields)
        .with_direct_input_processing_id(DIRECT_PROCESSING)
        .with_config_processing_id(CONFIG_PROCESSING)
        .with_loaded_project_config(loaded_project)
        .with_loaded_user_config(loaded_user)
        .resolve(None::<JsonValue>)
        .unwrap();

    let limit = resolution
        .value(&identity("docnav.defaults.pagination.limit"))
        .unwrap();
    assert_eq!(limit.value, TypedValue::Integer(700));
    assert_eq!(
        limit.source,
        ParameterSourceInfo::new(ParameterSourceKind::UserConfig)
    );

    assert_eq!(resolution.diagnostics().len(), 1);
    let issue = resolution.diagnostics()[0].as_config_source().unwrap();
    assert_config_source_issue(
        issue,
        "project",
        "explicit_cli",
        &invalid_project,
        "invalid_json",
    );
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
