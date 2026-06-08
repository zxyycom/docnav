use serde::Serialize;

use super::flags;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CliWarning {
    pub ignored_tokens: Vec<String>,
    pub kind: CliWarningKind,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl CliWarning {
    pub(super) fn unknown_flag(token: &str) -> Self {
        Self {
            ignored_tokens: vec![token.to_owned()],
            kind: CliWarningKind::UnknownFlag,
            reason: "unknown CLI flag ignored".to_owned(),
            adapter_id: None,
            stage: None,
            code: None,
        }
    }

    pub(super) fn extra_positional(token: &str) -> Self {
        Self {
            ignored_tokens: vec![token.to_owned()],
            kind: CliWarningKind::ExtraPositional,
            reason: "extra positional argument ignored".to_owned(),
            adapter_id: None,
            stage: None,
            code: None,
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
            adapter_id: None,
            stage: None,
            code: None,
        }
    }

    pub fn adapter_candidate_failure(
        adapter_id: &str,
        stage: &str,
        code: &str,
        reason: &str,
        preselected: bool,
    ) -> Self {
        let ignored_tokens = if preselected {
            vec![flags::ADAPTER.to_owned(), adapter_id.to_owned()]
        } else {
            Vec::new()
        };
        let reason = if preselected {
            format!("preselected adapter was not used: {reason}")
        } else {
            format!("adapter candidate was not used: {reason}")
        };
        Self {
            ignored_tokens,
            kind: CliWarningKind::AdapterCandidateFailure,
            reason,
            adapter_id: Some(adapter_id.to_owned()),
            stage: Some(stage.to_owned()),
            code: Some(code.to_owned()),
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
