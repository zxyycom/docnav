use docnav_adapter_contracts::{AdapterError, AdapterResult, NativeOptionIssue};
use docnav_protocol::{OptionEntry, Options};
use serde_json::Value;

use crate::adapter::{
    ADAPTER_ID, DEFAULT_MAX_HEADING_LEVEL, MAX_HEADING_LEVEL_OPTION, NATIVE_OPTIONS_NAMESPACE,
};

const EXPECTED_MAX_HEADING_LEVEL: &str = "integer in range 1..6";

pub fn max_heading_level_from_options(options: Option<&Options>) -> AdapterResult<u8> {
    let Some(options) = options else {
        return Ok(DEFAULT_MAX_HEADING_LEVEL);
    };
    if let Some((key, value)) = options
        .iter()
        .find(|(key, _)| key.as_str() != MAX_HEADING_LEVEL_OPTION)
    {
        return Err(unsupported_option(options, key, value));
    }
    let Some(value) = options.get(MAX_HEADING_LEVEL_OPTION) else {
        return Ok(DEFAULT_MAX_HEADING_LEVEL);
    };
    let Some(level) = value.as_u64() else {
        return Err(type_mismatch_max_heading_level(options, value));
    };
    if !(1..=6).contains(&level) {
        return Err(range_invalid_max_heading_level(options, value));
    }
    Ok(level as u8)
}

fn unsupported_option(options: &Options, key: &str, value: &Value) -> AdapterError {
    let issue = option_issue(OptionIssueDraft {
        options,
        key,
        reason_code: "unsupported",
        received: received_value(value),
        expected: Some(MAX_HEADING_LEVEL_OPTION.to_owned()),
        type_variant: None,
    });
    option_error(
        issue,
        "Remove the unsupported Markdown option or use a supported option key.",
    )
}

fn type_mismatch_max_heading_level(options: &Options, value: &Value) -> AdapterError {
    let issue = option_issue(OptionIssueDraft {
        options,
        key: MAX_HEADING_LEVEL_OPTION,
        reason_code: "type_mismatch",
        received: received_value(value),
        expected: Some(EXPECTED_MAX_HEADING_LEVEL.to_owned()),
        type_variant: Some("integer".to_owned()),
    });
    option_error(
        issue,
        "Use an integer Markdown max_heading_level value from 1 through 6.",
    )
}

fn range_invalid_max_heading_level(options: &Options, value: &Value) -> AdapterError {
    let issue = option_issue(OptionIssueDraft {
        options,
        key: MAX_HEADING_LEVEL_OPTION,
        reason_code: "range_invalid",
        received: received_value(value),
        expected: Some(EXPECTED_MAX_HEADING_LEVEL.to_owned()),
        type_variant: Some("integer".to_owned()),
    });
    option_error(
        issue,
        "Use a Markdown max_heading_level value from 1 through 6.",
    )
}

struct OptionIssueDraft<'a> {
    options: &'a Options,
    key: &'a str,
    reason_code: &'a str,
    received: String,
    expected: Option<String>,
    type_variant: Option<String>,
}

fn option_issue(draft: OptionIssueDraft<'_>) -> NativeOptionIssue {
    let entry = option_entry(draft.options, draft.key);
    NativeOptionIssue {
        owner: ADAPTER_ID.to_owned(),
        namespace: NATIVE_OPTIONS_NAMESPACE.to_owned(),
        key: draft.key.to_owned(),
        source: entry
            .map(|entry| entry.source.clone())
            .unwrap_or_else(|| "direct".to_owned()),
        reason_code: draft.reason_code.to_owned(),
        field: format!("arguments.options.{}", draft.key),
        received: Some(draft.received),
        expected: draft.expected,
        type_variant: draft
            .type_variant
            .or_else(|| entry.map(|entry| entry.type_variant.clone())),
    }
}

fn option_entry<'a>(options: &'a Options, key: &str) -> Option<&'a OptionEntry> {
    options
        .entry_for(ADAPTER_ID, NATIVE_OPTIONS_NAMESPACE, key)
        .or_else(|| options.entry_for_key(key))
}

fn option_error(issue: NativeOptionIssue, guidance: &str) -> AdapterError {
    AdapterError::native_option_invalid(
        "Markdown option value is invalid.",
        issue,
        [guidance.to_owned()],
    )
}

fn received_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Null => "null".to_owned(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}
