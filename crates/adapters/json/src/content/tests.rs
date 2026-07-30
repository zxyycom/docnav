use super::*;
use crate::document::load;

const MIXED_TREE_FIXTURE: &str = include_str!("../../tests/fixtures/mixed-tree.json");

#[test]
fn structured_root_preserves_source_order_raw_numbers_and_full_cost() {
    let document = load(MIXED_TREE_FIXTURE.as_bytes()).expect("mixed-tree fixture should load");

    let facts = structured_value_facts(&document.root).expect("parsed JSON node should serialize");

    assert_eq!(
        facts.content,
        r#"{
  "z": -0.50e+02,
  "a": [
    1,
    {
      "b": 2
    }
  ],
  "true-value": true,
  "null-value": null,
  "string-value": "scalar-hit",
  "huge": 1e9999,
  "~": "tilde",
  "/": "slash",
  "": "empty",
  "\u0000control": "control",
  "雪": "非 ASCII 🍣",
  "01": {
    "repeat": "repeat repeat"
  },
  "array-context": [
    "zero",
    "one"
  ],
  "cross-node": [
    "left",
    "right"
  ],
  "empty-object": {},
  "empty-array": [],
  "key-hit": "tail"
}"#
    );
    assert_selection_cost(&facts.cost, &facts.content);
}

#[test]
fn structured_selected_values_use_pinned_scalar_escaping_without_trailing_newline() {
    let document = load(
        r#"{"picked":{"escaped":"quote\" slash\/ line\n nul\u0000","unicode":"雪🍣"}}"#.as_bytes(),
    )
    .expect("escaping fixture should load");

    let picked = document
        .resolve_ref("json:#/picked")
        .expect("nested object ref should resolve");
    let picked_facts = structured_value_facts(picked).expect("nested parsed node should serialize");
    assert_eq!(
        picked_facts.content,
        r#"{
  "escaped": "quote\" slash/ line\n nul\u0000",
  "unicode": "雪🍣"
}"#
    );

    let escaped = document
        .resolve_ref("json:#/picked/escaped")
        .expect("nested scalar ref should resolve");
    let escaped_facts =
        structured_value_facts(escaped).expect("scalar parsed node should serialize");
    assert_eq!(
        escaped_facts.content,
        r#""quote\" slash/ line\n nul\u0000""#
    );
    assert!(!picked_facts.content.ends_with('\n'));
    assert!(!escaped_facts.content.ends_with('\n'));
    assert_selection_cost(&picked_facts.cost, &picked_facts.content);
    assert_selection_cost(&escaped_facts.cost, &escaped_facts.content);
}

#[test]
fn full_read_strips_one_bom_only_and_measures_the_actual_source() {
    let bytes = b"\xef\xbb\xbf \r\n{\"text\":\"\\u96ea\"}\n\t";
    let document = load(bytes).expect("BOM-prefixed JSON should load");

    let facts = full_read_facts(&document);

    assert_eq!(facts.content, " \r\n{\"text\":\"\\u96ea\"}\n\t");
    assert_eq!(facts.content, document.source);
    assert_selection_cost(&facts.cost, &facts.content);
}

fn assert_selection_cost(cost: &Cost, content: &str) {
    let actual = cost
        .measurements
        .iter()
        .map(|measurement| {
            (
                measurement.unit.as_str(),
                measurement.value,
                measurement.scope.as_deref(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        actual,
        vec![
            (
                "lines",
                docnav_text_cost::line_cost(content).value,
                Some("selection")
            ),
            (
                "bytes",
                docnav_text_cost::byte_cost(content).value,
                Some("selection")
            ),
            (
                "tokens",
                docnav_text_cost::token_cost(content).value,
                Some("selection")
            ),
        ]
    );
}
