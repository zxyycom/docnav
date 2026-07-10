#![forbid(unsafe_code)]
//! `serde_json::Value` companion helpers for `cli-config-resolution`.

use cli_config_resolution::{
    ConfigDocumentSource, FieldSet, SourceCandidate, SourceExtractor, SourceSpec, Value, ValueMap,
};

#[derive(Clone, Debug)]
pub struct JsonConfigSource {
    root: serde_json::Value,
}

impl JsonConfigSource {
    pub fn new(root: serde_json::Value) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &serde_json::Value {
        &self.root
    }
}

impl SourceExtractor for JsonConfigSource {
    fn extract(&self, source: &SourceSpec, fields: &FieldSet) -> Vec<SourceCandidate> {
        ConfigDocumentSource::new(value_from_json(self.root())).extract(source, fields)
    }
}

pub fn candidates_from_json_value(
    root: &serde_json::Value,
    source: &SourceSpec,
    fields: &FieldSet,
) -> Vec<SourceCandidate> {
    JsonConfigSource::new(root.clone()).extract(source, fields)
}

pub fn value_from_json(value: &serde_json::Value) -> Value {
    match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(value) => Value::Boolean(*value),
        serde_json::Value::Number(value) => number_from_json(value),
        serde_json::Value::String(value) => Value::String(value.clone()),
        serde_json::Value::Array(values) => {
            Value::List(values.iter().map(value_from_json).collect())
        }
        serde_json::Value::Object(values) => Value::Map(
            values
                .iter()
                .map(|(key, value)| (key.clone(), value_from_json(value)))
                .collect::<ValueMap>(),
        ),
    }
}

fn number_from_json(value: &serde_json::Number) -> Value {
    value
        .as_i64()
        .map(Value::Integer)
        .or_else(|| value.as_f64().map(Value::Number))
        .unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use cli_config_resolution::{
        CandidateState, ConfigPath, FieldContract, FieldProjectionDeclaration, FieldSet, SourceId,
        SourceKind, SourceLocator, SourceSpec, Value, ValueKind,
    };
    use serde_json::json;

    use super::{candidates_from_json_value, value_from_json, JsonConfigSource};

    // @case WB-PARAM-SERDE-001
    fn source_id(value: &str) -> SourceId {
        SourceId::new(value).expect("source id")
    }

    fn source(value: &str, kind: SourceKind, priority: i32) -> SourceSpec {
        SourceSpec::new(source_id(value), kind, priority)
    }

    fn field<I, S>(identity: &str, kind: ValueKind, path: I) -> FieldContract
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        FieldContract::builder(identity, kind)
            .projection(FieldProjectionDeclaration::config_path(path).expect("config path"))
            .build()
            .expect("field")
    }

    fn fields(values: Vec<FieldContract>) -> FieldSet {
        values
            .into_iter()
            .fold(FieldSet::builder(), |builder, field| {
                builder.add_field(field)
            })
            .build()
            .expect("fields")
    }

    #[test]
    fn json_value_maps_config_paths_to_source_candidates() {
        let field_set = fields(vec![
            field("limit", ValueKind::Integer, ["read", "limit"]),
            field("include", ValueKind::List, ["read", "include"]),
            field("labels", ValueKind::Map, ["read", "labels"]),
            field("missing", ValueKind::String, ["read", "missing"]),
        ]);
        let config = source("config", SourceKind::Config, 20);
        let root = json!({
            "read": {
                "limit": 10,
                "include": ["a", "b"],
                "labels": {
                    "team": "docs"
                }
            }
        });

        let candidates = candidates_from_json_value(&root, &config, &field_set);

        assert_candidate_value(&candidates, "limit", Value::Integer(10));
        assert_candidate_value(
            &candidates,
            "include",
            Value::List(vec![Value::from("a"), Value::from("b")]),
        );
        assert_candidate_value(
            &candidates,
            "labels",
            Value::Map(cli_config_resolution::ValueMap::from([(
                "team".to_owned(),
                Value::from("docs"),
            )])),
        );
        assert!(matches!(
            candidate_state(&candidates, "missing"),
            CandidateState::Missing
        ));
    }

    #[test]
    fn json_config_source_uses_core_source_extractor_contract() {
        let field_set = fields(vec![field("enabled", ValueKind::Boolean, ["enabled"])]);
        let config = source("config", SourceKind::Config, 20);
        let source = JsonConfigSource::new(json!({ "enabled": true }));

        let candidates =
            cli_config_resolution::SourceExtractor::extract(&source, &config, &field_set);

        assert_candidate_value(&candidates, "enabled", Value::Boolean(true));
    }

    #[test]
    fn json_value_conversion_preserves_shape() {
        assert_eq!(
            value_from_json(&json!({
                "name": "docnav",
                "count": 2,
                "ratio": 1.5,
                "enabled": true,
                "items": ["a", null]
            })),
            Value::Map(cli_config_resolution::ValueMap::from([
                ("count".to_owned(), Value::Integer(2)),
                ("enabled".to_owned(), Value::Boolean(true)),
                (
                    "items".to_owned(),
                    Value::List(vec![Value::from("a"), Value::Null])
                ),
                ("name".to_owned(), Value::from("docnav")),
                ("ratio".to_owned(), Value::Number(1.5)),
            ]))
        );
    }

    fn assert_candidate_value(
        candidates: &[cli_config_resolution::SourceCandidate],
        field: &str,
        expected: Value,
    ) {
        assert_eq!(
            candidate_state(candidates, field),
            &CandidateState::Present(expected)
        );
    }

    fn candidate_state<'a>(
        candidates: &'a [cli_config_resolution::SourceCandidate],
        field: &str,
    ) -> &'a CandidateState {
        candidates
            .iter()
            .find(|candidate| {
                candidate.field().as_str() == field
                    && matches!(candidate.locator(), SourceLocator::ConfigPath(_))
            })
            .expect("candidate")
            .state()
    }

    #[test]
    fn config_path_locator_is_preserved() {
        let field_set = fields(vec![field("limit", ValueKind::Integer, ["read", "limit"])]);
        let config = source("config", SourceKind::Config, 20);
        let candidates =
            candidates_from_json_value(&json!({ "read": { "limit": 10 } }), &config, &field_set);

        assert_eq!(
            candidates[0].locator(),
            &SourceLocator::ConfigPath(ConfigPath::new(["read", "limit"]).expect("path"))
        );
    }
}
