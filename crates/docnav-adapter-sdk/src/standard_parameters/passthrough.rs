use docnav_standard_parameters::{
    PassthroughValue, StandardParameterResolution, StandardParameterSourceKind,
};
use docnav_typed_fields::{JsonValue, ProcessingBuild};
use serde_json::{Map, Value};

use crate::AdapterError;

const DIRECT_PROCESSING: &str = "direct";

pub(super) fn native_options_processing(
) -> Result<ProcessingBuild<'static, JsonValue, JsonValue>, AdapterError> {
    ProcessingBuild::new(DIRECT_PROCESSING, native_options_passthrough)
        .map_err(|error| AdapterError::internal(format!("invoke-passthrough-processing:{error}")))
}

pub(crate) fn native_options_passthrough(raw: JsonValue) -> JsonValue {
    raw_options(&raw)
        .map(Value::Object)
        .unwrap_or_else(|| Value::Object(Map::new()))
}

pub(super) fn raw_options(raw: &JsonValue) -> Option<Map<String, Value>> {
    raw.get("options").and_then(Value::as_object).cloned()
}

pub(super) fn options_from_resolution(
    resolution: &StandardParameterResolution,
) -> Option<serde_json::Map<String, Value>> {
    let Value::Object(options) = passthrough_from_source(resolution)?.value.clone() else {
        return None;
    };
    (!options.is_empty()).then_some(options)
}

fn passthrough_from_source(resolution: &StandardParameterResolution) -> Option<&PassthroughValue> {
    resolution
        .passthrough()
        .iter()
        .find(|value| value.source.kind == StandardParameterSourceKind::DirectInput)
}
