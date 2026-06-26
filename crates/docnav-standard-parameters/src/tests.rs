#![allow(dead_code)]

// @case WB-STDPARAMS-RESOLVE-001
use docnav_typed_fields::{
    ExtractStrategy, FieldBound, FieldDef, FieldDefs, FieldIdentity, FieldValidation, JsonValue,
    TypedValue, ValidationReason,
};
use serde_json::json;

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputMode {
    ReadableView,
    ReadableJson,
}

impl docnav_typed_fields::FieldStringEnum for OutputMode {
    fn variants() -> &'static [Self] {
        &[Self::ReadableView, Self::ReadableJson]
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::ReadableView => "readable-view",
            Self::ReadableJson => "readable-json",
        }
    }
}

const CONFIG_STRATEGY: &str = "config";

fn config_json_path<I, S>(segments: I) -> ExtractStrategy
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    ExtractStrategy::json_path(segments)
}

#[derive(Debug, FieldDefs)]
struct Params {
    #[field(
        FieldDef::builder("docnav.defaults.limit_chars")
            .extract(CONFIG_STRATEGY, config_json_path(["defaults", "limit_chars"]))
            .validation(FieldValidation::int().between(
                FieldBound::closed(1),
                FieldBound::closed(100_000),
            ))
            .default_static(20_000)
    )]
    limit_chars: Option<i64>,

    #[field(
        FieldDef::builder("docnav.defaults.output")
            .extract(CONFIG_STRATEGY, config_json_path(["defaults", "output"]))
            .validation(FieldValidation::string_enum::<OutputMode>())
    )]
    output: OutputMode,
}

#[test]
fn direct_project_user_default_priority_preserves_source_info() {
    let registrations = vec![registration("docnav.defaults.limit_chars")];
    let identity = identity("docnav.defaults.limit_chars");
    let sources = StandardParameterSources {
        direct_input: source_with_value(&identity, json!(100)),
        project_config: source_with_value(&identity, json!(200)),
        user_config: source_with_value(&identity, json!(300)),
        default: source_with_value(&identity, json!(400)),
    };

    let resolution =
        resolve_standard_parameters(&registrations, sources, EntryPassthroughPolicy::Retain);

    let resolved = resolution.value(&identity).unwrap();
    assert_eq!(resolved.value, TypedValue::Integer(100));
    assert_eq!(
        resolved.source,
        StandardParameterSourceInfo::new(StandardParameterSourceKind::DirectInput)
    );
    assert!(resolution.diagnostics().is_empty());
}

#[test]
fn project_config_overrides_user_config_and_static_default_fills_absent_value() {
    let registrations = vec![registration("docnav.defaults.limit_chars")];
    let identity = identity("docnav.defaults.limit_chars");
    let project_resolution = resolve_standard_parameters(
        &registrations,
        StandardParameterSources {
            project_config: source_with_value(&identity, json!(200)),
            user_config: source_with_value(&identity, json!(300)),
            ..StandardParameterSources::default()
        },
        EntryPassthroughPolicy::Retain,
    );

    let project_value = project_resolution.value(&identity).unwrap();
    assert_eq!(project_value.value, TypedValue::Integer(200));
    assert_eq!(
        project_value.source,
        StandardParameterSourceInfo::new(StandardParameterSourceKind::ProjectConfig)
    );

    let default_resolution = resolve_standard_parameters(
        &registrations,
        StandardParameterSources::default(),
        EntryPassthroughPolicy::Retain,
    );

    let default_value = default_resolution.value(&identity).unwrap();
    assert_eq!(default_value.value, TypedValue::Integer(20_000));
    assert_eq!(
        default_value.source,
        StandardParameterSourceInfo::new(StandardParameterSourceKind::Default)
    );
}

#[test]
fn invalid_mapped_value_reports_diagnostic_without_safe_value() {
    let registrations = vec![registration("docnav.defaults.limit_chars")];
    let identity = identity("docnav.defaults.limit_chars");
    let resolution = resolve_standard_parameters(
        &registrations,
        StandardParameterSources {
            direct_input: source_with_value(&identity, json!(0)),
            project_config: source_with_value(&identity, json!(200)),
            ..StandardParameterSources::default()
        },
        EntryPassthroughPolicy::Retain,
    );

    assert!(resolution.value(&identity).is_none());
    assert_eq!(resolution.diagnostics().len(), 1);
    assert_eq!(
        resolution.diagnostics()[0].source,
        Some(StandardParameterSourceInfo::new(
            StandardParameterSourceKind::DirectInput
        ))
    );
    assert!(matches!(
        resolution.diagnostics()[0].failure.reason,
        ValidationReason::BelowMinimum { .. }
    ));
}

