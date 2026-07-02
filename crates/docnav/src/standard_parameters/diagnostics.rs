use docnav_diagnostics::{
    typed_codes, DiagnosticRecordDraft, DiagnosticSource, FieldReasonDetails,
};
use docnav_standard_parameters::{
    ids, StandardParameterConfigSourceIssue, StandardParameterHandoff, StandardParameterResolution,
    StandardParameterValidationIssue,
};

use crate::cli::OutputMode;
use crate::error::{AppError, AppResult};

pub(super) fn first_validation_error(resolution: &StandardParameterResolution) -> AppResult<()> {
    if let Some(diagnostic) = resolution.diagnostics().first() {
        return Err(match diagnostic {
            StandardParameterHandoff::Validation(diagnostic) => validation_error(diagnostic),
            StandardParameterHandoff::ConfigSource(issue) => config_source_error(issue),
        });
    }
    Ok(())
}

fn validation_error(diagnostic: &StandardParameterValidationIssue) -> AppError {
    validation_error_for_identity(diagnostic.identity.as_str())
}

fn config_source_error(issue: &StandardParameterConfigSourceIssue) -> AppError {
    AppError::new(issue.to_record_draft(DiagnosticSource::with_stage(
        "docnav",
        "standard-parameters",
    )))
}

fn field_label(identity: &str) -> &'static str {
    match identity {
        ids::ADAPTER => "--adapter",
        ids::LIMIT => "--limit",
        ids::OUTPUT => "--output",
        ids::PAGE => "--page",
        ids::PAGINATION_ENABLED => "--pagination",
        ids::PATH => "path",
        ids::QUERY => "--query",
        ids::REF => "--ref",
        _ => "parameter",
    }
}

fn validation_reason(identity: &str) -> String {
    match identity {
        ids::LIMIT => "--limit must be a positive integer".to_owned(),
        ids::OUTPUT => format!(
            "invalid --output: accepted values: {}",
            OutputMode::ACCEPTED_VALUES.join(", ")
        ),
        ids::PAGE => "--page must be a positive integer".to_owned(),
        ids::PAGINATION_ENABLED => "--pagination must be enabled or disabled".to_owned(),
        ids::PATH => "path value must not be empty".to_owned(),
        ids::QUERY => "--query value must not be empty".to_owned(),
        ids::REF => "--ref value must not be empty".to_owned(),
        ids::ADAPTER => "--adapter value must not be empty".to_owned(),
        _ => "standard parameter validation failed".to_owned(),
    }
}

pub(super) fn validation_error_for_identity(identity: &str) -> AppError {
    AppError::new(invalid_request_record(
        field_label(identity),
        validation_reason(identity),
        DiagnosticSource::with_stage("docnav", "standard-parameters"),
    ))
}

fn invalid_request_record(
    field: &str,
    reason: String,
    source: DiagnosticSource,
) -> DiagnosticRecordDraft {
    DiagnosticRecordDraft::new::<typed_codes::protocol::InvalidRequest>(
        reason.clone(),
        FieldReasonDetails::new(field, reason),
        source,
    )
}
