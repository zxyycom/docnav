use std::collections::BTreeMap;

use docnav_typed_fields::{DefaultMetadata, FieldIdentity, JsonValue};

use crate::{StandardParameterCatalogEntry, StandardParameterPath, StandardParameterSource};

pub(crate) fn construct_direct_input_source(
    entries: &[StandardParameterCatalogEntry],
    input: Option<&JsonValue>,
) -> StandardParameterSource {
    construct_source(input, entries, |entry| entry.direct_input_path.as_ref())
}

pub(crate) fn construct_config_source(
    entries: &[StandardParameterCatalogEntry],
    input: Option<&JsonValue>,
) -> StandardParameterSource {
    construct_source(input, entries, |entry| entry.config_path.as_ref())
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
    entries: &[StandardParameterCatalogEntry],
    path_for: impl Fn(&StandardParameterCatalogEntry) -> Option<&StandardParameterPath>,
) -> StandardParameterSource {
    let Some(input) = input else {
        return StandardParameterSource::default();
    };
    let mut source = StandardParameterSource::default();
    let mut mapped_paths = Vec::new();

    for entry in entries {
        let Some(path) = path_for(entry) else {
            continue;
        };
        mapped_paths.push(path.key());
        if let Some(value) = value_at_path(input, path) {
            source.insert_value(entry.identity().clone(), value.clone());
        }
    }

    collect_passthrough(input, &mut Vec::new(), &mapped_paths, &mut source);
    source
}

fn value_at_path<'a>(root: &'a JsonValue, path: &StandardParameterPath) -> Option<&'a JsonValue> {
    let mut current = root;
    for segment in path.segments() {
        current = current.as_object()?.get(segment)?;
    }
    Some(current)
}

fn collect_passthrough(
    value: &JsonValue,
    prefix: &mut Vec<String>,
    mapped_paths: &[Vec<String>],
    source: &mut StandardParameterSource,
) {
    if mapped_paths.iter().any(|path| path == prefix) {
        return;
    }

    if let JsonValue::Object(object) = value {
        for (key, value) in object {
            prefix.push(key.clone());
            collect_passthrough(value, prefix, mapped_paths, source);
            prefix.pop();
        }
        return;
    }

    if prefix.is_empty() {
        return;
    }
    let path = StandardParameterPath::new(prefix.clone())
        .expect("passthrough path is built from non-empty JSON object keys");
    source.push_passthrough(path, value.clone());
}
