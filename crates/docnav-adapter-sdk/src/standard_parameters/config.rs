use docnav_standard_parameters::{
    load_standard_parameter_config_source, LoadedStandardParameterConfigSource,
    StandardParameterConfigSourceDescriptor, StandardParameterConfigSourceIssue,
};
use docnav_typed_fields::JsonValue;
use serde_json::{Map, Value};

use crate::AdapterError;

const ROOT_KEYS: &[&str] = &["defaults", "options"];
const DEFAULTS_KEYS: &[&str] = &["pagination", "output"];
const PAGINATION_KEYS: &[&str] = &["enabled", "limit"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InvokeStandardParameterConfig {
    pub(crate) default_limit: u32,
    pub(crate) project_config: Option<StandardParameterConfigSourceDescriptor>,
    pub(crate) user_config: Option<StandardParameterConfigSourceDescriptor>,
    pub(crate) native_options: Vec<crate::NativeOptionSpec>,
}

impl InvokeStandardParameterConfig {
    pub(crate) const fn new(default_limit: u32) -> Self {
        Self {
            default_limit,
            project_config: None,
            user_config: None,
            native_options: Vec::new(),
        }
    }
}

pub(super) fn loaded_config_source(
    descriptor: &StandardParameterConfigSourceDescriptor,
) -> Result<LoadedStandardParameterConfigSource, AdapterError> {
    let loaded = load_standard_parameter_config_source(descriptor);
    if let Some(issue) = adapter_config_source_issue(descriptor, loaded.value()) {
        return Ok(loaded.with_config_source_issue(issue));
    }
    Ok(loaded)
}

pub(crate) fn adapter_config_source_issue(
    descriptor: &StandardParameterConfigSourceDescriptor,
    value: Option<&JsonValue>,
) -> Option<StandardParameterConfigSourceIssue> {
    let root = value?.as_object()?;
    if let Some(key) = first_unknown_key(root, ROOT_KEYS) {
        return Some(config_field_issue(descriptor, key, "unknown_config_field"));
    }
    validate_defaults(descriptor, root).or_else(|| object_field(descriptor, root, "options").err())
}

fn validate_defaults(
    descriptor: &StandardParameterConfigSourceDescriptor,
    root: &Map<String, Value>,
) -> Option<StandardParameterConfigSourceIssue> {
    let defaults = match object_field(descriptor, root, "defaults") {
        Ok(Some(defaults)) => defaults,
        Ok(None) => return None,
        Err(issue) => return Some(issue),
    };
    if let Some(key) = first_unknown_key(defaults, DEFAULTS_KEYS) {
        return Some(config_field_issue(
            descriptor,
            format!("defaults.{key}"),
            "unknown_config_field",
        ));
    }
    validate_pagination(descriptor, defaults)
}

fn validate_pagination(
    descriptor: &StandardParameterConfigSourceDescriptor,
    defaults: &Map<String, Value>,
) -> Option<StandardParameterConfigSourceIssue> {
    let pagination = match object_field(descriptor, defaults, "defaults.pagination") {
        Ok(Some(pagination)) => pagination,
        Ok(None) => return None,
        Err(issue) => return Some(issue),
    };
    first_unknown_key(pagination, PAGINATION_KEYS).map(|key| {
        config_field_issue(
            descriptor,
            format!("defaults.pagination.{key}"),
            "unknown_config_field",
        )
    })
}

fn object_field<'a>(
    descriptor: &StandardParameterConfigSourceDescriptor,
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<Option<&'a Map<String, Value>>, StandardParameterConfigSourceIssue> {
    let key = field.rsplit('.').next().unwrap_or(field);
    match object.get(key) {
        Some(value) => value
            .as_object()
            .map(Some)
            .ok_or_else(|| config_field_issue(descriptor, field, "invalid_config_field")),
        None => Ok(None),
    }
}

fn first_unknown_key<'a>(object: &'a Map<String, Value>, allowed: &[&str]) -> Option<&'a str> {
    object
        .keys()
        .find(|key| !allowed.contains(&key.as_str()))
        .map(String::as_str)
}

fn config_field_issue(
    descriptor: &StandardParameterConfigSourceDescriptor,
    field: impl Into<String>,
    reason_code: &str,
) -> StandardParameterConfigSourceIssue {
    StandardParameterConfigSourceIssue::new(
        descriptor.level.as_str(),
        descriptor.origin.as_str(),
        descriptor.path.display().to_string(),
        reason_code,
    )
    .with_field(field)
}
