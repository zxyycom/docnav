use cli_config_resolution::{FieldDefSet, Source};
use serde_json::{json, Map, Value};

use crate::NavigationCommand;

pub(super) struct DirectInput {
    common: Value,
}

impl DirectInput {
    pub(super) fn common(&self) -> &Value {
        &self.common
    }
}

pub(super) fn direct_input(command: &NavigationCommand) -> DirectInput {
    let mut input = Map::new();
    input.insert("path".to_owned(), json!(command.document_path));
    insert_optional_string(&mut input, "ref", command.ref_id.as_deref());
    insert_optional_string(&mut input, "query", command.query.as_deref());
    DirectInput {
        common: Value::Object(input),
    }
}

fn insert_optional_string(input: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        input.insert(key.to_owned(), json!(value));
    }
}

pub(super) fn cli_source_for_fields(
    source: &Source,
    fields: &FieldDefSet,
) -> Result<Source, cli_config_resolution::SourceError> {
    Source::new(
        source.id().clone(),
        source.kind().clone(),
        source.priority(),
        source
            .candidates()
            .iter()
            .filter(|candidate| fields.field(candidate.field()).is_some())
            .cloned()
            .collect(),
    )
}
