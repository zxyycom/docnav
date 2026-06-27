use docnav_adapter_sdk::{AdapterError, AdapterResult};

use crate::adapter::{DEFAULT_MAX_HEADING_LEVEL, MAX_HEADING_LEVEL_OPTION};

pub fn max_heading_level_from_options(
    options: Option<&docnav_protocol::Options>,
) -> AdapterResult<u8> {
    let Some(options) = options else {
        return Ok(DEFAULT_MAX_HEADING_LEVEL);
    };
    let Some(value) = options.get(MAX_HEADING_LEVEL_OPTION) else {
        return Ok(DEFAULT_MAX_HEADING_LEVEL);
    };
    let Some(level) = value.as_u64() else {
        return Err(invalid_max_heading_level());
    };
    if !(1..=6).contains(&level) {
        return Err(invalid_max_heading_level());
    }
    Ok(level as u8)
}

fn invalid_max_heading_level() -> AdapterError {
    AdapterError::invalid_request(
        format!("arguments.options.{MAX_HEADING_LEVEL_OPTION}"),
        "must be an integer from 1 to 6",
    )
}
