use docnav_adapter_contracts::AdapterError;
use docnav_diagnostics::{
    typed_codes, AdapterConfigSourceDetails, DiagnosticRecordDraft, DiagnosticSource,
    FieldReasonDetails, InternalDetails,
};
use docnav_protocol::protocol_error_record_draft_with_summary;

use crate::{NavigationConfigSource, NavigationFailureLayer, NavigationInvocationTrace};

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationError {
    diagnostic: Box<DiagnosticRecordDraft>,
    failure_layer: Option<NavigationFailureLayer>,
    selected_adapter_id: Option<String>,
    request_id: Option<String>,
}

pub(crate) struct ConfigFieldError<'a> {
    source_level: &'static str,
    path_origin: &'static str,
    path: &'a str,
    field: String,
    reason_code: &'static str,
    accepted: Option<Vec<String>>,
    summary: &'static str,
    guidance: String,
}

impl<'a> ConfigFieldError<'a> {
    pub(crate) fn invalid(
        source: &'a NavigationConfigSource,
        field: impl Into<String>,
        reason_code: &'static str,
        guidance: impl Into<String>,
    ) -> Self {
        Self {
            source_level: source.level,
            path_origin: source.origin,
            path: &source.path,
            field: field.into(),
            reason_code,
            accepted: None,
            summary: "Config file contains an invalid field value.",
            guidance: guidance.into(),
        }
    }

    pub(crate) fn with_accepted(mut self, accepted: Vec<String>) -> Self {
        self.accepted = Some(accepted);
        self
    }
}

impl NavigationError {
    pub fn new(diagnostic: DiagnosticRecordDraft) -> Self {
        Self {
            diagnostic: Box::new(diagnostic),
            failure_layer: None,
            selected_adapter_id: None,
            request_id: None,
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

    pub fn protocol_response_invalid(reason: impl Into<String>) -> Self {
        let reason = reason.into();
        Self::new(DiagnosticRecordDraft::new::<
            typed_codes::protocol::InternalError,
        >(
            "Navigation response validation failed.",
            InternalDetails::new(format!("protocol-response-validation-failed: {reason}")),
            DiagnosticSource::with_stage("docnav-navigation", "result-validation"),
        ))
    }

    pub fn config_unknown_field(
        source_level: &'static str,
        path_origin: &'static str,
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
            path_origin,
            path,
            field,
            reason_code: "unknown_config_field",
            accepted: accepted.map(|value| vec![value.to_owned()]),
            summary: "Config file contains an unknown field.",
            guidance,
        })
    }

    pub(crate) fn config_invalid_field(spec: ConfigFieldError<'_>) -> Self {
        Self::config_field_error(spec)
    }

    pub fn config_missing_field(
        source_level: &'static str,
        path_origin: &'static str,
        path: &str,
        field: impl Into<String>,
    ) -> Self {
        let field = field.into();
        Self::config_field_error(ConfigFieldError {
            source_level,
            path_origin,
            path,
            field: field.clone(),
            reason_code: "missing_config_field",
            accepted: None,
            summary: "Config file is missing a required field.",
            guidance: format!("Add config field {field}."),
        })
    }

    pub fn config_invalid_object(
        source_level: &'static str,
        path_origin: &'static str,
        path: &str,
        field: &str,
    ) -> Self {
        Self::config_field_error(ConfigFieldError {
            source_level,
            path_origin,
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
        details.accepted = spec.accepted;
        let mut issue = AdapterConfigSourceDetails::new(
            spec.source_level,
            spec.path_origin,
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

    pub fn failure_layer(&self) -> Option<NavigationFailureLayer> {
        self.failure_layer
    }

    pub fn selected_adapter_id(&self) -> Option<&str> {
        self.selected_adapter_id.as_deref()
    }

    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    pub(crate) fn with_invocation_trace(mut self, trace: &NavigationInvocationTrace) -> Self {
        self.failure_layer = trace.failure_layer;
        self.selected_adapter_id = trace.selected_adapter_id.clone();
        self.request_id = trace.request_id.clone();
        self
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
