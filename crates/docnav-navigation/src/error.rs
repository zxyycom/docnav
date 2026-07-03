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
        let mut details = FieldReasonDetails::new(field.clone(), "unknown_config_field");
        details.path = Some(path.to_owned());
        details.received = Some(field.clone());
        details.accepted = accepted.map(|value| vec![value.to_owned()]);
        let mut issue =
            AdapterConfigSourceDetails::new(source_level, "default", path, "unknown_config_field");
        issue = issue.with_field(field.clone());
        details.config_issues = Some(vec![issue]);

        let mut draft =
            protocol_error_record_draft_with_summary::<typed_codes::protocol::InvalidRequest>(
                "Config file contains an unknown field.",
                details,
                DiagnosticSource::with_stage("docnav", "config"),
            );
        draft = draft.with_guidance([match accepted {
            Some(accepted) => format!("Rename {field} to {accepted}."),
            None => format!("Remove unsupported config field {field}."),
        }]);
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
