use super::super::parse;
use crate::error::AppError;

// @case WB-CORE-ARGS-001
mod structural_errors;
mod values;

// @case WB-CORE-ARGS-REPAIR-001
mod protocol_errors;

fn assert_diagnostic(error: AppError, field: &str, reason_fragment: &str) {
    let details = error.diagnostic().details().to_value();
    assert_eq!(
        details.get("field").and_then(serde_json::Value::as_str),
        Some(field)
    );
    assert!(details
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|reason| reason.contains(reason_fragment)));
}

fn candidate<'a>(
    command: &'a crate::cli::DocumentCommand,
    identity: &str,
) -> &'a cli_config_resolution::SourceCandidate {
    command
        .cli_source
        .candidates()
        .iter()
        .find(|candidate| candidate.field().as_str() == identity)
        .unwrap_or_else(|| panic!("missing candidate {identity}"))
}
