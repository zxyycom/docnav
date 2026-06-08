use serde::Serialize;

use super::flags;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CliWarning {
    pub ignored_tokens: Vec<String>,
    pub kind: CliWarningKind,
    pub reason: String,
}

impl CliWarning {
    pub(super) fn unknown_flag(token: &str) -> Self {
        Self {
            ignored_tokens: vec![token.to_owned()],
            kind: CliWarningKind::UnknownFlag,
            reason: "unknown CLI flag ignored".to_owned(),
        }
    }

    pub(super) fn extra_positional(token: &str) -> Self {
        Self {
            ignored_tokens: vec![token.to_owned()],
            kind: CliWarningKind::ExtraPositional,
            reason: "extra positional argument ignored".to_owned(),
        }
    }

    pub(super) fn unused_operation_flag(flag: &str, value: Option<&str>, command: &str) -> Self {
        let mut ignored_tokens = vec![flag.to_owned()];
        if let Some(value) = value {
            ignored_tokens.push(value.to_owned());
        }
        Self {
            ignored_tokens,
            kind: CliWarningKind::UnusedOperationFlag,
            reason: format!("flag is not used by {command} command"),
        }
    }

    pub fn adapter_candidate_failure(adapter_id: &str, reason: &str) -> Self {
        Self {
            ignored_tokens: vec![flags::ADAPTER.to_owned(), adapter_id.to_owned()],
            kind: CliWarningKind::AdapterCandidateFailure,
            reason: format!("preselected adapter was not used: {reason}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CliWarningKind {
    UnknownFlag,
    ExtraPositional,
    UnusedOperationFlag,
    AdapterCandidateFailure,
}

impl CliWarningKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownFlag => "unknown_flag",
            Self::ExtraPositional => "extra_positional",
            Self::UnusedOperationFlag => "unused_operation_flag",
            Self::AdapterCandidateFailure => "adapter_candidate_failure",
        }
    }
}
