use crate::error::AppResult;
use crate::registry::AdapterRegistry;

mod listing;
mod resolve;
mod update;
mod validation;

pub(super) use self::listing::{
    config_value_to_json, effective_values, ensure_supported_key, scoped_key_value,
    supported_values_for_scope,
};
pub(super) use self::resolve::effective_key_value;
pub(super) use self::update::{set_key, unset_key};
pub(super) use self::validation::{
    validate_invocation_log_content_capture_root_key, validate_invocation_log_path_key,
    validate_output_key, validate_positive_key,
};

pub(super) fn registered_native_option_key<'a>(
    config_key: &'a str,
    registry: &AdapterRegistry,
) -> AppResult<&'a str> {
    self::validation::validate_native_option_key_for_registry(registry, config_key)?;
    config_key
        .strip_prefix("options.")
        .ok_or_else(|| self::validation::unknown_key(config_key))
}
