use crate::document::{CommentBundle, JsonDocument, JsonKind, JsonNode, JsonValue};
use crate::jsonc::CommentKind;
use crate::reference::{canonical_ref_for_view, RefView};

const ROOT_LABEL: &str = "<root>";
const TAIL_COMMENTS_LABEL: &str = "<tail comments>";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum JsonEntryKind {
    Value(JsonKind),
    TailComments,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct JsonEntry {
    pub(crate) ref_id: String,
    pub(crate) label: String,
    pub(crate) kind: JsonEntryKind,
    pub(crate) summary: Option<String>,
}

impl JsonDocument {
    pub(crate) fn preorder_entries(&self) -> Vec<JsonEntry> {
        let mut entries = Vec::with_capacity(self.node_count.saturating_add(self.comments.len()));
        let mut path = Vec::new();

        match &self.root.value {
            JsonValue::Object(_) | JsonValue::Array(_) => {
                if self.root.direct_comments.is_some() {
                    entries.push(logical_entry(self, &self.root, ROOT_LABEL.to_owned(), &[]));
                }
                append_descendant_entries(self, &self.root, &mut path, &mut entries);
            }
            JsonValue::String(_)
            | JsonValue::Number(_)
            | JsonValue::Boolean(_)
            | JsonValue::Null => {
                entries.push(logical_entry(self, &self.root, ROOT_LABEL.to_owned(), &[]));
            }
        }
        if let Some(comments) = self.root.tail_comments.as_ref() {
            entries.push(tail_entry(self, comments, &[]));
        }

        entries
    }
}

fn append_descendant_entries(
    document: &JsonDocument,
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
                append_entry(document, &member.value, label, path, entries);
                path.pop();
            }
        }
        JsonValue::Array(elements) => {
            for (index, element) in elements.iter().enumerate() {
                path.push(index.to_string());
                append_entry(document, element, format!("[{index}]"), path, entries);
                path.pop();
            }
        }
        JsonValue::String(_) | JsonValue::Number(_) | JsonValue::Boolean(_) | JsonValue::Null => {}
    }
}

fn append_entry(
    document: &JsonDocument,
    node: &JsonNode,
    label: String,
    path: &mut Vec<String>,
    entries: &mut Vec<JsonEntry>,
) {
    entries.push({
        let tokens = path.iter().map(String::as_str).collect::<Vec<_>>();
        logical_entry(document, node, label, &tokens)
    });
    append_descendant_entries(document, node, path, entries);
    if let Some(comments) = node.tail_comments.as_ref() {
        let tokens = path.iter().map(String::as_str).collect::<Vec<_>>();
        entries.push(tail_entry(document, comments, &tokens));
    }
}

fn logical_entry(
    document: &JsonDocument,
    node: &JsonNode,
    label: String,
    tokens: &[&str],
) -> JsonEntry {
    let (view, summary) = match node.direct_comments.as_ref() {
        Some(comments) => (RefView::DirectComments, comment_summary(document, comments)),
        None => (RefView::Base, None),
    };
    JsonEntry {
        ref_id: canonical_ref_for_view(view, tokens),
        label,
        kind: JsonEntryKind::Value(node.kind()),
        summary,
    }
}

fn tail_entry(document: &JsonDocument, comments: &CommentBundle, tokens: &[&str]) -> JsonEntry {
    JsonEntry {
        ref_id: canonical_ref_for_view(RefView::TailComments, tokens),
        label: TAIL_COMMENTS_LABEL.to_owned(),
        kind: JsonEntryKind::TailComments,
        summary: comment_summary(document, comments),
    }
}

fn comment_summary(document: &JsonDocument, bundle: &CommentBundle) -> Option<String> {
    let mut summary = String::new();
    for &index in bundle.indices() {
        #[cfg(test)]
        document.record_comment_bundle_step();
        let comment = document.comments[index];
        let body = match comment.kind {
            CommentKind::Line => &document.source[comment.span.start + 2..comment.span.end],
            CommentKind::Block => &document.source[comment.span.start + 2..comment.span.end - 2],
        };
        let body = body.split_whitespace().collect::<Vec<_>>().join(" ");
        if body.is_empty() {
            continue;
        }
        if !summary.is_empty() {
            summary.push_str("; ");
        }
        summary.push_str(&body);
    }
    (!summary.is_empty()).then_some(summary)
}

#[cfg(test)]
mod tests;
