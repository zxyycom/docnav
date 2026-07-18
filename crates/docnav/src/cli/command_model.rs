use std::str::FromStr;

use cli_config_resolution::{CandidateInput, Source};
use docnav_protocol::Operation;

pub(crate) const DOCUMENT_OUTPUT_FIELD_ID: &str = crate::parameter_catalog::OUTPUT_IDENTITY;
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
    /// Full protocol response envelope (stable machine format).
    ProtocolJson,
}

impl OutputMode {
    /// Currently accepted output values for document --output.
    pub const ACCEPTED_VALUES: &'static [&'static str] = &["readable-view", "protocol-json"];
}

impl FromStr for OutputMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
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

#[derive(Clone, Debug, PartialEq)]
pub struct ParsedCli {
    pub command: CliCommand,
}

impl ParsedCli {
    pub const fn new(command: CliCommand) -> Self {
        Self { command }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

    pub fn output_mode(&self) -> Option<OutputMode> {
        match self {
            Self::Document(command) => command.output_mode(),
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

#[derive(Clone, Debug, PartialEq)]
pub struct DocumentCommand {
    pub operation: Operation,
    pub path: String,
    pub ref_id: Option<String>,
    pub query: Option<String>,
    pub cli_source: Box<Source>,
    pub invocation_log: Option<String>,
    pub invocation_log_content_root: Option<String>,
    pub config_paths: ConfigPathArgs,
}

impl DocumentCommand {
    pub(crate) fn cli_candidate(
        &self,
        identity: &str,
    ) -> Option<&cli_config_resolution::SourceCandidate> {
        self.cli_source
            .candidates()
            .iter()
            .find(|candidate| candidate.field().as_str() == identity)
    }

    pub(crate) fn output_mode(&self) -> Option<OutputMode> {
        self.cli_candidate(DOCUMENT_OUTPUT_FIELD_ID)
            .and_then(|candidate| match candidate.input() {
                CandidateInput::Value(serde_json::Value::String(value)) => value.parse().ok(),
                CandidateInput::Value(_) | CandidateInput::Invalid { .. } => None,
            })
    }

    pub(crate) fn has_output_candidate(&self) -> bool {
        self.cli_candidate(DOCUMENT_OUTPUT_FIELD_ID).is_some()
    }
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
