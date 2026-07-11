use cli_config_resolution::{FieldDefSet, ResolutionResult};
use docnav_adapter_contracts::{AdapterError, AdapterOptionSpec, NativeOptionIssue};
use docnav_diagnostics::AdapterConfigSourceDetails;
use docnav_protocol::{OptionEntry, Options};
use serde_json::Value;

use crate::{NavigationCommand, NavigationConfigSources, NavigationError};

use super::{
    input::native_option_cli_value,
    values::{field_source_label, projected_field_value, typed_value_to_json},
};

pub(super) struct UnsupportedOptionContext<'a> {
    pub source: &'static str,
    pub path_origin: Option<&'static str>,
    pub path: &'a str,
    pub owner: &'a str,
    pub config_field: Option<String>,
    pub selected_native_options: &'a [AdapterOptionSpec],
}

pub(super) fn resolved_options(
    fields: &FieldDefSet,
    resolution: &ResolutionResult,
    selected_native_options: &[AdapterOptionSpec],
) -> Result<Options, NavigationError> {
    options_from_resolution(fields, resolution, selected_native_options)
}

pub(super) fn native_option_validation_error(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    spec: &AdapterOptionSpec,
    source: Option<&str>,
    reason_code: &str,
) -> AdapterError {
    let source = source.unwrap_or("explicit");
    let value = raw_native_option_value_for_source(command, config_sources, spec, source);
    let issue = NativeOptionIssue {
        owner: spec.owner.clone(),
        namespace: spec.namespace().to_owned(),
        key: spec.key().to_owned(),
        source: source.to_owned(),
        reason_code: reason_code.to_owned(),
        field: format!("arguments.options.{}", spec.key()),
        received: value.as_ref().map(received_value),
        expected: Some(spec.expected_value_description()),
        type_variant: Some(spec.value_kind_name().to_owned()),
        config_source: option_config_source_issue(config_sources, spec, source, reason_code),
    };
    AdapterError::native_option_invalid(
        "Native option value is invalid.",
        issue,
        [format!(
            "Use {} for option {}.",
            spec.expected_value_description(),
            spec.key()
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
        config_source: context.path_origin.map(|path_origin| {
            let field = context
                .config_field
                .unwrap_or_else(|| format!("options.{key}"));
            AdapterConfigSourceDetails::new(
                context.source,
                path_origin,
                context.path,
                "unsupported",
            )
            .with_field(field)
        }),
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

pub(super) fn spec_for_identity<'a>(
    specs: &'a [AdapterOptionSpec],
    identity: &str,
) -> Option<&'a AdapterOptionSpec> {
    specs.iter().find(|spec| spec.identity == identity)
}

fn options_from_resolution(
    fields: &FieldDefSet,
    resolution: &ResolutionResult,
    selected_native_options: &[AdapterOptionSpec],
) -> Result<Options, NavigationError> {
    let mut options = Options::new();
    for (identity, resolved) in resolution.fields() {
        let Some(value) = projected_field_value(fields, identity, resolved) else {
            continue;
        };
        let Some(spec) = spec_for_identity(selected_native_options, identity.as_str()) else {
            continue;
        };
        options.insert_entry(OptionEntry {
            identity: spec.identity.clone(),
            owner: spec.owner.clone(),
            namespace: spec.namespace().to_owned(),
            key: spec.key().to_owned(),
            source: field_source_label(resolved)
                .unwrap_or("built_in")
                .to_owned(),
            type_variant: spec.value_kind_name().to_owned(),
            value: typed_value_to_json(value),
        });
    }
    Ok(options)
}

fn raw_native_option_value_for_source(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    spec: &AdapterOptionSpec,
    source: &str,
) -> Option<Value> {
    match source {
        "explicit" => spec.cli_flag().and_then(|cli_flag| {
            command
                .native_options
                .iter()
                .find(|option| option.flag == cli_flag)
                .map(|option| native_option_cli_value(&option.value))
        }),
        "project" => spec
            .processing_path(super::CONFIG_PROCESSING)
            .ok()
            .flatten()
            .and_then(|path| value_at_path(config_sources.project.loaded.value()?, &path)),
        "user" => spec
            .processing_path(super::CONFIG_PROCESSING)
            .ok()
            .flatten()
            .and_then(|path| value_at_path(config_sources.user.loaded.value()?, &path)),
        _ => None,
    }
}

fn option_config_source_issue(
    config_sources: &NavigationConfigSources,
    spec: &AdapterOptionSpec,
    source: &str,
    reason_code: &str,
) -> Option<AdapterConfigSourceDetails> {
    let config_source = match source {
        "project" => &config_sources.project,
        "user" => &config_sources.user,
        _ => return None,
    };
    let field = spec
        .processing_path(super::CONFIG_PROCESSING)
        .ok()
        .flatten()
        .map(|path| path.join("."))
        .unwrap_or_else(|| format!("options.{}", spec.key()));
    Some(
        AdapterConfigSourceDetails::new(
            config_source.level.as_str(),
            config_source.origin.as_str(),
            &config_source.path,
            reason_code,
        )
        .with_field(field),
    )
}

fn supported_option_keys(selected_native_options: &[AdapterOptionSpec]) -> String {
    let keys = selected_native_options
        .iter()
        .map(|spec| spec.key())
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

fn value_at_path(root: &Value, path: &[String]) -> Option<Value> {
    let mut current = root;
    for segment in path {
        current = current.get(segment)?;
    }
    Some(current.clone())
}
