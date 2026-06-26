use std::collections::BTreeMap;

use docnav_diagnostics::{WarningDetails, WarningEffect};
use docnav_typed_fields::{
    DefaultMetadata, ExtractionStrategyId, FieldNumericRange, TypedValue, ValidationReason,
    ValueKind,
};
use serde_json::json;

use super::*;

#[test]
fn strategy_metadata_projection_includes_strategy_path_and_schema_facts() {
    let definitions = Params::field_defs().unwrap();
    let metadata = definitions.strategy_metadata(&ExtractionStrategyId::from(CONFIG_STRATEGY));
    let limit = metadata
        .iter()
        .find(|metadata| metadata.identity.as_str() == "docnav.defaults.limit_chars")
        .unwrap();

    assert_eq!(limit.strategy_id.as_str(), CONFIG_STRATEGY);
    assert_eq!(limit.path.segments(), vec!["defaults", "limit_chars"]);
    assert_eq!(limit.value_kind, ValueKind::Integer);
    let FieldNumericRange::Integer(range) = limit.constraints.numeric_range else {
        panic!("expected integer range");
    };
    assert_eq!(range.minimum.unwrap().value, 1);
    assert_eq!(limit.default, DefaultMetadata::Static(json!(20_000)));
}

#[test]
fn facade_constructs_direct_config_and_default_sources_from_registration_bindings() {
    let registrations = registration_set();
    let identity = identity("docnav.defaults.limit_chars");
    let mut dynamic_defaults = BTreeMap::new();
    dynamic_defaults.insert(identity.clone(), json!(400));

    let resolution = resolve_standard_parameter_inputs(
        StandardParameterResolutionInputs::new(registrations.as_slice())
            .with_direct_input(json!({
                "limit_chars": 100,
                "output": "readable-json",
                "native_options": {"theme": "direct"}
            }))
            .with_project_config(json!({
                "defaults": {"limit_chars": 200, "output": "readable-view"},
                "native_options": {"theme": "project"}
            }))
            .with_user_config(json!({
                "defaults": {"limit_chars": 300, "output": "readable-view"}
            }))
            .with_dynamic_defaults(dynamic_defaults)
            .with_passthrough_policy(EntryPassthroughPolicy::Retain),
    );

    let resolved = resolution.value(&identity).unwrap();
    assert_eq!(resolved.value, TypedValue::Integer(100));
    assert_eq!(
        resolved.source,
        StandardParameterSourceInfo::new(StandardParameterSourceKind::DirectInput)
    );
    assert!(resolution.diagnostics().is_empty());
}

#[test]
fn registration_set_rejects_conflicting_config_paths() {
    let definitions = Params::field_defs().unwrap();
    let mut registrations = definitions
        .schema_metadata()
        .into_iter()
        .map(StandardParameterRegistration::new)
        .collect::<Vec<_>>();
    registrations[0] = registrations[0]
        .clone()
        .with_config_binding(StandardParameterBinding::new(
            CONFIG_STRATEGY,
            path(["defaults", "shared"]),
        ));
    registrations[1] = registrations[1]
        .clone()
        .with_config_binding(StandardParameterBinding::new(
            CONFIG_STRATEGY,
            path(["defaults", "shared"]),
        ));

    let error = StandardParameterRegistrationSet::new(registrations).unwrap_err();

    assert_eq!(
        error.kind,
        StandardParameterRegistrationConflictKind::ConfigPath
    );
    assert_eq!(error.path.unwrap(), path(["defaults", "shared"]));
}

#[test]
fn facade_reports_validation_failures_through_diagnostic_handoff() {
    let registrations = registration_set();
    let identity = identity("docnav.defaults.limit_chars");

    let resolution = resolve_standard_parameter_inputs(
        StandardParameterResolutionInputs::new(registrations.as_slice())
            .with_direct_input(json!({"limit_chars": 0}))
            .with_project_config(json!({"defaults": {"limit_chars": 200}})),
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
    let registrations = registration_set();
    let missing = temp_path("missing-project-config.json");
    let loaded_project =
        load_standard_parameter_config_source(&StandardParameterConfigSourceDescriptor::new(
            ConfigSourceLevel::Project,
            ConfigPathOrigin::Override,
            missing.clone(),
        ));
    let loaded_user = LoadedStandardParameterConfigSource::from_value(json!({
        "defaults": {"limit_chars": 300, "output": "readable-view"}
    }));

    assert_eq!(loaded_project.diagnostics().len(), 1);
    let warning = loaded_project.diagnostics()[0].as_warning().unwrap();
    assert_eq!(warning.effect, WarningEffect::OperationContinued);
    assert_eq!(
        warning.details,
        WarningDetails::AdapterConfigSource {
            source_level: "project".to_owned(),
            path_origin: "override".to_owned(),
            path: missing.display().to_string(),
            reason_code: "missing_override".to_owned(),
        }
    );

    let resolution = resolve_standard_parameter_inputs(
        StandardParameterResolutionInputs::new(registrations.as_slice())
            .with_loaded_project_config(loaded_project)
            .with_loaded_user_config(loaded_user),
    );

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
fn constructed_passthrough_values_merge_by_source_priority_without_validation() {
    let registrations = registration_set();

    let resolution = resolve_standard_parameter_inputs(
        StandardParameterResolutionInputs::new(registrations.as_slice())
            .with_direct_input(json!({"native": {"shared": 1}, "limit_chars": 100}))
            .with_project_config(json!({
                "defaults": {"output": "readable-view"},
                "native": {"shared": 2, "project": true}
            }))
            .with_user_config(json!({
                "native": {"shared": 3, "user": true}
            }))
            .with_passthrough_policy(EntryPassthroughPolicy::Delegate),
    );

    assert!(resolution.diagnostics().is_empty());
    let shared = resolution
        .passthrough()
        .iter()
        .find(|value| value.path == path(["native", "shared"]))
        .unwrap();
    assert_eq!(shared.value, json!(1));
    assert_eq!(
        shared.source,
        StandardParameterSourceInfo::new(StandardParameterSourceKind::DirectInput)
    );
    assert_eq!(shared.disposition, PassthroughDisposition::Delegated);
}

fn assert_warning_reason(loaded: &LoadedStandardParameterConfigSource, reason_code: &str) {
    let warning = loaded.diagnostics()[0].as_warning().unwrap();
    let WarningDetails::AdapterConfigSource {
        reason_code: actual,
        ..
    } = &warning.details
    else {
        panic!("expected adapter config source warning details");
    };
    assert_eq!(actual, reason_code);
}
