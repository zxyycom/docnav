use std::fmt;
use std::io::{self, Write};

use serde::Serialize;
use serde_json::{Map, Value};

mod code;
mod details;
mod record;
mod stack;

pub use code::{
    BoundaryDiagnosticCode, DiagnosticCategory, DiagnosticCode, DiagnosticEffect,
    DiagnosticProjectionRule, DiagnosticSeverity, ProtocolDiagnosticCode,
    ReadableWarningDiagnosticCode,
};
pub use details::{
    DetailFieldRule, DetailFieldType, DiagnosticDetails, DiagnosticDetailsError,
    DiagnosticDetailsRule,
};
pub use record::{
    DiagnosticRecord, DiagnosticRecordDraft, DiagnosticRecordError, DiagnosticSource,
};
pub use stack::{DiagnosticId, DiagnosticMark, DiagnosticStack};

pub const CLI_ARGV_IGNORED: WarningId =
    WarningId::from_static(ReadableWarningDiagnosticCode::CliArgvIgnored.warning_id());
pub const ADAPTER_CANDIDATE_FAILURE: WarningId =
    WarningId::from_static(ReadableWarningDiagnosticCode::AdapterCandidateFailure.warning_id());
pub const ADAPTER_CONFIG_SOURCE_SKIPPED: WarningId =
    WarningId::from_static(ReadableWarningDiagnosticCode::AdapterConfigSourceSkipped.warning_id());

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WarningId(WarningIdRepr);

impl WarningId {
    pub const fn from_static(value: &'static str) -> Self {
        Self(WarningIdRepr::Static(value))
    }

    pub fn new(value: impl Into<String>) -> Result<Self, InvalidWarningId> {
        let value = value.into();
        validate_warning_id(&value)?;
        Ok(Self(WarningIdRepr::Owned(value)))
    }

    pub const fn cli_argv_ignored() -> Self {
        CLI_ARGV_IGNORED
    }

    pub const fn adapter_candidate_failure() -> Self {
        ADAPTER_CANDIDATE_FAILURE
    }

    pub const fn adapter_config_source_skipped() -> Self {
        ADAPTER_CONFIG_SOURCE_SKIPPED
    }

    pub fn as_str(&self) -> &str {
        match &self.0 {
            WarningIdRepr::Static(value) => value,
            WarningIdRepr::Owned(value) => value,
        }
    }
}

