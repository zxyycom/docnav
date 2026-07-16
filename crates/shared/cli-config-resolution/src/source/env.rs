use std::collections::BTreeMap;

use docnav_typed_fields::{FieldDefSet, JsonValue, ProcessingId, ProcessingLocator, ValueKind};

use super::{Source, SourceCandidate, SourceError, SourceId, SourceKind, SourceLocator};

pub fn extract_env<I, K, V>(
    fields: &FieldDefSet,
    processing_id: &ProcessingId,
    source_id: SourceId,
    priority: i32,
    variables: I,
) -> Result<Source, SourceError>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    let variables = variables
        .into_iter()
        .map(|(key, value)| (key.into(), value.into()))
        .collect::<BTreeMap<_, _>>();
    let candidates = fields
        .processing_metadata(processing_id)
        .into_iter()
        .filter_map(|metadata| {
            let ProcessingLocator::EnvVar(name) = metadata.locator else {
                return None;
            };
            let raw = variables.get(&name)?;
            let locator = SourceLocator::EnvVar(name);
            Some(match decode_env_value(raw, metadata.value_kind) {
                Ok(value) => SourceCandidate::value(metadata.identity, locator, value),
                Err(reason) => SourceCandidate::invalid(
                    metadata.identity,
                    locator,
                    JsonValue::String(raw.clone()),
                    reason,
                ),
            })
        })
        .collect();
    Source::new(source_id, SourceKind::Env, priority, candidates)
}

fn decode_env_value(raw: &str, kind: ValueKind) -> Result<JsonValue, String> {
    match kind {
        ValueKind::String => Ok(JsonValue::String(raw.to_owned())),
        ValueKind::Integer => raw
            .parse::<i64>()
            .map(JsonValue::from)
            .map_err(|_| "expected integer environment value".to_owned()),
        ValueKind::Number => raw
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(JsonValue::Number)
            .ok_or_else(|| "expected finite number environment value".to_owned()),
        ValueKind::Boolean => match raw {
            "true" => Ok(JsonValue::Bool(true)),
            "false" => Ok(JsonValue::Bool(false)),
            _ => Err("expected boolean environment value".to_owned()),
        },
        ValueKind::Array => decode_structured_env(raw, "array", JsonValue::is_array),
        ValueKind::Object => decode_structured_env(raw, "object", JsonValue::is_object),
        ValueKind::Json => serde_json::from_str(raw)
            .map_err(|error| format!("expected JSON environment value: {error}")),
    }
}

fn decode_structured_env(
    raw: &str,
    expected: &str,
    accepts: impl FnOnce(&JsonValue) -> bool,
) -> Result<JsonValue, String> {
    let value = serde_json::from_str(raw)
        .map_err(|error| format!("expected JSON {expected} environment value: {error}"))?;
    if accepts(&value) {
        Ok(value)
    } else {
        Err(format!("expected JSON {expected} environment value"))
    }
}
