use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::project_context::ProjectContext;

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

    #[cfg(test)]
    pub(crate) fn value_for_key(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
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

impl PaginationConfig {
    fn is_empty(&self) -> bool {
        self.enabled.is_none() && self.limit.is_none()
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