impl Serialize for WarningId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum WarningIdRepr {
    Static(&'static str),
    Owned(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidWarningId {
    value: String,
}

impl fmt::Display for InvalidWarningId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid warning id {:?}", self.value)
    }
}

impl std::error::Error for InvalidWarningId {}

fn validate_warning_id(value: &str) -> Result<(), InvalidWarningId> {
    let valid = !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
    if valid {
        Ok(())
    } else {
        Err(InvalidWarningId {
            value: value.to_owned(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningEffect {
    OperationContinued,
    CandidateSkipped,
}

impl WarningEffect {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OperationContinued => "operation_continued",
            Self::CandidateSkipped => "candidate_skipped",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum WarningDetails {
    CliArgv {
        tokens: Vec<String>,
    },
    AdapterCandidate {
        adapter_id: String,
        stage: String,
        code: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        preselected: Option<bool>,
    },
    AdapterConfigSource {
        source_level: String,
        path_origin: String,
        path: String,
        reason_code: String,
    },
    Other(Map<String, Value>),
}

impl WarningDetails {
    pub fn cli_argv_tokens(&self) -> Option<&[String]> {
        match self {
            Self::CliArgv { tokens } => Some(tokens),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Warning {
    pub id: WarningId,
    pub reason: String,
    pub effect: WarningEffect,
    pub details: WarningDetails,
}

impl Warning {
    pub fn new(
        id: WarningId,
        reason: impl Into<String>,
        effect: WarningEffect,
        details: WarningDetails,
    ) -> Result<Self, EmptyWarningReason> {
        let reason = reason.into();
        if reason.is_empty() {
            return Err(EmptyWarningReason);
        }
        Ok(Self {
            id,
            reason,
            effect,
            details,
        })
    }

    pub fn unknown_flag(token: &str) -> Self {
        Self::cli_argv_ignored(vec![token.to_owned()], "unknown CLI flag ignored")
    }

    pub fn extra_positional(token: &str) -> Self {
        Self::cli_argv_ignored(vec![token.to_owned()], "extra positional argument ignored")
    }

    pub fn unused_operation_flag(flag: &str, value: Option<&str>, command: &str) -> Self {
        let mut tokens = vec![flag.to_owned()];
        if let Some(value) = value {
            tokens.push(value.to_owned());
        }
        Self::cli_argv_ignored(tokens, format!("flag is not used by {command} command"))
    }

    pub fn adapter_candidate_failure(
        adapter_id: &str,
        stage: &str,
        code: &str,
        reason: &str,
        preselected: bool,
    ) -> Self {
        let reason = if preselected {
            format!("preselected adapter was not used: {reason}")
        } else {
            format!("adapter candidate was not used: {reason}")
        };
        Self {
            id: WarningId::adapter_candidate_failure(),
            reason,
            effect: WarningEffect::CandidateSkipped,
            details: WarningDetails::AdapterCandidate {
                adapter_id: adapter_id.to_owned(),
                stage: stage.to_owned(),
                code: code.to_owned(),
                preselected: if preselected { Some(true) } else { None },
            },
        }
    }

    pub fn adapter_config_source_skipped(
        source_level: &str,
        path_origin: &str,
        path: &str,
        reason_code: &str,
    ) -> Self {
        Self {
            id: WarningId::adapter_config_source_skipped(),
            reason: "adapter config source skipped".to_owned(),
            effect: WarningEffect::OperationContinued,
            details: WarningDetails::AdapterConfigSource {
                source_level: source_level.to_owned(),
                path_origin: path_origin.to_owned(),
                path: path.to_owned(),
                reason_code: reason_code.to_owned(),
            },
        }
    }

    pub fn cli_argv_ignored(tokens: Vec<String>, reason: impl Into<String>) -> Self {
        Self {
            id: WarningId::cli_argv_ignored(),
            reason: reason.into(),
            effect: WarningEffect::OperationContinued,
            details: WarningDetails::CliArgv { tokens },
        }
    }

    pub fn diagnostic_code(&self) -> Option<ReadableWarningDiagnosticCode> {
        match self.id.as_str() {
            id if id == ReadableWarningDiagnosticCode::CliArgvIgnored.warning_id() => {
                Some(ReadableWarningDiagnosticCode::CliArgvIgnored)
            }
            id if id == ReadableWarningDiagnosticCode::AdapterCandidateFailure.warning_id() => {
                Some(ReadableWarningDiagnosticCode::AdapterCandidateFailure)
            }
            id if id == ReadableWarningDiagnosticCode::AdapterConfigSourceSkipped.warning_id() => {
                Some(ReadableWarningDiagnosticCode::AdapterConfigSourceSkipped)
            }
            _ => None,
        }
    }

    pub fn to_record_draft(&self, source: DiagnosticSource) -> Option<DiagnosticRecordDraft> {
        let code = self.diagnostic_code()?;
        let details = match &self.details {
            WarningDetails::CliArgv { tokens } => DiagnosticDetails::CliArgv {
                tokens: tokens.clone(),
            },
            WarningDetails::AdapterCandidate {
                adapter_id,
                stage,
                code,
                preselected,
            } => DiagnosticDetails::AdapterCandidate {
                adapter_id: adapter_id.clone(),
                stage: stage.clone(),
                code: code.clone(),
                preselected: *preselected,
            },
            WarningDetails::AdapterConfigSource {
                source_level,
                path_origin,
                path,
                reason_code,
            } => DiagnosticDetails::AdapterConfigSource {
                source_level: source_level.clone(),
                path_origin: path_origin.clone(),
                path: path.clone(),
                reason_code: reason_code.clone(),
            },
            WarningDetails::Other(_) => return None,
        };
        Some(DiagnosticRecordDraft::new(
            code,
            self.reason.clone(),
            details,
            source,
        ))
    }

    pub fn from_record(record: &DiagnosticRecord) -> Option<Self> {
        match (record.code, &record.details) {
            (
                DiagnosticCode::ReadableWarning(ReadableWarningDiagnosticCode::CliArgvIgnored),
                DiagnosticDetails::CliArgv { tokens },
            ) => Some(Self::cli_argv_ignored(
                tokens.clone(),
                record.summary.clone(),
            )),
            (
                DiagnosticCode::ReadableWarning(
                    ReadableWarningDiagnosticCode::AdapterCandidateFailure,
                ),
                DiagnosticDetails::AdapterCandidate {
                    adapter_id,
                    stage,
                    code,
                    preselected,
                },
            ) => Some(Self {
                id: WarningId::adapter_candidate_failure(),
                reason: record.summary.clone(),
                effect: WarningEffect::CandidateSkipped,
                details: WarningDetails::AdapterCandidate {
                    adapter_id: adapter_id.clone(),
                    stage: stage.clone(),
                    code: code.clone(),
                    preselected: *preselected,
                },
            }),
            (
                DiagnosticCode::ReadableWarning(
                    ReadableWarningDiagnosticCode::AdapterConfigSourceSkipped,
                ),
                DiagnosticDetails::AdapterConfigSource {
                    source_level,
                    path_origin,
                    path,
                    reason_code,
                },
            ) => Some(Self {
                id: WarningId::adapter_config_source_skipped(),
                reason: record.summary.clone(),
                effect: WarningEffect::OperationContinued,
                details: WarningDetails::AdapterConfigSource {
                    source_level: source_level.clone(),
                    path_origin: path_origin.clone(),
                    path: path.clone(),
                    reason_code: reason_code.clone(),
                },
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmptyWarningReason;

impl fmt::Display for EmptyWarningReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("warning reason must not be empty")
    }
}

impl std::error::Error for EmptyWarningReason {}

pub fn warning_text_line(warning: &Warning) -> Result<String, serde_json::Error> {
    let details = serde_json::to_string(&warning.details)?;
    Ok(format!(
        "warning: id={}, effect={}, reason={}, details={}",
        warning.id.as_str(),
        warning.effect.as_str(),
        warning.reason.replace(['\r', '\n'], " "),
        details
    ))
}

pub fn attach_warnings_to_value<T: Serialize>(mut value: Value, warnings: &[T]) -> Value {
    if warnings.is_empty() {
        return value;
    }

    let warnings = serde_json::to_value(warnings).unwrap_or_else(|_| Value::Array(Vec::new()));
    match &mut value {
        Value::Object(object) => {
            object.insert("warnings".to_owned(), warnings);
            value
        }
        _ => {
            let mut object = Map::new();
            object.insert("value".to_owned(), value);
            object.insert("warnings".to_owned(), warnings);
            Value::Object(object)
        }
    }
}

pub fn write_warning_text_lines<W: Write>(warnings: &[Warning], writer: &mut W) -> io::Result<()> {
    for warning in warnings {
        writeln!(
            writer,
            "{}",
            warning_text_line(warning).map_err(io::Error::other)?
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
