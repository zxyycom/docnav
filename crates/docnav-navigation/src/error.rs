use docnav_adapter_contracts::AdapterError;
use docnav_diagnostics::{
    typed_codes, AdapterConfigSourceDetails, DiagnosticRecordDraft, DiagnosticSource,
    FieldReasonDetails, InternalDetails,
};
use docnav_protocol::protocol_error_record_draft_with_summary;

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationError {
    diagnostic: Box<DiagnosticRecordDraft>,
}

struct ConfigFieldError<'a> {
    source_level: &'static str,
    path: &'a str,
    field: String,
    reason_code: &'static str,
    accepted: Option<&'a str>,
    summary: &'static str,
    guidance: String,
}

impl NavigationError {
    pub fn new(diagnostic: DiagnosticRecordDraft) -> Self {
        Self {
            diagnostic: Box::new(diagnostic),
        }
    }

    pub fn invalid_request(field: impl Into<String>, reason: impl Into<String>) -> Self {
        let reason = reason.into();
        Self::new(DiagnosticRecordDraft::new::<
            typed_codes::protocol::InvalidRequest,
        >(
            reason.clone(),
            FieldReasonDetails::new(field, reason),
            DiagnosticSource::with_stage("docnav-navigation", "input"),
        ))
    }

    pub fn internal(error_id: impl Into<String>) -> Self {
        Self::new(DiagnosticRecordDraft::new::<
            typed_codes::protocol::InternalError,
        >(
            "Navigation input resolution failed.",
            InternalDetails::new(error_id),
            DiagnosticSource::with_stage("docnav-navigation", "internal"),
        ))
    }

    pub fn config_unknown_field(
        source_level: &'static str,
        path: &str,
        field: impl Into<String>,
        accepted: Option<&str>,
    ) -> Self {
        let field = field.into();
        let guidance = match accepted {
            Some(accepted) => format!("Rename {field} to {accepted}."),
            None => format!("Remove unsupported config field {field}."),
        };
        Self::config_field_error(ConfigFieldError {
            source_level,
            path,
            field,
            reason_code: "unknown_config_field",
            accepted,
            summary: "Config file contains an unknown field.",
            guidance,
        })
    }

    pub fn config_invalid_object(source_level: &'static str, path: &str, field: &str) -> Self {
        Self::config_field_error(ConfigFieldError {
            source_level,
            path,
            field: field.to_owned(),
            reason_code: "invalid_config_object",
            accepted: None,
            summary: "Config file field must be an object.",
            guidance: format!("Use an object for config field {field}."),
        })
    }

    fn config_field_error(spec: ConfigFieldError<'_>) -> Self {
        let mut details = FieldReasonDetails::new(spec.field.clone(), spec.reason_code);
        details.path = Some(spec.path.to_owned());
        details.received = Some(spec.field.clone());
        details.accepted = spec.accepted.map(|value| vec![value.to_owned()]);
        let mut issue = AdapterConfigSourceDetails::new(
            spec.source_level,
            "default",
            spec.path,
            spec.reason_code,
        );
        issue = issue.with_field(spec.field);
        details.config_issues = Some(vec![issue]);

        let draft =
            protocol_error_record_draft_with_summary::<typed_codes::protocol::InvalidRequest>(
                spec.summary,
                details,
                DiagnosticSource::with_stage("docnav", "config"),
            )
            .with_guidance([spec.guidance]);
        Self::new(draft)
    }

    pub fn diagnostic(&self) -> &DiagnosticRecordDraft {
        self.diagnostic.as_ref()
    }

    pub fn into_diagnostic(self) -> DiagnosticRecordDraft {
        *self.diagnostic
    }
}

impl From<AdapterError> for NavigationError {
    fn from(value: AdapterError) -> Self {
        Self::new(value.into_diagnostic())
    }
}
