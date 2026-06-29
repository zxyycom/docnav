use docnav_standard_parameters::{
    load_standard_parameter_config_source, LoadedStandardParameterConfigSource,
    StandardParameterConfigSourceDescriptor,
};
use docnav_typed_fields::JsonValue;

use crate::AdapterError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InvokeStandardParameterConfig {
    pub(crate) default_limit: u32,
    pub(crate) project_config: Option<StandardParameterConfigSourceDescriptor>,
    pub(crate) user_config: Option<StandardParameterConfigSourceDescriptor>,
}

impl InvokeStandardParameterConfig {
    pub(crate) const fn new(default_limit: u32) -> Self {
        Self {
            default_limit,
            project_config: None,
            user_config: None,
        }
    }
}

pub(super) fn loaded_config_source(
    descriptor: &StandardParameterConfigSourceDescriptor,
) -> Result<LoadedStandardParameterConfigSource, AdapterError> {
    let loaded = load_standard_parameter_config_source(descriptor);
    reject_legacy_limit_config(descriptor, loaded.value())?;
    Ok(loaded)
}

fn reject_legacy_limit_config(
    descriptor: &StandardParameterConfigSourceDescriptor,
    value: Option<&JsonValue>,
) -> Result<(), AdapterError> {
    let has_legacy_limit = value
        .and_then(|value| value.get("defaults"))
        .and_then(JsonValue::as_object)
        .is_some_and(|defaults| defaults.contains_key("limit"));
    if has_legacy_limit {
        return Err(AdapterError::invalid_request(
            "defaults.limit",
            format!(
                "{} config {} uses unsupported defaults.limit; use defaults.pagination.limit",
                descriptor.level.as_str(),
                descriptor.path.display()
            ),
        ));
    }
    Ok(())
}
