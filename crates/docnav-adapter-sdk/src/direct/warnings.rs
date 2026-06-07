use serde::Serialize;

// Warning kind 是 readable/MCP warning schema 的稳定取值。
mod warning_kinds {
    pub(super) const EXTRA_POSITIONAL: &str = "extra_positional";
    pub(super) const UNKNOWN_FLAG: &str = "unknown_flag";
    pub(super) const UNUSED_OPERATION_FLAG: &str = "unused_operation_flag";
}

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
    pub(super) ignored_tokens: Vec<String>,
    pub(super) kind: DirectCliWarningKind,
    pub(super) reason: String,
}

impl DirectCliWarning {
    pub(super) fn unknown_flag(token: &str) -> Self {
        Self {
            ignored_tokens: vec![token.to_owned()],
            kind: DirectCliWarningKind::UnknownFlag,
            reason: warning_reasons::UNKNOWN_FLAG_IGNORED.to_owned(),
        }
    }

    pub(super) fn extra_positional(token: &str) -> Self {
        Self {
            ignored_tokens: vec![token.to_owned()],
            kind: DirectCliWarningKind::ExtraPositional,
            reason: warning_reasons::EXTRA_POSITIONAL_IGNORED.to_owned(),
        }
    }

    pub(super) fn unused_operation_flag(flag: &str, value: Option<&str>, command: &str) -> Self {
        let mut ignored_tokens = vec![flag.to_owned()];
        if let Some(value) = value {
            ignored_tokens.push(value.to_owned());
        }
        Self {
            ignored_tokens,
            kind: DirectCliWarningKind::UnusedOperationFlag,
            reason: warning_reasons::unused_operation_flag(command),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum DirectCliWarningKind {
    UnknownFlag,
    ExtraPositional,
    UnusedOperationFlag,
}

impl DirectCliWarningKind {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownFlag => warning_kinds::UNKNOWN_FLAG,
            Self::ExtraPositional => warning_kinds::EXTRA_POSITIONAL,
            Self::UnusedOperationFlag => warning_kinds::UNUSED_OPERATION_FLAG,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warning_kind_strings_match_serialized_names() {
        assert_eq!(DirectCliWarningKind::UnknownFlag.as_str(), "unknown_flag");
        assert_eq!(
            DirectCliWarningKind::ExtraPositional.as_str(),
            "extra_positional"
        );
        assert_eq!(
            DirectCliWarningKind::UnusedOperationFlag.as_str(),
            "unused_operation_flag"
        );
    }

    #[test]
    fn warning_constructors_keep_tokens_and_reasons() {
        assert_eq!(
            DirectCliWarning::unknown_flag("--future"),
            DirectCliWarning {
                ignored_tokens: vec!["--future".to_owned()],
                kind: DirectCliWarningKind::UnknownFlag,
                reason: "unknown CLI flag ignored".to_owned(),
            }
        );
        assert_eq!(
            DirectCliWarning::extra_positional("extra").reason,
            "extra positional argument ignored"
        );
        assert_eq!(
            DirectCliWarning::unused_operation_flag("--max-heading-level", Some("3"), "read"),
            DirectCliWarning {
                ignored_tokens: vec!["--max-heading-level".to_owned(), "3".to_owned()],
                kind: DirectCliWarningKind::UnusedOperationFlag,
                reason: "flag is not used by read command".to_owned(),
            }
        );
    }
}
