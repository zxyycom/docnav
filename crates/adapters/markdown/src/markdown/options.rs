use docnav_adapter_contracts::{
    AdapterError, AdapterResult, NativeOptionHandoff, NativeOptionValue,
};

use crate::adapter::{
    ADAPTER_ID, MAX_HEADING_LEVEL_IDENTITY, MAX_HEADING_LEVEL_OPTION, NATIVE_OPTIONS_NAMESPACE,
};

pub fn max_heading_level_from_handoff(native_options: &NativeOptionHandoff) -> AdapterResult<u8> {
    let value = native_options
        .get(
            ADAPTER_ID,
            NATIVE_OPTIONS_NAMESPACE,
            MAX_HEADING_LEVEL_OPTION,
        )
        .ok_or_else(|| AdapterError::internal("markdown-max-heading-level-missing"))?;
    validate_max_heading_level_identity(value)?;

    let level = value
        .value
        .as_u64()
        .and_then(|value| u8::try_from(value).ok())
        .filter(|value| (1..=6).contains(value))
        .ok_or_else(|| AdapterError::internal("markdown-max-heading-level-invalid"))?;

    Ok(level)
}

fn validate_max_heading_level_identity(value: &NativeOptionValue) -> AdapterResult<()> {
    if value.identity != MAX_HEADING_LEVEL_IDENTITY
        || value.type_variant != "integer"
        || value.source.is_empty()
    {
        return Err(AdapterError::internal(
            "markdown-max-heading-level-identity-mismatch",
        ));
    }
    Ok(())
}
