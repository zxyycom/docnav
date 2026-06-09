use serde::Serialize;

// Warning reason 是用户可见诊断文本，集中后避免 parser/output 测试漂移。
mod warning_reasons {
    pub(super) const EXTRA_POSITIONAL_IGNORED: &str = "extra positional argument ignored";
    pub(super) const UNKNOWN_FLAG_IGNORED: &str = "unknown CLI flag ignored";

    pub(super) fn unused_operation_flag(command: &str) -> String {
        format!("flag is not used by {command} command")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct DirectCliWarning {
    pub(super) id: DirectCliWarningId,
    pub(super) reason: String,
    pub(super) effect: DirectCliWarningEffect,
    pub(super) details: DirectCliWarningDetails,
}

impl DirectCliWarning {
    pub(super) fn unknown_flag(token: &str) -> Self {
        Self::cli_argv_ignored(
            vec![token.to_owned()],
            warning_reasons::UNKNOWN_FLAG_IGNORED,
        )
    }

    pub(super) fn extra_positional(token: &str) -> Self {
        Self::cli_argv_ignored(
            vec![token.to_owned()],
            warning_reasons::EXTRA_POSITIONAL_IGNORED,
        )
    }

    pub(super) fn unused_operation_flag(flag: &str, value: Option<&str>, command: &str) -> Self {
        let mut tokens = vec![flag.to_owned()];
        if let Some(value) = value {
            tokens.push(value.to_owned());
        }
        Self::cli_argv_ignored(tokens, warning_reasons::unused_operation_flag(command))
    }

    fn cli_argv_ignored(tokens: Vec<String>, reason: impl Into<String>) -> Self {
        Self {
            id: DirectCliWarningId::CliArgvIgnored,
            reason: reason.into(),
            effect: DirectCliWarningEffect::OperationContinued,
            details: DirectCliWarningDetails { tokens },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum DirectCliWarningId {
    CliArgvIgnored,
}

impl DirectCliWarningId {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::CliArgvIgnored => "cli_argv_ignored",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum DirectCliWarningEffect {
    OperationContinued,
}

impl DirectCliWarningEffect {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::OperationContinued => "operation_continued",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct DirectCliWarningDetails {
    pub(super) tokens: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warning_id_strings_match_serialized_names() {
        assert_eq!(
            DirectCliWarningId::CliArgvIgnored.as_str(),
            "cli_argv_ignored"
        );
    }

    #[test]
    fn warning_constructors_keep_tokens_and_reasons() {
        assert_eq!(
            DirectCliWarning::unknown_flag("--future"),
            DirectCliWarning {
                id: DirectCliWarningId::CliArgvIgnored,
                reason: "unknown CLI flag ignored".to_owned(),
                effect: DirectCliWarningEffect::OperationContinued,
                details: DirectCliWarningDetails {
                    tokens: vec!["--future".to_owned()],
                },
            }
        );
        assert_eq!(
            DirectCliWarning::extra_positional("extra").reason,
            "extra positional argument ignored"
        );
        assert_eq!(
            DirectCliWarning::unused_operation_flag("--max-heading-level", Some("3"), "read"),
            DirectCliWarning {
                id: DirectCliWarningId::CliArgvIgnored,
                reason: "flag is not used by read command".to_owned(),
                effect: DirectCliWarningEffect::OperationContinued,
                details: DirectCliWarningDetails {
                    tokens: vec!["--max-heading-level".to_owned(), "3".to_owned()],
                },
            }
        );
    }
}
