use super::*;
use crate::document::{load, wide_comment_per_item_source, JsonKind, WIDE_COMMENT_ITEM_COUNT};

const MIXED_TREE_FIXTURE: &str = include_str!("../../tests/fixtures/mixed-tree.json");
const EMPTY_OBJECT_FIXTURE: &str = include_str!("../../tests/fixtures/empty-object.json");
const EMPTY_ARRAY_FIXTURE: &str = include_str!("../../tests/fixtures/empty-array.json");
const ROOT_SCALAR_FIXTURE: &str = include_str!("../../tests/fixtures/root-scalar.json");

#[test]
fn preorder_entries_preserve_source_order_labels_kinds_and_canonical_refs() {
    let document = load(MIXED_TREE_FIXTURE.as_bytes()).expect("mixed-tree fixture should load");

    let entries = document.preorder_entries();
    let facts = entries
        .iter()
        .map(|entry| (entry.ref_id.as_str(), entry.label.as_str(), entry.kind))
        .collect::<Vec<_>>();

    assert_eq!(
        facts,
        [
            ("json:#/z", "z", JsonEntryKind::Value(JsonKind::Number)),
            ("json:#/a", "a", JsonEntryKind::Value(JsonKind::Array)),
            ("json:#/a/0", "[0]", JsonEntryKind::Value(JsonKind::Number)),
            ("json:#/a/1", "[1]", JsonEntryKind::Value(JsonKind::Object)),
            ("json:#/a/1/b", "b", JsonEntryKind::Value(JsonKind::Number)),
            (
                "json:#/true-value",
                "true-value",
                JsonEntryKind::Value(JsonKind::Boolean)
            ),
            (
                "json:#/null-value",
                "null-value",
                JsonEntryKind::Value(JsonKind::Null)
            ),
            (
                "json:#/string-value",
                "string-value",
                JsonEntryKind::Value(JsonKind::String)
            ),
            (
                "json:#/huge",
                "huge",
                JsonEntryKind::Value(JsonKind::Number)
            ),
            ("json:#/~0", "~", JsonEntryKind::Value(JsonKind::String)),
            ("json:#/~1", "/", JsonEntryKind::Value(JsonKind::String)),
            ("json:#/", "\"\"", JsonEntryKind::Value(JsonKind::String)),
            (
                "json:#/%00control",
                "\0control",
                JsonEntryKind::Value(JsonKind::String)
            ),
            (
                "json:#/%E9%9B%AA",
                "雪",
                JsonEntryKind::Value(JsonKind::String)
            ),
            ("json:#/01", "01", JsonEntryKind::Value(JsonKind::Object)),
            (
                "json:#/01/repeat",
                "repeat",
                JsonEntryKind::Value(JsonKind::String)
            ),
            (
                "json:#/array-context",
                "array-context",
                JsonEntryKind::Value(JsonKind::Array)
            ),
            (
                "json:#/array-context/0",
                "[0]",
                JsonEntryKind::Value(JsonKind::String)
            ),
            (
                "json:#/array-context/1",
                "[1]",
                JsonEntryKind::Value(JsonKind::String)
            ),
            (
                "json:#/cross-node",
                "cross-node",
                JsonEntryKind::Value(JsonKind::Array)
            ),
            (
                "json:#/cross-node/0",
                "[0]",
                JsonEntryKind::Value(JsonKind::String)
            ),
            (
                "json:#/cross-node/1",
                "[1]",
                JsonEntryKind::Value(JsonKind::String)
            ),
            (
                "json:#/empty-object",
                "empty-object",
                JsonEntryKind::Value(JsonKind::Object)
            ),
            (
                "json:#/empty-array",
                "empty-array",
                JsonEntryKind::Value(JsonKind::Array)
            ),
            (
                "json:#/key-hit",
                "key-hit",
                JsonEntryKind::Value(JsonKind::String)
            ),
        ]
    );
    assert!(entries.iter().all(|entry| !entry.label.is_empty()));
}

#[test]
fn preorder_entries_omit_empty_container_roots() {
    for source in [EMPTY_OBJECT_FIXTURE, EMPTY_ARRAY_FIXTURE] {
        let document = load(source.as_bytes()).expect("empty container fixture should load");

        assert!(document.preorder_entries().is_empty());
    }
}

#[test]
fn preorder_entries_keep_a_root_scalar_navigable() {
    let document = load(ROOT_SCALAR_FIXTURE.as_bytes()).expect("root scalar fixture should load");

    assert_eq!(
        document.preorder_entries(),
        [JsonEntry {
            ref_id: "json:#".to_owned(),
            label: "<root>".to_owned(),
            kind: JsonEntryKind::Value(JsonKind::String),
            summary: None,
        }]
    );
}

#[test]
fn preorder_entries_insert_direct_and_tail_comment_entries_in_expanded_tree_order() {
    let document = load(
        br#"/* root direct */
// second root
{
  // member direct
  "member": {
    "child": 1
    /* member tail */
  },
  "array": [
    // index direct
    2,
    {
      "leaf": 3
      // nested tail
    }
    /* array tail */
  ]
  /* root internal tail */
}
/* root document tail */"#,
    )
    .expect("comment-aware traversal fixture should load");

    let entries = document.preorder_entries();
    let facts = entries
        .iter()
        .map(|entry| (entry.ref_id.as_str(), entry.label.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(
        facts,
        [
            ("json:comments:#", "<root>"),
            ("json:comments:#/member", "member"),
            ("json:#/member/child", "child"),
            ("json:tail-comments:#/member", "<tail comments>"),
            ("json:#/array", "array"),
            ("json:comments:#/array/0", "[0]"),
            ("json:#/array/1", "[1]"),
            ("json:#/array/1/leaf", "leaf"),
            ("json:tail-comments:#/array/1", "<tail comments>"),
            ("json:tail-comments:#/array", "<tail comments>"),
            ("json:tail-comments:#", "<tail comments>"),
        ]
    );
}

#[test]
fn preorder_entries_visit_each_comment_bundle_item_once_on_a_wide_comment_corpus() {
    let source = wide_comment_per_item_source();
    let document = load(source.as_bytes()).expect("wide comment corpus should load");

    let entries = document.preorder_entries();

    assert_eq!(document.comments.len(), WIDE_COMMENT_ITEM_COUNT);
    assert_eq!(entries.len(), WIDE_COMMENT_ITEM_COUNT);
    assert_eq!(document.comment_bundle_steps(), document.comments.len());
    assert_eq!(
        entries.last().map(|entry| entry.ref_id.as_str()),
        Some("json:comments:#/1023")
    );
}
