use std::str::FromStr;

use docnav_protocol::{Operation, PositiveInteger};
use serde::{Deserialize, Serialize};

/// Document output mode.
///
/// Only valid for document operations (outline, read, find, info).
/// Non-document commands (help, version, config, init, doctor) use a
/// separate `PlainText` channel that is NOT an `OutputMode` variant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputMode {
    /// Human/AI-readable view with JSON header and block-framed content.
    ReadableView,
    /// Structured JSON output without protocol envelope (documented shape).
    ReadableJson,
    /// Full protocol response envelope (stable machine format).
    ProtocolJson,
}

impl OutputMode {
    /// Currently accepted output values for document --output.
    pub const ACCEPTED_VALUES: &'static [&'static str] =
        &["readable-view", "readable-json", "protocol-json"];
}

impl FromStr for OutputMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "readable-view" => Ok(Self::ReadableView),
            "readable-json" => Ok(Self::ReadableJson),
            "protocol-json" => Ok(Self::ProtocolJson),
            _ => Err(format!(
                "invalid output value {value:?}, accepted values: {}",
                Self::ACCEPTED_VALUES.join(", ")
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParsedCli {
    pub command: CliCommand,
}

impl ParsedCli {
    pub const fn new(command: CliCommand) -> Self {
        Self { command }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliCommand {
    Document(DocumentCommand),
    Adapter(AdapterCommand),
    Config(ConfigCommand),
    Init(ConfigPathArgs),
    Doctor(ConfigPathArgs),
    Version,
    Help(String),
}

impl CliCommand {
    pub const fn operation(&self) -> Option<Operation> {
        match self {
            Self::Document(command) => Some(command.operation),
            Self::Adapter(_)
            | Self::Config(_)
            | Self::Init(_)
            | Self::Doctor(_)
            | Self::Version
            | Self::Help(_) => None,
        }
    }

    pub const fn output_mode(&self) -> Option<OutputMode> {
        match self {
            Self::Document(command) => command.output,
            Self::Adapter(_)
            | Self::Config(_)
            | Self::Init(_)
            | Self::Doctor(_)
            | Self::Version
            | Self::Help(_) => None,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigPathArgs {
    pub project_config: Option<String>,
    pub user_config: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentCommand {
    pub operation: Operation,
    pub path: String,
    pub ref_id: Option<String>,
    pub query: Option<String>,
    pub page: Option<PositiveInteger>,
    pub pagination_enabled: Option<bool>,
    pub limit: Option<PositiveInteger>,
    pub native_options: Vec<NativeOptionCliInput>,
    pub output: Option<OutputMode>,
    pub adapter: Option<String>,
    pub invocation_log: Option<String>,
    pub invocation_log_content_root: Option<String>,
    pub config_paths: ConfigPathArgs,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeOptionCliInput {
    pub flag: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterCommand {
    List,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigCommand {
    Inspect(ConfigInspect),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigInspect {
    pub config_paths: ConfigPathArgs,
}
