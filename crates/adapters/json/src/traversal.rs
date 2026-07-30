use crate::document::{JsonDocument, JsonKind, JsonNode, JsonValue};
use crate::reference::canonical_ref;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct JsonEntry {
    pub(crate) ref_id: String,
    pub(crate) label: String,
    pub(crate) kind: JsonKind,
}

impl JsonDocument {
    pub(crate) fn preorder_entries(&self) -> Vec<JsonEntry> {
        let mut entries = Vec::with_capacity(self.node_count);
        let mut path = Vec::new();

        match self.root.value {
            JsonValue::Object(_) | JsonValue::Array(_) => {
                append_descendant_entries(&self.root, &mut path, &mut entries);
            }
            JsonValue::String(_)
            | JsonValue::Number(_)
            | JsonValue::Boolean(_)
            | JsonValue::Null => entries.push(JsonEntry {
                ref_id: canonical_ref(&[]),
                label: "<root>".to_owned(),
                kind: self.root.kind(),
            }),
        }

        entries
    }
}

fn append_descendant_entries(
    node: &JsonNode,
    path: &mut Vec<String>,
    entries: &mut Vec<JsonEntry>,
) {
    match &node.value {
        JsonValue::Object(members) => {
            for member in members {
                path.push(member.name.clone());
                let label = if member.name.is_empty() {
                    "\"\"".to_owned()
                } else {
                    member.name.clone()
                };
                append_entry(&member.value, label, path, entries);
                path.pop();
            }
        }
        JsonValue::Array(elements) => {
            for (index, element) in elements.iter().enumerate() {
                path.push(index.to_string());
                append_entry(element, format!("[{index}]"), path, entries);
                path.pop();
            }
        }
        JsonValue::String(_) | JsonValue::Number(_) | JsonValue::Boolean(_) | JsonValue::Null => {}
    }
}

fn append_entry(
    node: &JsonNode,
    label: String,
    path: &mut Vec<String>,
    entries: &mut Vec<JsonEntry>,
) {
    let tokens = path.iter().map(String::as_str).collect::<Vec<_>>();
    entries.push(JsonEntry {
        ref_id: canonical_ref(&tokens),
        label,
        kind: node.kind(),
    });
    append_descendant_entries(node, path, entries);
}

#[cfg(test)]
mod tests;
