use super::*;
use crate::document::{load, JsonDocument};
use docnav_protocol::{positive_result, Location};

const MIXED_TREE_FIXTURE: &str = include_str!("../../tests/fixtures/mixed-tree.json");

#[test]
fn literal_occurrences_are_bom_stripped_case_sensitive_non_overlapping_and_preserved() {
    let document = load(b"\xef\xbb\xbf{\"hit\":\"aaaa HIT hit\"}").expect("valid JSON should load");

    assert_eq!(
        document.source_matches("hit"),
        [
            SourceMatch {
                ref_id: "json:#/hit".to_owned(),
                start: 2,
                end: 5,
            },
            SourceMatch {
                ref_id: "json:#/hit".to_owned(),
                start: 17,
                end: 20,
            },
        ]
    );
    assert_eq!(
        document.source_matches("aa"),
        [
            SourceMatch {
                ref_id: "json:#/hit".to_owned(),
                start: 8,
                end: 10,
            },
            SourceMatch {
                ref_id: "json:#/hit".to_owned(),
                start: 10,
                end: 12,
            },
        ]
    );
    assert_eq!(
        document.source_matches("HIT"),
        [SourceMatch {
            ref_id: "json:#/hit".to_owned(),
            start: 13,
            end: 16,
        }]
    );
    assert!(document.source_matches("Hit").is_empty());
    assert!(document.source_matches("missing").is_empty());
}

