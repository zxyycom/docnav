use std::collections::BTreeMap;

use docnav_typed_fields::{DefaultMetadata, FieldIdentity, JsonValue};

use crate::{StandardParameterCatalogEntry, StandardParameterPath, StandardParameterSource};

#[cfg(test)]
pub(crate) fn construct_direct_input_source(
    entries: &[StandardParameterCatalogEntry],
    input: Option<&JsonValue>,
) -> StandardParameterSource {
    construct_direct_input_source_with_passthrough(entries, input, input)
}

pub(crate) fn construct_direct_input_source_with_passthrough(
    entries: &[StandardParameterCatalogEntry],
    input: Option<&JsonValue>,
    passthrough_input: Option<&JsonValue>,
) -> StandardParameterSource {
    construct_source(input, passthrough_input, entries, |entry| {
        entry.direct_input_path.as_ref()
    })
}

#[cfg(test)]
pub(crate) fn construct_config_source(
    entries: &[StandardParameterCatalogEntry],
    input: Option<&JsonValue>,
) -> StandardParameterSource {
    construct_config_source_with_passthrough(entries, input, input)
}

pub(crate) fn construct_config_source_with_passthrough(
    entries: &[StandardParameterCatalogEntry],
    input: Option<&JsonValue>,
    passthrough_input: Option<&JsonValue>,
) -> StandardParameterSource {
    construct_source(input, passthrough_input, entries, |entry| {
        entry.config_path.as_ref()
    })
}

pub(crate) fn construct_default_source(
    entries: &[StandardParameterCatalogEntry],
    dynamic_defaults: &BTreeMap<FieldIdentity, JsonValue>,
) -> StandardParameterSource {
    let mut source = StandardParameterSource::default();
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
    passthrough_input: Option<&JsonValue>,
    entries: &[StandardParameterCatalogEntry],
    path_for: impl Fn(&StandardParameterCatalogEntry) -> Option<&StandardParameterPath>,
) -> StandardParameterSource {
    let Some(input) = input else {
        return StandardParameterSource::default();
    };
    let mut source = StandardParameterSource::default();

    for entry in entries {
        let Some(path) = path_for(entry) else {
            continue;
        };
        if let Some(value) = value_at_path(input, path) {
            source.insert_value(entry.identity().clone(), value.clone());
        }
    }

    if let Some(passthrough_input) = passthrough_input {
        collect_passthrough(passthrough_input, &mut source);
    }
    source
}

fn value_at_path<'a>(root: &'a JsonValue, path: &StandardParameterPath) -> Option<&'a JsonValue> {
    let mut current = root;
    for segment in path.segments() {
        current = current.as_object()?.get(segment)?;
    }
    Some(current)
}

fn collect_passthrough(value: &JsonValue, source: &mut StandardParameterSource) {
    let JsonValue::Object(object) = value else {
        return;
    };
    for (key, value) in object.clone() {
        let path = StandardParameterPath::new([key])
            .expect("passthrough path is built from non-empty JSON object keys");
        source.push_passthrough(path, value);
    }
}
