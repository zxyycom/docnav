mod adapter_source;
mod auto_read_composition;
mod auto_read_config;
mod config_sources;
mod hard_cutover;
mod native_options;
mod outline_mode;

use serde_json::Value;

use super::support::diagnostic_record;

fn protocol_error(
    diagnostic: &docnav_diagnostics::DiagnosticRecordDraft,
) -> docnav_protocol::ProtocolError {
    let record = diagnostic_record(diagnostic);
    docnav_protocol::ProtocolError::from_diagnostic_record(&record).expect("protocol projection")
}

fn first_option_issue_source(error: &docnav_protocol::ProtocolError) -> Option<&str> {
    error
        .details()
        .get("option_issues")
        .and_then(Value::as_array)
        .and_then(|issues| issues.first())
        .and_then(|issue| issue.get("source"))
        .and_then(Value::as_str)
}
