use docnav_protocol::{Cost, Measurement};
use docnav_text_cost::{byte_cost, line_cost, token_cost};
use serde::ser::{Error as _, SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
use serde_json::value::RawValue;

use crate::document::{CommentBundle, JsonDocument, JsonNode, JsonValue};
use crate::reference::{RefView, ResolvedSelection};

#[derive(Debug, PartialEq)]
pub(crate) struct ContentFacts {
    pub(crate) content: String,
    pub(crate) cost: Cost,
}

impl ContentFacts {
    fn new(content: String) -> Self {
        let cost = selection_cost(&content);
        Self { content, cost }
    }
}

pub(crate) fn structured_value_facts(node: &JsonNode) -> Result<ContentFacts, serde_json::Error> {
    serde_json::to_string_pretty(&StructuredNode(node)).map(ContentFacts::new)
}

pub(crate) fn selection_facts(
    document: &JsonDocument,
    selection: &ResolvedSelection<'_>,
) -> Result<ContentFacts, serde_json::Error> {
    let selected = selection
        .frames
        .first()
        .expect("every resolved JSON selection includes a selected frame");

    match selection.view {
        RefView::Base => structured_value_facts(selected.value),
        RefView::DirectComments => comment_projection_facts(
            document,
            selected.value,
            selected
                .direct_comments
                .expect("a resolved direct-comment selection includes selected comments"),
        ),
        RefView::TailComments => comment_projection_facts(
            document,
            selected.value,
            selected
                .tail_comments
                .expect("a resolved tail-comment selection includes selected comments"),
        ),
    }
}

pub(crate) fn full_read_facts(document: &JsonDocument) -> ContentFacts {
    ContentFacts::new(document.source.clone())
}

fn comment_projection_facts(
    document: &JsonDocument,
    node: &JsonNode,
    comments: &CommentBundle,
) -> Result<ContentFacts, serde_json::Error> {
    let value = serde_json::to_string_pretty(&StructuredNode(node))?;
    let comments_len = comments
        .indices()
        .iter()
        .map(|&index| {
            #[cfg(test)]
            document.record_comment_bundle_step();
            let span = document.comments[index].span;
            span.end - span.start + 1
        })
        .sum::<usize>();
    let mut content = String::with_capacity(comments_len + value.len());
    for &index in comments.indices() {
        #[cfg(test)]
        document.record_comment_bundle_step();
        let span = document.comments[index].span;
        content.push_str(&document.source[span.start..span.end]);
        content.push('\n');
    }
    content.push_str(&value);
    Ok(ContentFacts::new(content))
}

struct StructuredNode<'node>(&'node JsonNode);

impl Serialize for StructuredNode<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.0.value {
            JsonValue::Object(members) => {
                let mut map = serializer.serialize_map(Some(members.len()))?;
                for member in members {
                    map.serialize_entry(&member.name, &Self(&member.value))?;
                }
                map.end()
            }
            JsonValue::Array(elements) => {
                let mut sequence = serializer.serialize_seq(Some(elements.len()))?;
                for element in elements {
                    sequence.serialize_element(&Self(element))?;
                }
                sequence.end()
            }
            JsonValue::String(value) => value.serialize(serializer),
            JsonValue::Number(value) => RawValue::from_string(value.clone())
                .map_err(S::Error::custom)?
                .serialize(serializer),
            JsonValue::Boolean(value) => value.serialize(serializer),
            JsonValue::Null => serializer.serialize_unit(),
        }
    }
}

fn selection_cost(content: &str) -> Cost {
    Cost {
        measurements: vec![
            with_selection_scope(line_cost(content)),
            with_selection_scope(byte_cost(content)),
            with_selection_scope(token_cost(content)),
        ],
    }
}

fn with_selection_scope(mut measurement: Measurement) -> Measurement {
    measurement.scope = Some("selection".to_owned());
    measurement
}

#[cfg(test)]
mod tests;
