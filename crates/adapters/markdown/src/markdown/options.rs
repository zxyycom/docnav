use docnav_adapter_contracts::{AdapterError, AdapterResult, NativeOptionIssue};

use crate::adapter::ADAPTER_ID;

const OPTION_NAMESPACE: &str = "options";
const MAX_HEADING_LEVEL_OPTION: &str = "max_heading_level";

pub fn max_heading_level(value: Option<i64>) -> AdapterResult<u8> {
    let value =
        value.ok_or_else(|| AdapterError::internal("markdown-max-heading-level-missing"))?;
    u8::try_from(value)
        .ok()
        .filter(|value| (1..=6).contains(value))
        .ok_or_else(|| {
            AdapterError::native_option_invalid(
                "Native option value is invalid.",
                NativeOptionIssue {
                    owner: ADAPTER_ID.to_owned(),
                    namespace: OPTION_NAMESPACE.to_owned(),
                    key: MAX_HEADING_LEVEL_OPTION.to_owned(),
                    source: "standard_input".to_owned(),
                    reason_code: "range_invalid".to_owned(),
                    field: format!("arguments.options.{MAX_HEADING_LEVEL_OPTION}"),
                    received: Some(value.to_string()),
                    expected: Some("integer in range 1..6".to_owned()),
                    type_variant: Some("integer".to_owned()),
                    config_source: None,
                },
                [format!(
                    "Use integer in range 1..6 for option {MAX_HEADING_LEVEL_OPTION}."
                )],
            )
        })
}
