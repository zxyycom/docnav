use std::path::PathBuf;

use cli_config_resolution::Source;
use docnav_protocol::{Operation, ProtocolResponse};
use serde_json::Value;

use crate::{config_source::LoadedNavigationConfigSource, context::NavigationContextSelection};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationOutputMode {
    ReadableView,
    ProtocolJson,
}

impl NavigationOutputMode {
    pub const ACCEPTED_VALUES: &'static [&'static str] = &["readable-view", "protocol-json"];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadableView => "readable-view",
            Self::ProtocolJson => "protocol-json",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "readable-view" => Ok(Self::ReadableView),
            "protocol-json" => Ok(Self::ProtocolJson),
            _ => Err(format!(
                "invalid output value {value:?}, accepted values: {}",
                Self::ACCEPTED_VALUES.join(", ")
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutoReadMode {
    Disabled,
    UniqueRef,
}

impl AutoReadMode {
    pub const ACCEPTED_VALUES: &'static [&'static str] = &["disabled", "unique-ref"];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::UniqueRef => "unique-ref",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "disabled" => Ok(Self::Disabled),
            "unique-ref" => Ok(Self::UniqueRef),
            _ => Err(format!(
                "invalid auto-read value {value:?}, accepted values: {}",
                Self::ACCEPTED_VALUES.join(", ")
            )),
        }
    }
}

/// Canonical identity used for invocation-local document CLI candidates.
pub const DOCUMENT_CLI_SOURCE_ID: &str = "explicit";
/// Source priority for invocation-local document CLI candidates.
pub const DOCUMENT_CLI_SOURCE_PRIORITY: i32 = 400;

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationCommand {
    pub operation: Operation,
    pub document_path: String,
    pub ref_id: Option<String>,
    pub query: Option<String>,
    pub cli_source: Source,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationConfigSourceOrigin {
    Default,
    ExplicitCli,
    Override,
}

impl NavigationConfigSourceOrigin {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::ExplicitCli => "explicit_cli",
            Self::Override => "override",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationConfigSourceLevel {
    Project,
    User,
}

impl NavigationConfigSourceLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::User => "user",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationConfigSourceDescriptor {
    pub origin: NavigationConfigSourceOrigin,
    pub path: PathBuf,
}

impl NavigationConfigSourceDescriptor {
    pub fn new(origin: NavigationConfigSourceOrigin, path: PathBuf) -> Self {
        Self { origin, path }
    }

    pub fn default(path: PathBuf) -> Self {
        Self::new(NavigationConfigSourceOrigin::Default, path)
    }

    pub fn explicit_cli(path: PathBuf) -> Self {
        Self::new(NavigationConfigSourceOrigin::ExplicitCli, path)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationConfigSourceDescriptors {
    pub project: NavigationConfigSourceDescriptor,
    pub user: NavigationConfigSourceDescriptor,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NavigationConfigSource {
    pub(crate) level: NavigationConfigSourceLevel,
    pub(crate) origin: NavigationConfigSourceOrigin,
    pub(crate) path: String,
    pub(crate) loaded: LoadedNavigationConfigSource,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NavigationConfigSources {
    pub(crate) project: NavigationConfigSource,
    pub(crate) user: NavigationConfigSource,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationCommandOutcome {
    pub response: ProtocolResponse,
    pub output: NavigationOutputMode,
    pub trace: NavigationInvocationTrace,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationInvocationTrace {
    pub operation: Operation,
    pub selected_adapter_id: Option<String>,
    pub request_id: Option<String>,
    pub failure_layer: Option<NavigationFailureLayer>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationFailureLayer {
    Config,
    AdapterSelection,
    RequestConstruction,
    AdapterDispatch,
    ResultValidation,
}

impl NavigationFailureLayer {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::AdapterSelection => "adapter_selection",
            Self::RequestConstruction => "request_construction",
            Self::AdapterDispatch => "adapter_dispatch",
            Self::ResultValidation => "result_validation",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationContextOutcome {
    pub selection: NavigationContextSelection,
    pub defaults: NavigationContextDefaults,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationContextDefaults {
    pub adapter: NavigationResolvedValue,
    pub pagination: Option<NavigationPaginationDefaults>,
    pub output: NavigationResolvedValue,
    pub page: Option<NavigationResolvedValue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationPaginationDefaults {
    pub enabled: NavigationResolvedValue,
    pub limit: NavigationResolvedValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationResolvedValue {
    pub value: Value,
    pub source: String,
}

impl NavigationResolvedValue {
    pub fn new(value: Value, source: impl Into<String>) -> Self {
        Self {
            value,
            source: source.into(),
        }
    }
}
