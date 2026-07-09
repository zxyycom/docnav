use std::path::Path;

use docnav_diagnostics::{
    typed_codes, AdapterConfigSourceDetails, DiagnosticSource, FieldReasonDetails,
};
use docnav_protocol::protocol_error_record_draft_with_summary;

use crate::error::AppError;
use crate::project_context::ConfigPathOrigin;

use super::{path_string, ConfigFileSource};

pub(super) fn config_source_error(
    path: &Path,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
    reason_code: &str,
) -> AppError {
    config_error(
        path,
        source,
        origin,
        ConfigErrorSpec {
            field: "config",
            reason_code,
            accepted: None,
            summary: "Config file is invalid.",
            guidance: Some("Fix the config file so it is a readable JSON object.".to_owned()),
        },
    )
}

struct ConfigErrorSpec<'a> {
    field: &'a str,
    reason_code: &'a str,
    accepted: Option<&'a str>,
    summary: &'a str,
    guidance: Option<String>,
}

fn config_error(
    path: &Path,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
    spec: ConfigErrorSpec<'_>,
) -> AppError {
    let path = path_string(path);
    let mut details = FieldReasonDetails::new(spec.field, spec.reason_code);
    details.path = Some(path.clone());
    details.received = Some(if spec.field == "config" {
        path.clone()
    } else {
        spec.field.to_owned()
    });
    details.accepted = spec.accepted.map(|value| vec![value.to_owned()]);
    let mut issue =
        AdapterConfigSourceDetails::new(source.as_str(), origin.as_str(), &path, spec.reason_code);
    if spec.field != "config" {
        issue = issue.with_field(spec.field);
    }
    details.config_issues = Some(vec![issue]);

    let mut draft = protocol_error_record_draft_with_summary::<typed_codes::protocol::InvalidRequest>(
        spec.summary,
        details,
        DiagnosticSource::with_stage("docnav", "config"),
    );
    if let Some(guidance) = spec.guidance {
        draft = draft.with_guidance([guidance]);
    }
    AppError::new(draft)
}
