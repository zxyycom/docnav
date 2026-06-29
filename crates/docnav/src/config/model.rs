use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::project_context::ProjectContext;

pub(super) const DEFAULT_LIMIT: u32 = 6000;
pub(super) const DEFAULT_OUTPUT: &str = "readable-view";
pub(super) const SUPPORTED_KEYS: [&str; 3] =
    ["defaults.adapter", "defaults.limit", "defaults.output"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigContext {
    pub project: ProjectContext,
    pub project_config: CoreConfig,
    pub user_config: CoreConfig,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CoreConfig {
    #[serde(default, skip_serializing_if = "DefaultsConfig::is_empty")]
    pub defaults: DefaultsConfig,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct DefaultsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

impl DefaultsConfig {
    fn is_empty(&self) -> bool {
        self.adapter.is_none() && self.limit.is_none() && self.output.is_none()
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
