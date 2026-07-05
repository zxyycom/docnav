use std::collections::BTreeMap;

use docnav_typed_fields::{DefaultMetadata, FieldIdentity, JsonValue};

use crate::{ParameterCatalogEntry, ParameterPath, ParameterSource};

#[cfg(test)]
pub(crate) fn construct_direct_input_source(
    entries: &[ParameterCatalogEntry],
    input: Option<&JsonValue>,
) -> ParameterSource {
    construct_direct_input_source_with_passthrough(entries, input, input)
}

pub(crate) fn construct_direct_input_source_with_passthrough(
    entries: &[ParameterCatalogEntry],
    input: Option<&JsonValue>,
    processing_result: Option<&JsonValue>,
) -> ParameterSource {
    construct_source(input, processing_result, entries, |entry| {
        entry.direct_input_path.as_ref()
    })
}

#[cfg(test)]
pub(crate) fn construct_config_source(
    entries: &[ParameterCatalogEntry],
    input: Option<&JsonValue>,
) -> ParameterSource {
    construct_config_source_with_passthrough(entries, input, input)
}

pub(crate) fn construct_config_source_with_passthrough(
    entries: &[ParameterCatalogEntry],
    input: Option<&JsonValue>,
    processing_result: Option<&JsonValue>,
) -> ParameterSource {
    construct_source(input, processing_result, entries, |entry| {
        entry.config_path.as_ref()
    })
}

pub(crate) fn construct_default_source(
    entries: &[ParameterCatalogEntry],
    dynamic_defaults: &BTreeMap<FieldIdentity, JsonValue>,
) -> ParameterSource {
    let mut source = ParameterSource::default();
    for entry in entries {
        if let Some(value) = dynamic_defaults.get(entry.identity()) {
            source.insert_value(entry.identity().clone(), value.clone());
            continue;
        }
        if let DefaultMetadata::Static(value) = &entry.metadata.default {
            source.insert_value(entry.identity().clone(), value.clone());
        }
    }
    source
}

fn construct_source(
    input: Option<&JsonValue>,
    processing_result: Option<&JsonValue>,
    entries: &[ParameterCatalogEntry],
    path_for: impl Fn(&ParameterCatalogEntry) -> Option<&ParameterPath>,
) -> ParameterSource {
    let Some(input) = input else {
        return ParameterSource::default();
    };
    let mut source = ParameterSource::default();

    for entry in entries {
        let Some(path) = path_for(entry) else {
            continue;
        };
        if let Some(value) = value_at_path(input, path) {
            source.insert_value(entry.identity().clone(), value.clone());
        }
    }

    if let Some(processing_result) = processing_result {
        source.set_processing_result(processing_result.clone());
    }
    source
}

fn value_at_path<'a>(root: &'a JsonValue, path: &ParameterPath) -> Option<&'a JsonValue> {
    let mut current = root;
    for segment in path.segments() {
        current = current.as_object()?.get(segment)?;
    }
    Some(current)
}
