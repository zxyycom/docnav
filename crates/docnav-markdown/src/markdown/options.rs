use docnav_adapter_contracts::{AdapterError, AdapterResult};
use docnav_protocol::Options;

use crate::adapter::{ADAPTER_ID, MAX_HEADING_LEVEL_OPTION, NATIVE_OPTIONS_NAMESPACE};

pub fn max_heading_level_from_options(options: Option<&Options>) -> AdapterResult<u8> {
    let value = options
        .and_then(|options| options.entry_for_key(MAX_HEADING_LEVEL_OPTION))
        .ok_or_else(|| AdapterError::internal("markdown-max-heading-level-missing"))?;

    let level = value
        .value
        .as_u64()
        .and_then(|value| u8::try_from(value).ok())
        .filter(|value| (1..=6).contains(value))
        .ok_or_else(|| AdapterError::internal("markdown-max-heading-level-invalid"))?;

    if value.owner != ADAPTER_ID
        || value.namespace != NATIVE_OPTIONS_NAMESPACE
        || value.key != MAX_HEADING_LEVEL_OPTION
    {
        return Err(AdapterError::internal(
            "markdown-max-heading-level-identity-mismatch",
        ));
    }

    Ok(level)
}
