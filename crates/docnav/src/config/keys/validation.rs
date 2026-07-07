use std::path::Path;

use docnav_diagnostics::{typed_codes, DiagnosticSource, FieldReasonDetails};
use docnav_protocol::protocol_error_record_draft_with_summary;

use crate::cli::OutputMode;
use crate::error::{AppError, AppResult};
use crate::project_paths::path_to_slash;
use crate::registry::AdapterRegistry;

pub(in crate::config) fn validate_output_key(
    field: &str,
    value: &Option<String>,
    path: &Path,
) -> AppResult<()> {
    if let Some(value) = value {
        parse_output(field, value, path)?;
    }
    Ok(())
}

pub(in crate::config) fn validate_native_option_key_for_registry(
    registry: &AdapterRegistry,
    key: &str,
) -> AppResult<()> {
    if registry.has_native_option_config_key(key) {
        Ok(())
    } else {
        Err(unknown_key(key))
    }
}

pub(super) fn parse_output(field: &str, value: &str, path: &Path) -> AppResult<OutputMode> {
    value.parse::<OutputMode>().map_err(|reason: String| {
        let path = path_to_slash(path);
        let accepted = OutputMode::ACCEPTED_VALUES.join(", ");
        let details = FieldReasonDetails {
            field: field.to_owned(),
            reason: format!(
                "{path} contains invalid {field}: received {value:?}; accepted values: {accepted}; {reason}"
            ),
            path: Some(path),
            received: Some(value.to_owned()),
            accepted: Some(
                OutputMode::ACCEPTED_VALUES
                    .iter()
                    .map(|value| (*value).to_owned())
                    .collect(),
            ),
            field_issues: None,
            config_issues: None,
            typed_validation_failures: None,
            option_issues: None,
        };
        AppError::new(protocol_error_record_draft_with_summary::<
            typed_codes::protocol::InvalidRequest,
        >(
            "Invalid protocol request.",
            details,
            DiagnosticSource::with_stage("docnav", "config"),
        ))
    })
}

pub(super) fn unknown_key(key: &str) -> AppError {
    AppError::invalid_request("key", format!("unsupported docnav config key {key:?}"))
}

pub(super) fn parse_pagination_enabled(key: &str, value: &str) -> AppResult<bool> {
    match value {
        "true" | "enabled" => Ok(true),
        "false" | "disabled" => Ok(false),
        _ => Err(AppError::invalid_request(
            key,
            format!("{key} must be true, false, enabled, or disabled"),
        )),
    }
}

pub(super) fn parse_native_option_value(
    registry: &AdapterRegistry,
    key: &str,
    value: &str,
) -> AppResult<serde_json::Value> {
    validate_native_option_key_for_registry(registry, key)?;
    Ok(serde_json::from_str(value).unwrap_or_else(|_| serde_json::Value::String(value.to_owned())))
}

pub(in crate::config) fn validate_positive_key(key: &str, value: u32) -> AppResult<()> {
    if value == 0 {
        Err(AppError::invalid_request(
            key,
            format!("{key} must be a positive integer"),
        ))
    } else {
        Ok(())
    }
}

pub(in crate::config) fn validate_invocation_log_path_key(key: &str, value: &str) -> AppResult<()> {
    if value.is_empty() {
        return Err(AppError::invalid_request(
            key,
            "invocation log path must not be empty",
        ));
    }
    Ok(())
}

pub(in crate::config) fn validate_invocation_log_content_capture_root_key(
    key: &str,
    value: &str,
) -> AppResult<()> {
    if value.is_empty() {
        return Err(AppError::invalid_request(
            key,
            "invocation log content capture root must not be empty",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::Value;

    use super::*;
    use crate::project_context::{ProjectContext, SelectedConfigPath, SelectedConfigPaths};

    #[test]
    fn max_heading_level_config_accepts_registered_raw_values() {
        let registry = AdapterRegistry::load(&project_context()).unwrap();

        assert_eq!(
            parse_native_option_value(&registry, MAX_HEADING_LEVEL_KEY, "7").unwrap(),
            serde_json::json!(7)
        );
        assert_eq!(
            parse_native_option_value(&registry, MAX_HEADING_LEVEL_KEY, "\"wide\"").unwrap(),
            serde_json::json!("wide")
        );
    }

    #[test]
    fn unregistered_native_option_config_key_fails() {
        let registry = AdapterRegistry::load(&project_context()).unwrap();
        let error = parse_native_option_value(&registry, "options.missing", "1").unwrap_err();
        let Value::Object(details) = error.diagnostic().details().to_value() else {
            panic!("invalid request details should be an object");
        };
        assert_eq!(
            details.get("reason").and_then(Value::as_str),
            Some("unsupported docnav config key \"options.missing\"")
        );
    }

    fn project_context() -> ProjectContext {
        let root = PathBuf::from("D:/docnav-config-validation-test");
        ProjectContext {
            cwd: root.clone(),
            project_root: root.clone(),
            config_paths: SelectedConfigPaths {
                project: SelectedConfigPath::default(root.join(".docnav").join("docnav.json")),
                user: SelectedConfigPath::default(root.join("user").join("docnav.json")),
            },
        }
    }

    const MAX_HEADING_LEVEL_KEY: &str = "options.max_heading_level";
}