#[test]
fn required_missing_value_reports_standard_parameter_diagnostic() {
    let registrations = vec![registration("docnav.defaults.output")];
    let identity = identity("docnav.defaults.output");

    let resolution = resolve_standard_parameters(
        &registrations,
        StandardParameterSources::default(),
        EntryPassthroughPolicy::Retain,
    );

    assert!(resolution.value(&identity).is_none());
    assert_eq!(resolution.diagnostics().len(), 1);
    assert_eq!(resolution.diagnostics()[0].source, None);
    assert_eq!(
        resolution.diagnostics()[0].failure.reason,
        ValidationReason::MissingRequired
    );
}

#[test]
fn dynamic_default_source_is_validated_like_other_mapped_values() {
    let registrations = vec![registration("docnav.defaults.limit_chars")];
    let identity = identity("docnav.defaults.limit_chars");
    let resolution = resolve_standard_parameters(
        &registrations,
        StandardParameterSources {
            default: source_with_value(&identity, json!(0)),
            ..StandardParameterSources::default()
        },
        EntryPassthroughPolicy::Retain,
    );

    assert!(resolution.value(&identity).is_none());
    assert_eq!(resolution.diagnostics().len(), 1);
    assert_eq!(
        resolution.diagnostics()[0].source,
        Some(StandardParameterSourceInfo::new(
            StandardParameterSourceKind::Default
        ))
    );
    assert!(matches!(
        resolution.diagnostics()[0].failure.reason,
        ValidationReason::BelowMinimum { .. }
    ));
}

#[test]
fn passthrough_remains_outside_standard_parameter_validation() {
    let registrations = vec![registration("docnav.defaults.limit_chars")];
    let mut sources = StandardParameterSources::default();
    sources.direct_input.push_passthrough(
        path(["native_options", "future_flag"]),
        json!({"adapter": "owned"}),
    );

    let resolution =
        resolve_standard_parameters(&registrations, sources, EntryPassthroughPolicy::Delegate);

    assert!(resolution.diagnostics().is_empty());
    assert_eq!(resolution.passthrough().len(), 1);
    assert_eq!(
        resolution.passthrough()[0].source,
        StandardParameterSourceInfo::new(StandardParameterSourceKind::DirectInput)
    );
    assert_eq!(
        resolution.passthrough()[0].disposition,
        PassthroughDisposition::Delegated
    );
}

#[test]
fn operation_argument_binding_preserves_direct_config_and_default_source_info() {
    let identity = identity("docnav.defaults.limit_chars");

    assert_operation_binding_source(
        StandardParameterSources {
            direct_input: source_with_value(&identity, json!(100)),
            project_config: source_with_value(&identity, json!(200)),
            user_config: source_with_value(&identity, json!(300)),
            ..StandardParameterSources::default()
        },
        StandardParameterSourceKind::DirectInput,
    );
    assert_operation_binding_source(
        StandardParameterSources {
            project_config: source_with_value(&identity, json!(200)),
            user_config: source_with_value(&identity, json!(300)),
            ..StandardParameterSources::default()
        },
        StandardParameterSourceKind::ProjectConfig,
    );
    assert_operation_binding_source(
        StandardParameterSources {
            user_config: source_with_value(&identity, json!(300)),
            ..StandardParameterSources::default()
        },
        StandardParameterSourceKind::UserConfig,
    );
    assert_operation_binding_source(
        StandardParameterSources::default(),
        StandardParameterSourceKind::Default,
    );
}

fn assert_operation_binding_source(
    sources: StandardParameterSources,
    expected_source: StandardParameterSourceKind,
) {
    let identity = identity("docnav.defaults.limit_chars");
    let registration = registration("docnav.defaults.limit_chars")
        .with_operation_argument(OperationArgumentBinding::new(path(["limit_chars"])));
    let resolution =
        resolve_standard_parameters(&[registration], sources, EntryPassthroughPolicy::Retain);

    let resolved = resolution.value(&identity).unwrap();
    let binding = resolved.operation_argument.as_ref().unwrap();
    assert_eq!(binding.arguments_path, path(["limit_chars"]));
    assert_eq!(
        binding.source,
        StandardParameterSourceInfo::new(expected_source)
    );
}

fn registration(identity: &str) -> StandardParameterRegistration {
    let metadata = Params::field_defs()
        .unwrap()
        .schema_metadata()
        .into_iter()
        .find(|metadata| metadata.identity.as_str() == identity)
        .unwrap();
    StandardParameterRegistration::new(metadata)
}

fn source_with_value(identity: &FieldIdentity, value: JsonValue) -> StandardParameterSource {
    StandardParameterSource::default().with_value(identity.clone(), value)
}

fn identity(value: &str) -> FieldIdentity {
    FieldIdentity::new(value).unwrap()
}

fn path<const N: usize>(segments: [&str; N]) -> StandardParameterPath {
    StandardParameterPath::new(segments).unwrap()
}
