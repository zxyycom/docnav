pub(super) use docnav_diagnostics::Warning as DirectCliWarning;

#[cfg(test)]
pub(super) use docnav_diagnostics::{
    WarningDetails as DirectCliWarningDetails, WarningEffect as DirectCliWarningEffect,
    CLI_ARGV_IGNORED,
};

#[cfg(test)]
mod tests {
    // @case WB-SDK-DIRECT-WARN-001
    use super::*;

    #[test]
    fn warning_id_strings_match_serialized_names() {
        assert_eq!(CLI_ARGV_IGNORED.as_str(), "cli_argv_ignored");
    }

    #[test]
    fn warning_constructors_keep_tokens_and_reasons() {
        assert_eq!(
            DirectCliWarning::unknown_flag("--future"),
            DirectCliWarning {
                id: CLI_ARGV_IGNORED,
                reason: "unknown CLI flag ignored".to_owned(),
                effect: DirectCliWarningEffect::OperationContinued,
                details: DirectCliWarningDetails::CliArgv {
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
                id: CLI_ARGV_IGNORED,
                reason: "flag is not used by read command".to_owned(),
                effect: DirectCliWarningEffect::OperationContinued,
                details: DirectCliWarningDetails::CliArgv {
                    tokens: vec!["--max-heading-level".to_owned(), "3".to_owned()],
                },
            }
        );
    }
}
