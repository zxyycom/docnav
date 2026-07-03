use docnav_adapter_contracts::{AdapterError, NativeOptionIssue, NativeOptionSpec};
use docnav_parameter_resolution::{ParameterResolution, ParameterSourceKind};
use docnav_protocol::{OptionEntry, Options};
use docnav_typed_fields::ValidationReason;
use serde_json::{Map, Value};

use crate::{NavigationCommand, NavigationConfigSources, NavigationError};

use super::{
    input::native_option_cli_value,
    values::{source_label, typed_value_to_json},
};

pub(super) struct UnsupportedOptionContext<'a> {
    pub source: &'static str,
    pub path: &'a str,
    pub owner: &'a str,
    pub selected_native_options: &'a [NativeOptionSpec],
}

pub(super) fn resolved_options(
    resolution: &ParameterResolution,
    selected_native_options: &[NativeOptionSpec],
) -> Result<Options, NavigationError> {
    options_from_resolution(resolution, selected_native_options)
}

pub(super) fn native_option_validation_error(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    spec: NativeOptionSpec,
    source: Option<ParameterSourceKind>,
    reason: &ValidationReason,
) -> AdapterError {
    let source = source.map(source_label).unwrap_or("explicit");
    let value = raw_native_option_value_for_source(command, config_sources, spec, source);
    let issue = NativeOptionIssue {
        owner: spec.owner.to_owned(),
        namespace: spec.namespace.to_owned(),
        key: spec.key.to_owned(),
        source: source.to_owned(),
        reason_code: native_option_reason_code(reason).to_owned(),
        field: format!("arguments.options.{}", spec.key),
        received: value.as_ref().map(received_value),
        expected: Some(spec.expected_value_description()),
        type_variant: Some(spec.value_kind().as_str().to_owned()),
    };
    AdapterError::native_option_invalid(
        "Native option value is invalid.",
        issue,
        [format!(
            "Use {} for option {}.",
            spec.expected_value_description(),
            spec.key
        )],
    )
}

pub(super) fn unsupported_option(
    context: UnsupportedOptionContext<'_>,
    key: &str,
    value: Value,
) -> AdapterError {
    let issue = NativeOptionIssue {
        owner: context.owner.to_owned(),
        namespace: "options".to_owned(),
        key: key.to_owned(),
        source: context.source.to_owned(),
        reason_code: "unsupported".to_owned(),
        field: format!("arguments.options.{key}"),
        received: Some(received_value(&value)),
        expected: Some(supported_option_keys(context.selected_native_options)),
        type_variant: None,
    };
    AdapterError::native_option_invalid(
        "Native option is not supported by the selected adapter.",
        issue,
        [format!(
            "Remove option {key} from {} or select an adapter that supports it.",
            context.path
        )],
    )
}

pub(super) fn spec_for_identity(
    specs: &[NativeOptionSpec],
    identity: &str,
) -> Option<NativeOptionSpec> {
    specs.iter().copied().find(|spec| spec.identity == identity)
}

pub(super) fn config_options(config: Option<&Value>) -> Option<&Map<String, Value>> {
    config?.get("options").and_then(Value::as_object)
}

fn options_from_resolution(
    resolution: &ParameterResolution,
    selected_native_options: &[NativeOptionSpec],
) -> Result<Options, NavigationError> {
    let mut options = Options::new();
    for (identity, resolved) in resolution.values() {
        let Some(spec) = spec_for_identity(selected_native_options, identity.as_str()) else {
            continue;
        };
        options.insert_entry(OptionEntry {
            identity: spec.identity.to_owned(),
            owner: spec.owner.to_owned(),
            namespace: spec.namespace.to_owned(),
            key: spec.key.to_owned(),
            source: source_label(resolved.source.kind).to_owned(),
            type_variant: spec.value_kind().as_str().to_owned(),
            value: typed_value_to_json(&resolved.value),
        });
    }
    Ok(options)
}

fn raw_native_option_value_for_source(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    spec: NativeOptionSpec,
    source: &str,
) -> Option<Value> {
    match source {
        "explicit" => spec.cli_flag.and_then(|cli_flag| {
            command
                .native_options
                .iter()
                .find(|option| option.flag == cli_flag)
                .map(|option| native_option_cli_value(&option.value))
        }),
        "project" => config_options(config_sources.project.loaded.value())
            .and_then(|options| options.get(spec.key))
            .cloned(),
        "user" => config_options(config_sources.user.loaded.value())
            .and_then(|options| options.get(spec.key))
            .cloned(),
        _ => None,
    }
}

fn native_option_reason_code(reason: &ValidationReason) -> &'static str {
    match reason {
        ValidationReason::WrongType { .. } => "type_mismatch",
        ValidationReason::BelowMinimum { .. } | ValidationReason::AboveMaximum { .. } => {
            "range_invalid"
        }
        ValidationReason::MissingRequired => "missing_required",
        ValidationReason::DisallowedEnumValue { .. } => "enum_invalid",
        ValidationReason::BelowMinimumLength { .. }
        | ValidationReason::AboveMaximumLength { .. } => "length_invalid",
        ValidationReason::RegexMismatch { .. } => "pattern_invalid",
        ValidationReason::DuplicateArrayItem { .. } => "duplicate_item",
    }
}

fn supported_option_keys(selected_native_options: &[NativeOptionSpec]) -> String {
    let keys = selected_native_options
        .iter()
        .map(|spec| spec.key)
        .collect::<Vec<_>>();
    if keys.is_empty() {
        "no native options".to_owned()
    } else {
        keys.join(", ")
    }
}

fn received_value(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| value.to_string())
}
