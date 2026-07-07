use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::project_context::ProjectContext;

pub(super) const DEFAULT_LIMIT: u32 = 6000;
pub(super) const DEFAULT_OUTPUT: &str = "readable-view";
pub(super) const DEFAULT_PAGINATION_ENABLED: bool = true;
pub(super) const SUPPORTED_CORE_KEYS: [&str; 8] = [
    "defaults.adapter",
    "defaults.pagination.enabled",
    "defaults.pagination.limit",
    "defaults.output",
    "invocation_log.enabled",
    "invocation_log.path",
    "invocation_log.content_capture.enabled",
    "invocation_log.content_capture.root",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigContext {
    pub project: ProjectContext,
    pub project_config: CoreConfig,
    pub user_config: CoreConfig,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreConfig {
    #[serde(default, skip_serializing_if = "DefaultsConfig::is_empty")]
    pub defaults: DefaultsConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outline: Option<Value>,
    #[serde(default, skip_serializing_if = "InvocationLogConfig::is_empty")]
    pub invocation_log: InvocationLogConfig,
    #[serde(default, skip_serializing_if = "NativeOptionsConfig::is_empty")]
    pub options: NativeOptionsConfig,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefaultsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter: Option<String>,
    #[serde(default, skip_serializing_if = "PaginationConfig::is_empty")]
    pub pagination: PaginationConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaginationConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationLogConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "ContentCaptureConfig::is_empty")]
    pub content_capture: ContentCaptureConfig,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentCaptureConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NativeOptionsConfig {
    #[serde(flatten)]
    values: BTreeMap<String, Value>,
}

impl DefaultsConfig {
    fn is_empty(&self) -> bool {
        self.adapter.is_none() && self.pagination.is_empty() && self.output.is_none()
    }
}

impl NativeOptionsConfig {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub(crate) fn value_for_key(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    pub(crate) fn insert(&mut self, key: impl Into<String>, value: Value) {
        self.values.insert(key.into(), value);
    }

    pub(crate) fn remove(&mut self, key: &str) {
        self.values.remove(key);
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(String::as_str)
    }
}

impl InvocationLogConfig {
    pub(crate) fn is_empty(&self) -> bool {
        self.enabled.is_none() && self.path.is_none() && self.content_capture.is_empty()
    }
}

impl ContentCaptureConfig {
    pub(crate) fn is_empty(&self) -> bool {
        self.enabled.is_none() && self.root.is_none()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn native_options_config_accepts_generic_raw_map() {
        let config: CoreConfig = serde_json::from_value(json!({
            "options": {
                "registered_elsewhere": {
                    "raw": true
                }
            }
        }))
        .expect("generic native option map");

        assert_eq!(
            config.options.value_for_key("registered_elsewhere"),
            Some(&json!({ "raw": true }))
        );
    }
}

impl PaginationConfig {
    fn is_empty(&self) -> bool {
        self.enabled.is_none() && self.limit.is_none()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ConfigSource {
    Explicit,
    Project,
    User,
    BuiltIn,
    Unset,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ResolvedValue {
    pub value: Value,
    pub source: String,
}

impl ResolvedValue {
    pub fn explicit(value: Value) -> Self {
        Self::new(value, ConfigSource::Explicit)
    }

    pub fn project(value: Value) -> Self {
        Self::new(value, ConfigSource::Project)
    }

    pub fn user(value: Value) -> Self {
        Self::new(value, ConfigSource::User)
    }

    pub fn built_in(value: Value) -> Self {
        Self::new(value, ConfigSource::BuiltIn)
    }

    pub fn unset() -> Self {
        Self::new(Value::Null, ConfigSource::Unset)
    }

    pub(super) fn new(value: Value, source: ConfigSource) -> Self {
        let source = serde_json::to_value(source)
            .ok()
            .and_then(|value| value.as_str().map(str::to_owned))
            .unwrap_or_else(|| "unknown".to_owned());
        Self { value, source }
    }
}
