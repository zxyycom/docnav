use std::path::Path;

use docnav_diagnostics::{
    typed_codes, AdapterConfigSourceDetails, DiagnosticSource, FieldReasonDetails,
};
use docnav_protocol::protocol_error_record_draft_with_summary;

use crate::error::AppError;
use crate::project_context::ConfigPathOrigin;

use super::{path_string, ConfigFileSource};

const INVALID_CONFIG_OBJECT: &str = "invalid_config_object";

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

pub(super) fn unknown_config_field_error(
    path: &Path,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
    field: &str,
    accepted: Option<&str>,
) -> AppError {
    config_error(
        path,
        source,
        origin,
        ConfigErrorSpec {
            field,
            reason_code: "unknown_config_field",
            accepted,
            summary: "Config file contains an unknown field.",
            guidance: Some(match accepted {
                Some(accepted) => format!("Rename {field} to {accepted}."),
                None => format!("Remove unsupported config field {field}."),
            }),
        },
    )
}

pub(super) fn invalid_config_object_error(
    path: &Path,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
    field: &str,
) -> AppError {
    config_error(
        path,
        source,
        origin,
        ConfigErrorSpec {
            field,
            reason_code: INVALID_CONFIG_OBJECT,
            accepted: None,
            summary: "Config file field must be an object.",
            guidance: Some(format!("Use an object for config field {field}.")),
        },
    )
}

pub(super) fn invalid_config_array_error(
    path: &Path,
    source: ConfigFileSource,
    origin: ConfigPathOrigin,
    field: &str,
) -> AppError {
    config_error(
        path,
        source,
        origin,
        ConfigErrorSpec {
            field,
            reason_code: "invalid_config_array",
            accepted: None,
            summary: "Config file contains an invalid field value.",
            guidance: Some(format!("Use an array for config field {field}.")),
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
