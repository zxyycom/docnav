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
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadableView => "readable-view",
            Self::ReadableJson => "readable-json",
            Self::ProtocolJson => "protocol-json",
        }
    }

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
    Init,
    Doctor,
    Version,
    Help(String),
}

impl CliCommand {
    pub const fn operation(&self) -> Option<Operation> {
        match self {
            Self::Document(command) => Some(command.operation),
            Self::Adapter(_)
            | Self::Config(_)
            | Self::Init
            | Self::Doctor
            | Self::Version
            | Self::Help(_) => None,
        }
    }

    pub const fn output_mode(&self) -> Option<OutputMode> {
        match self {
            Self::Document(command) => command.output,
            Self::Adapter(_)
            | Self::Config(_)
            | Self::Init
            | Self::Doctor
            | Self::Version
            | Self::Help(_) => None,
        }
    }
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
    pub max_heading_level: Option<PositiveInteger>,
    pub output: Option<OutputMode>,
    pub adapter: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterCommand {
    List,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigCommand {
    Get(ConfigGet),
    Set(ConfigSet),
    Unset(ConfigUnset),
    List(ConfigList),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigGet {
    pub key: String,
    pub user: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigSet {
    pub key: String,
    pub value: String,
    pub user: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigUnset {
    pub key: String,
    pub user: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigList {
    pub user: bool,
    pub path: Option<String>,
    pub operation: Option<Operation>,
}
