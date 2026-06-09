use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CliWarning {
    pub id: CliWarningId,
    pub reason: String,
    pub effect: CliWarningEffect,
    pub details: CliWarningDetails,
}

impl CliWarning {
    pub(super) fn unknown_flag(token: &str) -> Self {
        Self::cli_argv_ignored(vec![token.to_owned()], "unknown CLI flag ignored")
    }

    pub(super) fn extra_positional(token: &str) -> Self {
        Self::cli_argv_ignored(vec![token.to_owned()], "extra positional argument ignored")
    }

    pub(super) fn unused_operation_flag(flag: &str, value: Option<&str>, command: &str) -> Self {
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
            id: CliWarningId::AdapterCandidateFailure,
            reason,
            effect: CliWarningEffect::CandidateSkipped,
            details: CliWarningDetails::AdapterCandidate {
                adapter_id: adapter_id.to_owned(),
                stage: stage.to_owned(),
                code: code.to_owned(),
                preselected: if preselected { Some(true) } else { None },
            },
        }
    }

    fn cli_argv_ignored(tokens: Vec<String>, reason: impl Into<String>) -> Self {
        Self {
            id: CliWarningId::CliArgvIgnored,
            reason: reason.into(),
            effect: CliWarningEffect::OperationContinued,
            details: CliWarningDetails::CliArgv { tokens },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CliWarningId {
    CliArgvIgnored,
    AdapterCandidateFailure,
}

impl CliWarningId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CliArgvIgnored => "cli_argv_ignored",
            Self::AdapterCandidateFailure => "adapter_candidate_failure",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CliWarningEffect {
    OperationContinued,
    CandidateSkipped,
}

impl CliWarningEffect {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OperationContinued => "operation_continued",
            Self::CandidateSkipped => "candidate_skipped",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum CliWarningDetails {
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
}
