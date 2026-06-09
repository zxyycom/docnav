use std::str::FromStr;

use docnav_protocol::{Operation, PositiveInteger};
use serde::{Deserialize, Serialize};

use crate::cli::CliWarning;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputMode {
    Text,
    ReadableJson,
    ProtocolJson,
}

impl OutputMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::ReadableJson => "readable-json",
            Self::ProtocolJson => "protocol-json",
        }
    }
}

impl FromStr for OutputMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "text" => Ok(Self::Text),
            "readable-json" => Ok(Self::ReadableJson),
            "protocol-json" => Ok(Self::ProtocolJson),
            _ => Err(format!("invalid --output {value:?}")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedCli {
    pub command: CliCommand,
    pub warnings: Vec<CliWarning>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliCommand {
    Document(DocumentCommand),
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
            Self::Config(_) | Self::Init | Self::Doctor | Self::Version | Self::Help(_) => None,
        }
    }

    pub const fn output_mode(&self) -> Option<OutputMode> {
        match self {
            Self::Document(command) => command.output,
            Self::Config(_) | Self::Init | Self::Doctor | Self::Version | Self::Help(_) => None,
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
    pub limit_chars: Option<PositiveInteger>,
    pub output: Option<OutputMode>,
    pub adapter: Option<String>,
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