#[test]
fn source_regions_map_occurrences_to_the_deepest_canonical_readable_ref() {
    let document = load(b" \n{\"outer\": [  1,\t2  ], \"tail\": 3}\n\t")
        .expect("region mapping fixture should load");

    assert_single_match(&document, "\"outer\"", "json:#/outer");
    assert_single_match(&document, "\"outer\": ", "json:#/outer");
    assert_single_match(&document, "1", "json:#/outer/0");
    assert_single_match(&document, "[  ", "json:#/outer");
    assert_single_match(&document, ",\t", "json:#/outer");
    assert_single_match(&document, "1,\t2", "json:#/outer");
    assert_single_match(&document, "], \"tail\"", "json:#");
    assert_single_match(&document, " \n", "json:#");
    assert_single_match(&document, "\n\t", "json:#");

    let mixed = load(MIXED_TREE_FIXTURE.as_bytes()).expect("mixed-tree fixture should load");
    assert_single_match(&mixed, r"\u0061", "json:#/a");
    assert_single_match(&mixed, "scalar-hit", "json:#/string-value");

    let wide_object_source = format!(
        "{{{}}}",
        (0..1_024)
            .map(|index| format!(r#""key-{index}":"value-{index}""#))
            .collect::<Vec<_>>()
            .join(",")
    );
    let wide_object = load(wide_object_source.as_bytes()).expect("wide object fixture should load");
    assert_single_match(&wide_object, r#""key-997""#, "json:#/key-997");
    assert_single_match(&wide_object, "value-997", "json:#/key-997");
    assert_single_match(&wide_object, r#"value-996","key-997"#, "json:#");

    let wide_array_source = format!(
        "[{}]",
        (0..1_024)
            .map(|index| index.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    let wide_array = load(wide_array_source.as_bytes()).expect("wide array fixture should load");
    assert_single_match(&wide_array, "997", "json:#/997");
    assert_single_match(&wide_array, "996,997", "json:#");
}

#[test]
fn find_entries_emit_nonempty_bounded_unicode_safe_labels() {
    let source = format!(
        r#"{{"value":"{}needle{}"}}"#,
        "界".repeat(120),
        "🙂".repeat(80)
    );
    let document = load(source.as_bytes()).expect("long Unicode fixture should load");

    let entries = document.find_entries("needle").collect::<Vec<_>>();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].ref_id, "json:#/value");
    assert!(entries[0].label.starts_with("..."));
    assert!(entries[0].label.contains("needle"));
    assert!(entries[0].label.ends_with("..."));
    assert_eq!(entries[0].label.chars().count(), 96);
    assert_eq!(entries[0].location, line_location(1));

    let multiline_source = format!(
        "{{\"value\":[\"{}needle\",\r\n\"第二🙂{}\"]}}",
        "界".repeat(120),
        "🙂".repeat(80)
    );
    let multiline =
        load(multiline_source.as_bytes()).expect("multiline Unicode fixture should load");
    let multiline_entries = multiline
        .find_entries("needle\",\r\n\"第二🙂")
        .collect::<Vec<_>>();

    assert_eq!(multiline_entries.len(), 1);
    assert!(multiline_entries[0].label.starts_with("..."));
    assert!(multiline_entries[0].label.contains(r#"needle", "第二🙂"#));
    assert!(multiline_entries[0].label.ends_with("..."));
    assert_eq!(multiline_entries[0].label.chars().count(), 96);

    let blank_line = load(b"\nnull").expect("leading blank line fixture should load");
    assert_eq!(
        blank_line
            .find_entries("\n")
            .next()
            .expect("blank-line query should match")
            .label,
        "."
    );

    let unicode_whitespace =
        load("{\"value\":\"left\u{2003}\u{00a0}needle\u{2009}right\"}".as_bytes())
            .expect("Unicode whitespace fixture should load");
    assert_eq!(
        unicode_whitespace
            .find_entries("needle")
            .next()
            .expect("Unicode whitespace query should match")
            .label,
        r#"{"value":"left needle right"}"#
    );

    let long_query = "🙂".repeat(120);
    let long_query_source = format!(
        r#"{{"value":"{}{}{}"}}"#,
        "a".repeat(120),
        long_query,
        "z".repeat(120)
    );
    let long_query_document =
        load(long_query_source.as_bytes()).expect("long-query fixture should load");
    assert_eq!(
        long_query_document
            .find_entries(&long_query)
            .next()
            .expect("long query should match")
            .label,
        format!("...{}...", "🙂".repeat(90))
    );
}

#[test]
fn find_entries_preserve_repeated_matches_and_report_bom_stripped_crlf_lines() {
    let document = load(b"\xef\xbb\xbf{\r\n  \"value\": \"hit hit\",\r\n  \"tail\": \"hit\"\r\n}")
        .expect("CRLF fixture should load");

    assert_eq!(
        document.find_entries("hit").collect::<Vec<_>>(),
        [
            FindEntry {
                ref_id: "json:#/value".to_owned(),
                label: r#""value": "hit hit","#.to_owned(),
                location: line_location(2),
            },
            FindEntry {
                ref_id: "json:#/value".to_owned(),
                label: r#""value": "hit hit","#.to_owned(),
                location: line_location(2),
            },
            FindEntry {
                ref_id: "json:#/tail".to_owned(),
                label: r#""tail": "hit""#.to_owned(),
                location: line_location(3),
            },
        ]
    );
    assert!(document.find_entries("missing").next().is_none());
}

#[test]
fn find_entries_keep_large_single_line_match_sets_complete_and_bounded() {
    let occurrence_count = 10_000;
    let source = format!(r#"{{"x":"{}"}}"#, "a".repeat(occurrence_count));
    let document = load(source.as_bytes()).expect("large single-line fixture should load");

    let entries = document.find_entries("a").collect::<Vec<_>>();

    assert_eq!(entries.len(), occurrence_count);
    assert!(entries.iter().all(|entry| entry.ref_id == "json:#/x"));
    assert!(entries.iter().all(|entry| {
        !entry.label.is_empty()
            && entry.label.contains('a')
            && entry.label.chars().count() <= MAX_LABEL_CHARS
            && entry.location == line_location(1)
    }));
}

#[test]
fn find_label_working_set_is_bounded_by_the_label_budget() {
    let source = format!(
        r#"{{"value":"{}needle{}"}}"#,
        "界".repeat(10_000),
        "🙂".repeat(10_000)
    );
    let match_start = source.find("needle").expect("fixture should contain query");
    let excerpt = CompactedExcerpt::new(&source, match_start, match_start + "needle".len());

    assert!(excerpt.label().contains("needle"));
    assert!(
        excerpt.buffered_chars() <= MAX_LABEL_CHARS * 4,
        "label construction must not retain one record per source character"
    );
}

#[test]
fn find_label_context_scan_is_bounded_by_raw_unicode_scalars() {
    let source = format!(
        "{}needle{}",
        "\u{2003}".repeat(10_000),
        "\u{00a0}".repeat(10_000)
    );
    let match_start = source.find("needle").expect("fixture should contain query");
    let match_end = match_start + "needle".len();
    let context_start = bounded_context_start(&source, match_start);
    let context_end = bounded_context_end(&source, match_end);

    assert!(
        source[context_start..match_start].chars().count() <= MAX_LABEL_CHARS + 1,
        "leading context scan must have a raw Unicode scalar bound"
    );
    assert!(
        source[match_end..context_end].chars().count() <= MAX_LABEL_CHARS + 1,
        "trailing context scan must have a raw Unicode scalar bound"
    );
    assert_eq!(
        CompactedExcerpt::new(&source, match_start, match_end).label(),
        "needle"
    );
}

fn assert_single_match(document: &JsonDocument, query: &str, expected_ref: &str) {
    let start = document
        .source
        .find(query)
        .expect("query should occur exactly once in the fixture");
    assert_eq!(
        document.source_matches(query),
        [SourceMatch {
            ref_id: expected_ref.to_owned(),
            start,
            end: start + query.len(),
        }]
    );
}

fn line_location(line: u32) -> Location {
    Location {
        line_start: positive_result(line).expect("test line should be positive"),
        line_end: None,
    }
}
