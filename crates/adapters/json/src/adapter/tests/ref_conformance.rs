use docnav_adapter_contracts::assert_ref_round_trip;

use super::*;

#[test]
fn strict_json_emitted_refs_round_trip_on_same_and_fresh_compatible_documents() {
    let document = TempDocument::write(
        "strict-ref-conformance.json",
        br#"{
  "object": {
    "first": "strict-hit strict-hit",
    "second": ["strict-hit", {"leaf": "other"}]
  },
  "empty": {}
}"#,
    );
    let definition = crate::json_adapter_definition();
    let mut same_document = definition.create_document(document.path_str().to_owned());

    let outline =
        collect_outline_ref_round_trips(&definition, same_document.as_mut(), &document, 1);
    assert_ref_ids(
        &outline,
        &[
            "json:#/object",
            "json:#/object/first",
            "json:#/object/second",
            "json:#/object/second/0",
            "json:#/object/second/1",
            "json:#/object/second/1/leaf",
            "json:#/empty",
        ],
    );
    assert_round_trip_display(&outline, CONTENT_TYPE_JSON);
    assert_strict_outline_materialization(&outline);

    let find = collect_find_ref_round_trips(
        &definition,
        same_document.as_mut(),
        &document,
        "strict-hit",
        1,
    );
    assert_ref_ids(
        &find,
        &[
            "json:#/object/first",
            "json:#/object/first",
            "json:#/object/second/0",
        ],
    );
    assert_round_trip_display(&find, CONTENT_TYPE_JSON);
    assert_eq!(find[0].1.content, r#""strict-hit strict-hit""#);
    assert_eq!(find[1].1.content, r#""strict-hit strict-hit""#);
    assert_eq!(find[2].1.content, r#""strict-hit""#);
}

#[test]
fn jsonc_emitted_refs_round_trip_on_same_and_fresh_compatible_documents() {
    let document = TempDocument::write(
        "ref-conformance.jsonc",
        r#"/* root-direct-hit */
{
  "very-long-navigation-key-that-forces-label-truncation": "base-hit hit",
  // member-direct-hit
  "member": {
    "ordinary-hit": "snow",
    "pair": [1, 2]
    // member-tail-hit
  },
  "empty": {}
}
// root-tail-hit"#
            .as_bytes(),
    );
    let definition = crate::json_adapter_definition();
    let mut same_document = definition.create_document(document.path_str().to_owned());

    let outline =
        collect_outline_ref_round_trips(&definition, same_document.as_mut(), &document, 1);
    assert!(outline.iter().all(|(entry, _)| !entry.ref_id.is_empty()));
    assert_long_ref_label_is_truncated(&outline);
    assert_contains_refs(
        &outline,
        &[
            "json:comments:#",
            "json:comments:#/member",
            "json:tail-comments:#/member",
            "json:tail-comments:#",
        ],
    );

    let find =
        collect_find_ref_round_trips(&definition, same_document.as_mut(), &document, "hit", 1);
    assert!(find.iter().all(|(entry, _)| entry.label == "."));
    assert_eq!(
        find.iter()
            .filter(|(entry, _)| {
                entry.ref_id == "json:#/very-long-navigation-key-that-forces-label-truncation"
            })
            .count(),
        2,
        "repeated source occurrences must retain the same readable base ref",
    );
    assert_jsonc_find_materialization(&find);
    assert_normalized_container_round_trip(&definition, same_document.as_mut(), &document);
}

#[test]
fn root_scalar_and_tail_only_refs_round_trip_on_compatible_documents() {
    let definition = crate::json_adapter_definition();
    let scalar = TempDocument::write("scalar.json", b"false");
    assert_single_outline_ref_round_trip(
        &definition,
        &scalar,
        entry("json:#", "<root>", "boolean"),
        "false",
    );

    let tail = TempDocument::write("tail-only.jsonc", b"{}\n// document tail");
    assert_single_outline_ref_round_trip(
        &definition,
        &tail,
        entry_with_summary(
            "json:tail-comments:#",
            "<tail comments>",
            "tail_comments",
            "document tail",
        ),
        "// document tail\n{}",
    );
}

fn assert_single_outline_ref_round_trip(
    definition: &AdapterDefinition<'_>,
    source: &TempDocument,
    expected_entry: Entry,
    expected_content: &str,
) {
    let mut document = definition.create_document(source.path_str().to_owned());
    let outline = document
        .outline(&outline_input(source, 1, 100, None))
        .expect("single-ref outline should succeed")
        .into_structured()
        .expect("JSON outline should be structured");
    assert_eq!(
        outline.entries.as_slice(),
        std::slice::from_ref(&expected_entry)
    );

    let (same, fresh) = assert_ref_round_trip(
        definition,
        document.as_mut(),
        &read_input(source, &expected_entry.ref_id, 1, 100),
    );
    assert_eq!(same, fresh);
    assert_eq!(same.content, expected_content);
}

fn assert_round_trip_display(round_trips: &[(Entry, ReadResult)], content_type: &str) {
    assert!(round_trips
        .iter()
        .all(|(entry, read)| entry.label == "." && read.content_type == content_type));
}

fn assert_strict_outline_materialization(round_trips: &[(Entry, ReadResult)]) {
    for (entry, read) in round_trips {
        match entry.ref_id.as_str() {
            "json:#/object" => {
                assert!(read.content.contains(r#""first": "strict-hit strict-hit""#))
            }
            "json:#/object/first" => assert_eq!(read.content, r#""strict-hit strict-hit""#),
            "json:#/object/second" => assert!(read.content.starts_with("[\n")),
            "json:#/object/second/0" => assert_eq!(read.content, r#""strict-hit""#),
            "json:#/object/second/1" => assert_eq!(read.content, "{\n  \"leaf\": \"other\"\n}"),
            "json:#/object/second/1/leaf" => assert_eq!(read.content, r#""other""#),
            "json:#/empty" => assert_eq!(read.content, "{}"),
            ref_id => panic!("unexpected strict JSON outline ref: {ref_id}"),
        }
    }
}

fn assert_long_ref_label_is_truncated(round_trips: &[(Entry, ReadResult)]) {
    assert!(round_trips.iter().any(|(entry, _)| {
        entry.ref_id == "json:#/very-long-navigation-key-that-forces-label-truncation"
            && entry.label == "."
    }));
}

fn assert_contains_refs(round_trips: &[(Entry, ReadResult)], expected: &[&str]) {
    for expected_ref in expected {
        assert!(round_trips
            .iter()
            .any(|(entry, _)| entry.ref_id == *expected_ref));
    }
}

fn assert_jsonc_find_materialization(round_trips: &[(Entry, ReadResult)]) {
    for (entry, read) in round_trips {
        match entry.ref_id.as_str() {
            "json:comments:#" => {
                assert_eq!(read.content_type, CONTENT_TYPE_JSONC);
                assert!(read.content.starts_with("/* root-direct-hit */\n"));
            }
            "json:comments:#/member" => {
                assert_eq!(read.content_type, CONTENT_TYPE_JSONC);
                assert!(read.content.starts_with("// member-direct-hit\n"));
            }
            "json:tail-comments:#/member" => {
                assert_eq!(read.content_type, CONTENT_TYPE_JSONC);
                assert!(read.content.starts_with("// member-tail-hit\n"));
            }
            "json:tail-comments:#" => {
                assert_eq!(read.content_type, CONTENT_TYPE_JSONC);
                assert!(read.content.starts_with("// root-tail-hit\n"));
            }
            "json:#/very-long-navigation-key-that-forces-label-truncation" => {
                assert_eq!(read.content_type, CONTENT_TYPE_JSON);
                assert_eq!(read.content, r#""base-hit hit""#);
            }
            "json:#/member/ordinary-hit" => {
                assert_eq!(read.content_type, CONTENT_TYPE_JSON);
                assert_eq!(read.content, r#""snow""#);
            }
            ref_id => panic!("unexpected conformance ref: {ref_id}"),
        }
    }
}

fn assert_normalized_container_round_trip(
    definition: &AdapterDefinition<'_>,
    same_document: &mut dyn AdapterDocument,
    source: &TempDocument,
) {
    let normalized = same_document
        .find(&find_input(source, "1, 2", 1, 100))
        .expect("container-spanning source evidence should be findable");
    assert_eq!(normalized.matches.len(), 1);
    assert_eq!(normalized.matches[0].ref_id, "json:#/member/pair");
    let (same, fresh) = assert_ref_round_trip(
        definition,
        same_document,
        &read_input(source, &normalized.matches[0].ref_id, 7, 10_000),
    );
    assert_eq!(same, fresh);
    assert_eq!(same.content, "[\n  1,\n  2\n]");
    assert!(!same.content.contains("1, 2"));
}

fn collect_outline_ref_round_trips(
    definition: &AdapterDefinition<'_>,
    same_document: &mut dyn AdapterDocument,
    source: &TempDocument,
    limit: u32,
) -> Vec<(Entry, ReadResult)> {
    let mut page = 1;
    let mut round_trips = Vec::new();

    loop {
        let outline = same_document
            .outline(&outline_input(source, page, limit, None))
            .expect("conformance outline page should succeed")
            .into_structured()
            .expect("JSON outline should be structured");
        let next_page = outline.page.map(|next| next.get());
        for entry in outline.entries {
            let input = read_input(source, &entry.ref_id, 9, 10_000);
            let (same, fresh) = assert_ref_round_trip(definition, same_document, &input);
            assert_eq!(same, fresh);
            round_trips.push((entry, same));
        }

        let Some(next_page) = next_page else {
            break;
        };
        assert_eq!(next_page, page + 1);
        page = next_page;
    }

    let terminal = same_document
        .outline(&outline_input(source, page + 1, limit, None))
        .expect("past-end outline page should remain terminal")
        .into_structured()
        .expect("JSON outline should be structured");
    assert!(terminal.entries.is_empty());
    assert_eq!(terminal.page, None);
    round_trips
}

fn collect_find_ref_round_trips(
    definition: &AdapterDefinition<'_>,
    same_document: &mut dyn AdapterDocument,
    source: &TempDocument,
    query: &str,
    limit: u32,
) -> Vec<(Entry, ReadResult)> {
    let mut page = 1;
    let mut round_trips = Vec::new();

    loop {
        let find = same_document
            .find(&find_input(source, query, page, limit))
            .expect("conformance find page should succeed");
        let next_page = find.page.map(|next| next.get());
        for entry in find.matches {
            let input = read_input(source, &entry.ref_id, 11, 10_000);
            let (same, fresh) = assert_ref_round_trip(definition, same_document, &input);
            assert_eq!(same, fresh);
            round_trips.push((entry, same));
        }

        let Some(next_page) = next_page else {
            break;
        };
        assert_eq!(next_page, page + 1);
        page = next_page;
    }

    let terminal = same_document
        .find(&find_input(source, query, page + 1, limit))
        .expect("past-end find page should remain terminal");
    assert!(terminal.matches.is_empty());
    assert_eq!(terminal.page, None);
    round_trips
}
