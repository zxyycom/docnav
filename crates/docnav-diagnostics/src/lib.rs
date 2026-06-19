use std::fmt;
use std::io::{self, Write};

use serde::Serialize;
use serde_json::{Map, Value};

pub const CLI_ARGV_IGNORED: WarningId = WarningId::from_static("cli_argv_ignored");
pub const ADAPTER_CANDIDATE_FAILURE: WarningId =
    WarningId::from_static("adapter_candidate_failure");

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

    pub fn cli_argv_ignored(tokens: Vec<String>, reason: impl Into<String>) -> Self {
        Self {
            id: WarningId::cli_argv_ignored(),
            reason: reason.into(),
            effect: WarningEffect::OperationContinued,
            details: WarningDetails::CliArgv { tokens },
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
mod tests {
    // @case WB-DIAG-WARN-001
    use super::*;
    use serde_json::json;

    #[test]
    fn warning_ids_serialize_as_stable_strings() {
        assert_eq!(CLI_ARGV_IGNORED.as_str(), "cli_argv_ignored");
        assert_eq!(
            ADAPTER_CANDIDATE_FAILURE.as_str(),
            "adapter_candidate_failure"
        );
        assert_eq!(
            serde_json::to_value(WarningId::new("adapter_owned").unwrap()).unwrap(),
            json!("adapter_owned")
        );
        assert!(WarningId::new("AdapterOwned").is_err());
    }

    #[test]
    fn argv_warning_constructors_keep_existing_shape() {
        let warning = Warning::unused_operation_flag("--page", Some("nope"), "info");
        assert_eq!(warning.id, CLI_ARGV_IGNORED);
        assert_eq!(warning.effect, WarningEffect::OperationContinued);
        assert_eq!(warning.reason, "flag is not used by info command");
        assert_eq!(
            serde_json::to_value(warning.details).unwrap(),
            json!({"tokens": ["--page", "nope"]})
        );
    }

    #[test]
    fn adapter_candidate_warning_keeps_existing_shape() {
        let warning = Warning::adapter_candidate_failure(
            "markdown",
            "probe",
            "UNSUPPORTED",
            "no match",
            true,
        );
        assert_eq!(warning.id, ADAPTER_CANDIDATE_FAILURE);
        assert_eq!(warning.effect, WarningEffect::CandidateSkipped);
        assert_eq!(
            serde_json::to_value(warning.details).unwrap(),
            json!({
                "adapter_id": "markdown",
                "stage": "probe",
                "code": "UNSUPPORTED",
                "preselected": true
            })
        );
    }

    #[test]
    fn warning_text_line_matches_stderr_contract() {
        let warning =
            Warning::cli_argv_ignored(vec!["--future".to_owned()], "unknown\nCLI flag ignored");

        assert_eq!(
            warning_text_line(&warning).unwrap(),
            "warning: id=cli_argv_ignored, effect=operation_continued, reason=unknown CLI flag ignored, details={\"tokens\":[\"--future\"]}"
        );
    }
}
