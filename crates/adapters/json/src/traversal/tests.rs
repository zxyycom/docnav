use super::*;
use crate::document::{load, JsonKind};

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
            ("json:#/z", "z", JsonKind::Number),
            ("json:#/a", "a", JsonKind::Array),
            ("json:#/a/0", "[0]", JsonKind::Number),
            ("json:#/a/1", "[1]", JsonKind::Object),
            ("json:#/a/1/b", "b", JsonKind::Number),
            ("json:#/true-value", "true-value", JsonKind::Boolean),
            ("json:#/null-value", "null-value", JsonKind::Null),
            ("json:#/string-value", "string-value", JsonKind::String),
            ("json:#/huge", "huge", JsonKind::Number),
            ("json:#/~0", "~", JsonKind::String),
            ("json:#/~1", "/", JsonKind::String),
            ("json:#/", "\"\"", JsonKind::String),
            ("json:#/%00control", "\0control", JsonKind::String),
            ("json:#/%E9%9B%AA", "雪", JsonKind::String),
            ("json:#/01", "01", JsonKind::Object),
            ("json:#/01/repeat", "repeat", JsonKind::String),
            ("json:#/array-context", "array-context", JsonKind::Array),
            ("json:#/array-context/0", "[0]", JsonKind::String),
            ("json:#/array-context/1", "[1]", JsonKind::String),
            ("json:#/cross-node", "cross-node", JsonKind::Array),
            ("json:#/cross-node/0", "[0]", JsonKind::String),
            ("json:#/cross-node/1", "[1]", JsonKind::String),
            ("json:#/empty-object", "empty-object", JsonKind::Object),
            ("json:#/empty-array", "empty-array", JsonKind::Array),
            ("json:#/key-hit", "key-hit", JsonKind::String),
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
            kind: JsonKind::String,
        }]
    );
}
