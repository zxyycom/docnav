use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use docnav_adapter_contracts::{
    FindInput, InfoInput, OutlineInput, ReadInput, StandardOperationInput,
};
use docnav_protocol::{
    positive_result, validate_protocol_response_value, Document, Entry, FindResult, Location,
    Operation, OperationArguments, OperationResult, OutlineArguments, OutlineResult,
    ProtocolDiagnosticCode, ProtocolResponse, ReadResult, RequestEnvelope, PROTOCOL_VERSION,
};
use serde_json::json;

use super::*;

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn manifest_declares_fixed_json_identity() {
    let definition = crate::json_adapter_definition();
    let manifest = definition.manifest();

    manifest.validate_semantics().expect("manifest semantics");
    assert_eq!(manifest.adapter.id, "docnav-json");
    assert_eq!(manifest.formats.len(), 1);
    assert_eq!(manifest.formats[0].id, "json");
    assert_eq!(
        manifest.formats[0].extensions,
        [".json", ".code-workspace", ".jsonc"]
    );
    assert_eq!(
        manifest.formats[0].filenames,
        [".prettierrc", ".watchmanconfig"]
    );
    assert_eq!(
        manifest.formats[0].content_types,
        ["application/json", "application/jsonc"]
    );

    let capabilities = definition
        .unstructured_full_read_capabilities()
        .expect("JSON should declare unstructured full-read capabilities");
    assert!(capabilities.content_hook);
    assert_eq!(
        capabilities.cost_measurement_units,
        ["lines", "bytes", "tokens"]
    );
    assert!(!capabilities.result_facts_hook);
}

#[test]
fn selected_outline_parses_actual_document_independently_of_path_hint() {
    let document = TempDocument::write("settings.data", b"\xef\xbb\xbf{\"enabled\":true}\n");
    let input = StandardOperationInput::Outline(outline_input(&document, 1, 100, None));

    let result = crate::json_adapter_definition()
        .execute_operation(&input)
        .expect("selected JSON outline should parse the actual document");

    assert_eq!(
        result,
        OperationResult::Outline(OutlineResult::structured(
            vec![entry("json:#/enabled", "enabled", "boolean")],
            None,
        ))
    );
}

#[test]
fn outline_projects_mixed_json_to_exact_common_entries() {
    let document = TempDocument::write(
        "mixed.json",
        br#"{
            "object": {"nested": 1},
            "array": [true, null],
            "string": "value",
            "number": 2,
            "boolean": false,
            "null": null,
            "": "empty",
            "\u96EA": "snow"
        }"#,
    );

    let result = execute_outline(outline_input(&document, 1, 6_000, Some(i64::MAX)));

    assert_eq!(
        result,
        OutlineResult::structured(
            vec![
                entry("json:#/object", "object", "object"),
                entry("json:#/object/nested", "nested", "number"),
                entry("json:#/array", "array", "array"),
                entry("json:#/array/0", "[0]", "boolean"),
                entry("json:#/array/1", "[1]", "null"),
                entry("json:#/string", "string", "string"),
                entry("json:#/number", "number", "number"),
                entry("json:#/boolean", "boolean", "boolean"),
                entry("json:#/null", "null", "null"),
                entry("json:#/", "\"\"", "string"),
                entry("json:#/%E9%9B%AA", "雪", "string"),
            ],
            None,
        )
    );
    let entries = &result
        .as_structured()
        .expect("JSON outline should be structured")
        .entries;
    assert!(entries.iter().all(|entry| !entry.label.is_empty()));

    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "json-outline-label-test",
        OperationResult::Outline(result),
    );
    let value = serde_json::to_value(response).expect("outline response should serialize");
    validate_protocol_response_value(&value).expect("outline response should satisfy the schema");
}

#[test]
fn outline_handles_empty_container_roots_and_root_scalar() {
    for source in [b"{}".as_slice(), b"[]".as_slice()] {
        let document = TempDocument::write("empty.json", source);

        assert_eq!(
            JsonAdapter
                .outline(&outline_input(&document, 1, 100, None))
                .expect("empty container outline"),
            OutlineResult::structured(Vec::new(), None),
        );
    }

    let scalar = TempDocument::write("scalar.json", b"false");
    assert_eq!(
        JsonAdapter
            .outline(&outline_input(&scalar, 1, 100, None))
            .expect("root scalar outline"),
        OutlineResult::structured(vec![entry("json:#", "<root>", "boolean")], None),
    );
}

#[test]
fn outline_projects_comment_refs_summaries_and_virtual_tail_entries() {
    let document = TempDocument::write(
        "comments.jsonc",
        br#"/* root
   direct */
// second root
{
  // member direct
  "member": {
    "child": 1
    /* member tail */
  },
  "array": [
    // index   direct
    2,
    {
      "leaf": 3
      // nested tail
    }
    /* array
       tail */
  ]
  /* root internal tail */
}
/* root document tail */"#,
    );

    let result = structured_outline(&document, 1, 10_000);

    assert_eq!(
        result.entries,
        [
            entry_with_summary(
                "json:comments:#",
                "<root>",
                "object",
                "root direct; second root",
            ),
            entry_with_summary(
                "json:comments:#/member",
                "member",
                "object",
                "member direct",
            ),
            entry("json:#/member/child", "child", "number"),
            entry_with_summary(
                "json:tail-comments:#/member",
                "<tail comments>",
                "tail_comments",
                "member tail",
            ),
            entry("json:#/array", "array", "array"),
            entry_with_summary("json:comments:#/array/0", "[0]", "number", "index direct",),
            entry("json:#/array/1", "[1]", "object"),
            entry("json:#/array/1/leaf", "leaf", "number"),
            entry_with_summary(
                "json:tail-comments:#/array/1",
                "<tail comments>",
                "tail_comments",
                "nested tail",
            ),
            entry_with_summary(
                "json:tail-comments:#/array",
                "<tail comments>",
                "tail_comments",
                "array tail",
            ),
            entry_with_summary(
                "json:tail-comments:#",
                "<tail comments>",
                "tail_comments",
                "root internal tail; root document tail",
            ),
        ]
    );
    assert_eq!(result.page, None);
    for entry in result
        .entries
        .iter()
        .filter(|entry| entry.kind.as_deref() == Some("tail_comments"))
    {
        assert_eq!(entry.location, None);
        assert_eq!(entry.metadata, None);
        assert_eq!(entry.excerpt, None);
        assert_eq!(entry.rank, None);
        assert_eq!(entry.cost, None);
    }

    let empty_direct = TempDocument::write("empty-direct.jsonc", b"/* */ {}");
    assert_eq!(
        structured_outline(&empty_direct, 1, 100).entries,
        [entry("json:comments:#", "<root>", "object")],
        "an empty normalized body keeps the direct-comment ref but omits summary",
    );

    let tail_only = TempDocument::write("tail-only.jsonc", b"{}\n// document tail");
    assert_eq!(
        structured_outline(&tail_only, 1, 100).entries,
        [entry_with_summary(
            "json:tail-comments:#",
            "<tail comments>",
            "tail_comments",
            "document tail",
        )],
        "a root container with only tail comments must not gain a root logical entry",
    );
}

#[test]
fn outline_comment_summary_budget_shrinks_before_label_and_pages_forward() {
    let document = TempDocument::write(
        "comment-paging.jsonc",
        "{\"very-long\": /* 雪界导航说明 */ 1, \"next\": 2}".as_bytes(),
    );
    let full = structured_outline(&document, 1, 1_000);
    let first = &full.entries[0];
    let limit = first.ref_id.chars().count() + first.label.chars().count() + 5;

    let first_page = structured_outline(
        &document,
        1,
        u32::try_from(limit).expect("test budget fits u32"),
    );
    assert_eq!(
        first_page.entries,
        [entry_with_summary(
            "json:comments:#/very-long",
            "very-long",
            "number",
            "雪界...",
        )]
    );
    assert_eq!(first_page.page.map(|page| page.get()), Some(2));

    let second_page = structured_outline(
        &document,
        2,
        u32::try_from(limit).expect("test budget fits u32"),
    );
    assert_eq!(
        second_page.entries,
        [entry("json:#/next", "next", "number")]
    );
    assert_eq!(second_page.page, None);

    let tiny = structured_outline(&document, 1, 1);
    assert_eq!(
        tiny.entries,
        [entry("json:comments:#/very-long", ".", "number")],
    );
    assert_eq!(tiny.page.map(|page| page.get()), Some(2));
}

#[test]
fn outline_tiny_pages_preserve_complete_refs_and_terminate() {
    let document = TempDocument::write(
        "paging.json",
        br#"{"long-container-name":{"long-child-name":true},"last":null}"#,
    );

    let first = structured_outline(&document, 1, 1);
    assert_eq!(
        first.entries,
        [entry("json:#/long-container-name", ".", "object")]
    );
    assert_eq!(first.page.map(|page| page.get()), Some(2));

    let second = structured_outline(&document, 2, 1);
    assert_eq!(
        second.entries,
        [entry(
            "json:#/long-container-name/long-child-name",
            ".",
            "boolean",
        )]
    );
    assert_eq!(second.page.map(|page| page.get()), Some(3));

    let third = structured_outline(&document, 3, 1);
    assert_eq!(third.entries, [entry("json:#/last", ".", "null")]);
    assert_eq!(third.page, None);

    let past_end = structured_outline(&document, 4, 1);
    assert!(past_end.entries.is_empty());
    assert_eq!(past_end.page, None);
}

#[test]
fn selected_outline_maps_current_document_failures_to_stable_diagnostics() {
    let missing = TempDocument::write("missing.json", b"{}");
    let selected = crate::json_adapter_definition();
    fs::remove_file(&missing.path).expect("remove selected document");
    assert_protocol_error(
        &selected_outline_error(&selected, &missing),
        "DOCUMENT_NOT_FOUND",
        json!({ "path": missing.path_str() }),
    );

    let invalid_utf8 = TempDocument::write("encoding.json", b"{}");
    let selected = crate::json_adapter_definition();
    fs::write(&invalid_utf8.path, b"{\"value\":\"\xff\"}")
        .expect("replace selected document with non-UTF-8 bytes");
    assert_protocol_error(
        &selected_outline_error(&selected, &invalid_utf8),
        "DOCUMENT_ENCODING_UNSUPPORTED",
        json!({
            "path": invalid_utf8.path_str(),
            "encoding": "non-utf-8",
        }),
    );

    let unreadable = read_error(
        "/normalized/unreadable.json",
        std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "secret operating-system attachment",
        ),
    )
    .protocol_error();
    assert_protocol_error(
        &unreadable,
        "DOCUMENT_PATH_INVALID",
        json!({
            "path": "/normalized/unreadable.json",
            "reason": "document path could not be read",
        }),
    );

    let invalid_content = [
        (
            "syntax.json",
            br#"{"value":}"#.to_vec(),
            "JSON_SYNTAX_INVALID",
        ),
        (
            "trailing.json",
            b"{} trailing".to_vec(),
            "JSON_TRAILING_INPUT",
        ),
        (
            "multiple-roots.jsonc",
            b"{} /* accepted trivia */ []".to_vec(),
            "JSON_TRAILING_INPUT",
        ),
        (
            "unterminated-comment.jsonc",
            b"{/* secret parser attachment".to_vec(),
            "JSON_SYNTAX_INVALID",
        ),
        (
            "json5.jsonc",
            b"{unquoted:'value', hexadecimal:0x10}".to_vec(),
            "JSON_SYNTAX_INVALID",
        ),
        (
            "missing-comma.jsonc",
            br#"{"first":1 "second":2}"#.to_vec(),
            "JSON_SYNTAX_INVALID",
        ),
        (
            "doubled-comma.jsonc",
            br#"{"first":1,,"second":2}"#.to_vec(),
            "JSON_SYNTAX_INVALID",
        ),
        (
            "empty-object-comma.jsonc",
            b"{,}".to_vec(),
            "JSON_SYNTAX_INVALID",
        ),
        (
            "empty-array-comma.jsonc",
            b"[,]".to_vec(),
            "JSON_SYNTAX_INVALID",
        ),
        (
            "trailing-empty-object-comma.jsonc",
            b"1 {,}".to_vec(),
            "JSON_TRAILING_INPUT",
        ),
        (
            "trailing-empty-array-comma.jsonc",
            b"1 [,]".to_vec(),
            "JSON_TRAILING_INPUT",
        ),
        (
            "duplicate.json",
            br#"{"secret-duplicate-name":1,"secret-\u0064uplicate-name":2}"#.to_vec(),
            "JSON_DUPLICATE_MEMBER",
        ),
        (
            "depth.json",
            format!("{}[]{}", "[".repeat(128), "]".repeat(128)).into_bytes(),
            "JSON_MAXIMUM_DEPTH_EXCEEDED",
        ),
    ];
    for (name, bytes, reason) in invalid_content {
        let document = TempDocument::write(name, b"{}");
        let selected = crate::json_adapter_definition();
        fs::write(&document.path, bytes).expect("replace selected document with invalid JSON");

        assert_protocol_error(
            &selected_outline_error(&selected, &document),
            "DOCUMENT_CONTENT_INVALID",
            json!({
                "path": document.path_str(),
                "reason": reason,
            }),
        );
    }
}

#[test]
fn read_round_trips_outline_refs_and_formats_selected_values() {
    let document = TempDocument::write(
        "read.json",
        r#"{
            "nested": {
                "huge": 1e9999,
                "empty-object": {},
                "empty-array": []
            },
            "a/b~c": "雪",
            "scalar": false
        }"#
        .as_bytes(),
    );
    let outline = structured_outline(&document, 1, 10_000);

    for entry in &outline.entries {
        let result = read_result(&document, &entry.ref_id, 1, 10_000);
        assert_eq!(result.ref_id, entry.ref_id);
        assert_eq!(result.content_type, CONTENT_TYPE_JSON);
        assert_eq!(result.page, None);
    }

    assert_eq!(
        read_result(&document, "json:#", 1, 10_000).content,
        r#"{
  "nested": {
    "huge": 1e9999,
    "empty-object": {},
    "empty-array": []
  },
  "a/b~c": "雪",
  "scalar": false
}"#
    );
    assert_eq!(
        read_result(&document, "json:#/nested", 1, 10_000).content,
        r#"{
  "huge": 1e9999,
  "empty-object": {},
  "empty-array": []
}"#
    );
    assert_eq!(
        read_result(&document, "json:#/nested/huge", 1, 10_000).content,
        "1e9999"
    );
    assert_eq!(
        read_result(&document, "json:#/nested/empty-object", 1, 10_000).content,
        "{}"
    );
    assert_eq!(
        read_result(&document, "json:#/nested/empty-array", 1, 10_000).content,
        "[]"
    );
    assert_eq!(
        read_result(&document, "json:#/a~1b~0c", 1, 10_000).content,
        r#""雪""#
    );
    assert_eq!(
        read_result(&document, "json:#/scalar", 1, 10_000).content,
        "false"
    );

    let root_scalar = TempDocument::write("root-scalar.json", b"-0.50e+9999");
    assert_eq!(
        read_result(&root_scalar, "json:#", 1, 10_000).content,
        "-0.50e+9999"
    );
}

#[test]
fn read_maps_invalid_and_missing_refs_to_distinct_diagnostics() {
    let document = TempDocument::write("refs.json", br#"{"items":["zero"]}"#);

    for (ref_id, expected_reason) in [
        ("markdown:doc", "expected ref to start with json:#"),
        (
            "json:#/items/01",
            "expected a canonical nonnegative array index",
        ),
        (
            "json:unknown-comments:#/items",
            "expected ref to start with json:#",
        ),
        (
            "json:comments:#/items/~2",
            "expected ~0 or ~1 JSON Pointer escape",
        ),
    ] {
        let error = JsonAdapter
            .read(&read_input(&document, ref_id, 1, 100))
            .expect_err("invalid ref should fail")
            .protocol_error();

        assert_eq!(error.code(), ProtocolDiagnosticCode::RefInvalid);
        assert_eq!(error.details().get("ref"), Some(&json!(ref_id)));
        assert_eq!(error.details().get("reason"), Some(&json!(expected_reason)));
    }

    for ref_id in [
        "json:#/items/9",
        "json:comments:#/items",
        "json:tail-comments:#/items",
    ] {
        let error = JsonAdapter
            .read(&read_input(&document, ref_id, 1, 100))
            .expect_err("missing canonical ref or comment bundle should fail")
            .protocol_error();

        assert_eq!(error.code(), ProtocolDiagnosticCode::RefNotFound);
        assert_eq!(error.details().get("ref"), Some(&json!(ref_id)));
        assert_eq!(error.details().get("reason"), None);
        assert_eq!(error.details().len(), 1);
    }
}

#[test]
fn read_paginates_unicode_and_keeps_complete_cost_on_every_page() {
    let document = TempDocument::write("unicode.json", r#"{"text":"雪🍣界🙂文档导航"}"#.as_bytes());
    let expected_content = r#""雪🍣界🙂文档导航""#;
    let mut requested_page = 1;
    let mut reconstructed = String::new();
    let mut expected_cost = None;

    loop {
        let result = read_result(&document, "json:#/text", requested_page, 2);

        assert_eq!(result.ref_id, "json:#/text");
        assert_eq!(result.content_type, CONTENT_TYPE_JSON);
        assert!(result.content.chars().count() <= 2);
        if let Some(cost) = &expected_cost {
            assert_eq!(&result.cost, cost);
        } else {
            assert_selection_cost(&result.cost, expected_content);
            expected_cost = Some(result.cost.clone());
        }
        reconstructed.push_str(&result.content);

        let Some(next_page) = result.page else {
            break;
        };
        assert_eq!(next_page.get(), requested_page + 1);
        requested_page = next_page.get();
    }

    assert_eq!(reconstructed, expected_content);

    let past_end = read_result(&document, "json:#/text", requested_page + 1, 2);
    assert_eq!(past_end.content, "");
    assert_eq!(past_end.cost, expected_cost.expect("at least one page"));
    assert_eq!(past_end.page, None);
}

#[test]
fn read_projects_base_and_comment_views_from_only_the_selected_frame() {
    let document = TempDocument::write(
        "comment-views.jsonc",
        r#"/* root direct */
{
  "": /* empty direct */ [
    // index direct
    "雪" // index suffix
    /* array tail */
  ]
} // root suffix
/* document tail */"#
            .as_bytes(),
    );
    let base = r#"{
  "": [
    "雪"
  ]
}"#;
    let root_direct = format!("/* root direct */\n// root suffix\n{base}");
    let empty_key_direct = "/* empty direct */\n[\n  \"雪\"\n]";
    let index_direct = "// index direct\n// index suffix\n\"雪\"";
    let empty_key_tail = "/* array tail */\n[\n  \"雪\"\n]";
    let root_tail = format!("/* document tail */\n{base}");

    for (ref_id, expected_content, expected_type) in [
        ("json:#", base, CONTENT_TYPE_JSON),
        ("json:comments:#", root_direct.as_str(), CONTENT_TYPE_JSONC),
        ("json:comments:#/", empty_key_direct, CONTENT_TYPE_JSONC),
        ("json:comments:#//0", index_direct, CONTENT_TYPE_JSONC),
        ("json:tail-comments:#/", empty_key_tail, CONTENT_TYPE_JSONC),
        (
            "json:tail-comments:#",
            root_tail.as_str(),
            CONTENT_TYPE_JSONC,
        ),
    ] {
        let result = read_result(&document, ref_id, 1, 10_000);
        assert_eq!(result.ref_id, ref_id);
        assert_eq!(result.content, expected_content, "ref: {ref_id}");
        assert_eq!(result.content_type, expected_type, "ref: {ref_id}");
        assert_eq!(result.page, None, "ref: {ref_id}");
        assert_selection_cost(&result.cost, expected_content);
    }

    let mut page = 1;
    let mut reconstructed = String::new();
    let mut expected_cost = None;
    loop {
        let result = read_result(&document, "json:comments:#//0", page, 2);
        assert_eq!(result.content_type, CONTENT_TYPE_JSONC);
        assert!(result.content.chars().count() <= 2);
        assert_selection_cost(&result.cost, index_direct);
        if let Some(cost) = &expected_cost {
            assert_eq!(&result.cost, cost);
        } else {
            expected_cost = Some(result.cost.clone());
        }
        reconstructed.push_str(&result.content);

        let Some(next_page) = result.page else {
            break;
        };
        assert_eq!(next_page.get(), page + 1);
        page = next_page.get();
    }
    assert_eq!(reconstructed, index_direct);
}

#[test]
fn find_projects_mixed_occurrences_and_round_trips_every_ref_through_read() {
    let document = TempDocument::write(
        "find.json",
        b"\xef\xbb\xbf{\n  \"key-hit\": \"key value\",\n  \"scalar\": \"scalar hit\",\n  \"repeat\": \"hit hit\",\n  \"pair\": [1, 2],\n  \"tail\": true\n}\n",
    );

    let result = find_result(&document, "hit", 1, 10_000);

    assert_eq!(
        result,
        FindResult::new(
            vec![
                match_entry("json:#/key-hit", r#""key-hit": "key value","#, 2,),
                match_entry("json:#/scalar", r#""scalar": "scalar hit","#, 3,),
                match_entry("json:#/repeat", r#""repeat": "hit hit","#, 4),
                match_entry("json:#/repeat", r#""repeat": "hit hit","#, 4),
            ],
            None,
        )
    );
    for (entry, expected_content) in result.matches.iter().zip([
        r#""key value""#,
        r#""scalar hit""#,
        r#""hit hit""#,
        r#""hit hit""#,
    ]) {
        assert_eq!(
            read_result(&document, &entry.ref_id, 1, 10_000).content,
            expected_content
        );
    }

    let structure = find_result(&document, "[1", 1, 10_000);
    assert_eq!(
        structure,
        FindResult::new(
            vec![match_entry("json:#/pair", r#""pair": [1, 2],"#, 5)],
            None,
        )
    );
    let cross_child = find_result(&document, "1, 2", 1, 10_000);
    assert_eq!(cross_child, structure);
    assert_eq!(
        read_result(&document, &cross_child.matches[0].ref_id, 1, 10_000).content,
        "[\n  1,\n  2\n]"
    );

    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "json-find-projection-test",
        OperationResult::Find(result),
    );
    let value = serde_json::to_value(response).expect("find response should serialize");
    validate_protocol_response_value(&value).expect("find response should satisfy the schema");
}

#[test]
fn find_maps_comment_occurrences_to_comment_views_and_preserves_find_facts() {
    let document = TempDocument::write(
        "comment-find.jsonc",
        r#"// root-direct-hit
{
  // member-direct-hit
  "member": {
    "ordinary-hit": "雪"
    // member-tail-hit
  }
}
// root-tail-hit"#
            .as_bytes(),
    );

    let result = find_result(&document, "hit", 1, 10_000);
    assert_eq!(
        result.matches,
        [
            match_entry("json:comments:#", "// root-direct-hit", 1),
            match_entry("json:comments:#/member", "// member-direct-hit", 3),
            match_entry("json:#/member/ordinary-hit", r#""ordinary-hit": "雪""#, 5),
            match_entry("json:tail-comments:#/member", "// member-tail-hit", 6),
            match_entry("json:tail-comments:#", "// root-tail-hit", 9),
        ]
    );

    for (entry, expected_content, expected_type) in [
        (
            &result.matches[0],
            r#"// root-direct-hit
{
  "member": {
    "ordinary-hit": "雪"
  }
}"#,
            CONTENT_TYPE_JSONC,
        ),
        (
            &result.matches[1],
            r#"// member-direct-hit
{
  "ordinary-hit": "雪"
}"#,
            CONTENT_TYPE_JSONC,
        ),
        (&result.matches[2], r#""雪""#, CONTENT_TYPE_JSON),
        (
            &result.matches[3],
            r#"// member-tail-hit
{
  "ordinary-hit": "雪"
}"#,
            CONTENT_TYPE_JSONC,
        ),
        (
            &result.matches[4],
            r#"// root-tail-hit
{
  "member": {
    "ordinary-hit": "雪"
  }
}"#,
            CONTENT_TYPE_JSONC,
        ),
    ] {
        let read = read_result(&document, &entry.ref_id, 1, 10_000);
        assert_eq!(read.content, expected_content, "ref: {}", entry.ref_id);
        assert_eq!(read.content_type, expected_type, "ref: {}", entry.ref_id);
    }

    let first = find_result(&document, "hit", 1, 1);
    assert_eq!(first.matches, [match_entry("json:comments:#", ".", 1)]);
    assert_eq!(first.page.map(|page| page.get()), Some(2));
    let second = find_result(&document, "hit", 2, 1);
    assert_eq!(
        second.matches,
        [match_entry("json:comments:#/member", ".", 3)]
    );
    assert_eq!(second.page.map(|page| page.get()), Some(3));
}

#[test]
fn find_tiny_pages_preserve_occurrences_complete_refs_and_terminal_no_match() {
    let document = TempDocument::write(
        "find-pages.json",
        b"{\n  \"value\": \"hit hit\",\n  \"tail\": \"hit\"\n}",
    );
    let expected = [
        match_entry("json:#/value", ".", 2),
        match_entry("json:#/value", ".", 2),
        match_entry("json:#/tail", ".", 3),
    ];
    let mut requested_page = 1;
    let mut actual = Vec::new();

    loop {
        let result = find_result(&document, "hit", requested_page, 1);

        assert_eq!(result.matches, [expected[actual.len()].clone()]);
        actual.push(result.matches[0].clone());

        let Some(next_page) = result.page else {
            break;
        };
        assert_eq!(next_page.get(), requested_page + 1);
        requested_page = next_page.get();
    }

    assert_eq!(actual, expected);
    assert_eq!(
        find_result(&document, "hit", requested_page + 1, 1),
        FindResult::new(Vec::new(), None)
    );
    assert_eq!(
        find_result(&document, "missing", 1, 1),
        FindResult::new(Vec::new(), None)
    );
}

#[test]
fn find_rejects_an_empty_query_with_the_existing_invalid_request_diagnostic() {
    let document = TempDocument::write("empty-query.json", b"{}");

    let error = JsonAdapter
        .find(&find_input(&document, "", 1, 100))
        .expect_err("empty query should fail")
        .protocol_error();

    assert_eq!(error.code(), ProtocolDiagnosticCode::InvalidRequest);
    assert_eq!(
        error.details().get("field"),
        Some(&json!("arguments.query"))
    );
    assert_eq!(
        error.details().get("reason"),
        Some(&json!("query must not be empty"))
    );
}

#[test]
fn info_reports_exact_bom_aware_document_and_nested_metadata() {
    let bytes = b"\xef\xbb\xbf{\"array\":[1,{\"leaf\":null}],\"empty\":{}}";
    let document = TempDocument::write("info-nested.json", bytes);

    let info = execute_info(info_input(&document));

    assert_eq!(
        info.document,
        Some(docnav_protocol::InfoDocument {
            content_type: Some("application/json".to_owned()),
            encoding: Some("UTF-8".to_owned()),
            size: Some(docnav_protocol::Measurement {
                unit: "bytes".to_owned(),
                value: bytes.len() as u64,
                scope: None,
            }),
        })
    );
    assert_eq!(
        info.adapter,
        Some(docnav_protocol::InfoAdapter {
            id: Some("docnav-json".to_owned()),
            format: Some("json".to_owned()),
        })
    );
    assert_eq!(
        info.metadata,
        Some(serde_json::Map::from_iter([
            ("root_kind".to_owned(), json!("object")),
            ("node_count".to_owned(), json!(6)),
            ("max_depth".to_owned(), json!(3)),
        ]))
    );

    for (name, source, expected_content_type) in [
        (
            "strict-string-markers.json",
            br#"{"line":"// not a comment","block":"/* not a comment */","comma":"value,}"}"#
                .as_slice(),
            CONTENT_TYPE_JSON,
        ),
        (
            "comments-only.jsonc",
            b"/* accepted comment */\n{\"value\":1}".as_slice(),
            CONTENT_TYPE_JSONC,
        ),
        (
            "trailing-comma-only.jsonc",
            b"{\"value\":1,}".as_slice(),
            CONTENT_TYPE_JSONC,
        ),
    ] {
        let document = TempDocument::write(name, source);
        let info = execute_info(info_input(&document));

        assert_eq!(
            info.document
                .as_ref()
                .and_then(|document| document.content_type.as_deref()),
            Some(expected_content_type),
            "source: {}",
            String::from_utf8_lossy(source),
        );
        assert_eq!(
            info.adapter
                .as_ref()
                .and_then(|adapter| adapter.format.as_deref()),
            Some(FORMAT_ID_JSON),
        );
    }
}

#[test]
fn info_reports_every_root_kind_with_root_depth_zero() {
    let cases: &[(&[u8], &str)] = &[
        (b"{}", "object"),
        (b"[]", "array"),
        (br#""value""#, "string"),
        (b"-0.50e+9999", "number"),
        (b"true", "boolean"),
        (b"null", "null"),
    ];

    for (source, expected_kind) in cases {
        let document = TempDocument::write("info-root.json", source);
        let info = execute_info(info_input(&document));

        assert_eq!(
            info.metadata,
            Some(serde_json::Map::from_iter([
                ("root_kind".to_owned(), json!(expected_kind)),
                ("node_count".to_owned(), json!(1)),
                ("max_depth".to_owned(), json!(0)),
            ])),
            "source: {}",
            String::from_utf8_lossy(source),
        );
        assert_eq!(
            info.document
                .as_ref()
                .and_then(|facts| facts.size.as_ref())
                .map(|size| size.value),
            Some(source.len() as u64),
        );
    }
}

#[test]
fn full_read_hooks_preserve_bom_stripped_source_and_measure_actual_cost() {
    for (name, expected_source, expected_content_type) in [
        (
            "full-read.json",
            " \r\n{\"line\":\"// marker\",\"block\":\"/* marker */\",\"items\":[]}\n\t",
            CONTENT_TYPE_JSON,
        ),
        (
            "full-read-comments.jsonc",
            "/* exact comment */\r\n{\"text\":\"\\u96ea\"}\n",
            CONTENT_TYPE_JSONC,
        ),
        (
            "full-read-trailing-comma.jsonc",
            "{\n  \"items\": [1, 2,],\n}\n",
            CONTENT_TYPE_JSONC,
        ),
    ] {
        let mut bytes = b"\xef\xbb\xbf".to_vec();
        bytes.extend_from_slice(expected_source.as_bytes());
        let document = TempDocument::write(name, &bytes);
        let request = full_read_request(&document);

        let expected = JsonAdapter
            .unstructured_full_read(&request)
            .expect("full read should succeed");
        let definition = crate::json_adapter_definition();
        let result = definition
            .unstructured_full_read(&request)
            .expect("definition full read should succeed");

        assert_eq!(result, expected);
        assert_eq!(result.content, expected_source);
        assert_eq!(result.content_type, expected_content_type);
        let full_cost = result
            .facts
            .cost
            .expect("full read should return cost facts");
        assert_selection_cost(&full_cost, expected_source);

        let requested_units = ["tokens".to_owned(), "bytes".to_owned()];
        let expected_measured = JsonAdapter
            .measure_unstructured_full_read_cost(&request, &requested_units)
            .expect("full-read cost measurement should succeed");
        let measured = definition
            .measure_unstructured_full_read_cost(&request, &requested_units)
            .expect("definition full-read cost measurement should succeed");
        assert_eq!(measured, expected_measured);
        let mut expected_measurements = full_cost.measurements;
        expected_measurements
            .retain(|measurement| matches!(measurement.unit.as_str(), "bytes" | "tokens"));
        assert_eq!(measured.measurements, expected_measurements);
    }
}

fn outline_input(
    document: &TempDocument,
    page: u32,
    limit: u32,
    max_heading_level: Option<i64>,
) -> OutlineInput {
    OutlineInput {
        document_path: document.path_str().to_owned(),
        page: positive_result(page).expect("positive page"),
        limit: positive_result(limit).expect("positive limit"),
        max_heading_level,
    }
}

fn read_input(document: &TempDocument, ref_id: &str, page: u32, limit: u32) -> ReadInput {
    ReadInput {
        document_path: document.path_str().to_owned(),
        ref_id: ref_id.to_owned(),
        page: positive_result(page).expect("positive page"),
        limit: positive_result(limit).expect("positive limit"),
    }
}

fn find_input(document: &TempDocument, query: &str, page: u32, limit: u32) -> FindInput {
    FindInput {
        document_path: document.path_str().to_owned(),
        query: query.to_owned(),
        page: positive_result(page).expect("positive page"),
        limit: positive_result(limit).expect("positive limit"),
        max_heading_level: None,
    }
}

fn info_input(document: &TempDocument) -> InfoInput {
    InfoInput {
        document_path: document.path_str().to_owned(),
    }
}

fn full_read_request(document: &TempDocument) -> RequestEnvelope {
    RequestEnvelope {
        protocol_version: PROTOCOL_VERSION.to_owned(),
        request_id: "json-full-read-test".to_owned(),
        operation: Operation::Outline,
        document: Document {
            path: document.path_str().to_owned(),
        },
        arguments: OperationArguments::Outline(OutlineArguments {
            limit: positive_result(100).expect("positive limit"),
            page: positive_result(1).expect("positive page"),
            options: None,
        }),
    }
}

fn read_result(document: &TempDocument, ref_id: &str, page: u32, limit: u32) -> ReadResult {
    execute_read(read_input(document, ref_id, page, limit))
}

fn find_result(document: &TempDocument, query: &str, page: u32, limit: u32) -> FindResult {
    execute_find(find_input(document, query, page, limit))
}

fn assert_selection_cost(cost: &docnav_protocol::Cost, content: &str) {
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

fn structured_outline(
    document: &TempDocument,
    page: u32,
    limit: u32,
) -> docnav_protocol::StructuredOutlineResult {
    execute_outline(outline_input(document, page, limit, None))
        .into_structured()
        .expect("JSON outline should be structured")
}

fn execute_outline(input: OutlineInput) -> OutlineResult {
    let expected = JsonAdapter
        .outline(&input)
        .expect("direct Adapter outline strategy should succeed");
    let actual = crate::json_adapter_definition()
        .execute_operation(&StandardOperationInput::Outline(input))
        .expect("definition outline should succeed");
    let OperationResult::Outline(actual) = actual else {
        panic!("outline input should return an outline result");
    };
    assert_eq!(actual, expected);
    actual
}

fn execute_read(input: ReadInput) -> ReadResult {
    let expected = JsonAdapter
        .read(&input)
        .expect("direct Adapter read strategy should succeed");
    let actual = crate::json_adapter_definition()
        .execute_operation(&StandardOperationInput::Read(input))
        .expect("definition read should succeed");
    let OperationResult::Read(actual) = actual else {
        panic!("read input should return a read result");
    };
    assert_eq!(actual, expected);
    actual
}

fn execute_find(input: FindInput) -> FindResult {
    let expected = JsonAdapter
        .find(&input)
        .expect("direct Adapter find strategy should succeed");
    let actual = crate::json_adapter_definition()
        .execute_operation(&StandardOperationInput::Find(input))
        .expect("definition find should succeed");
    let OperationResult::Find(actual) = actual else {
        panic!("find input should return a find result");
    };
    assert_eq!(actual, expected);
    actual
}

fn execute_info(input: InfoInput) -> docnav_protocol::InfoResult {
    let expected = JsonAdapter
        .info(&input)
        .expect("direct Adapter info strategy should succeed");
    let actual = crate::json_adapter_definition()
        .execute_operation(&StandardOperationInput::Info(input))
        .expect("definition info should succeed");
    let OperationResult::Info(actual) = actual else {
        panic!("info input should return an info result");
    };
    assert_eq!(actual, expected);
    actual
}

fn entry(ref_id: &str, label: &str, kind: &str) -> Entry {
    Entry {
        ref_id: ref_id.to_owned(),
        label: label.to_owned(),
        kind: Some(kind.to_owned()),
        location: None,
        summary: None,
        excerpt: None,
        rank: None,
        cost: None,
        metadata: None,
    }
}

fn entry_with_summary(ref_id: &str, label: &str, kind: &str, summary: &str) -> Entry {
    Entry {
        summary: Some(summary.to_owned()),
        ..entry(ref_id, label, kind)
    }
}

fn match_entry(ref_id: &str, label: &str, line: u32) -> Entry {
    Entry {
        ref_id: ref_id.to_owned(),
        label: label.to_owned(),
        kind: Some("match".to_owned()),
        location: Some(Location {
            line_start: positive_result(line).expect("positive line"),
            line_end: None,
        }),
        summary: None,
        excerpt: None,
        rank: None,
        cost: None,
        metadata: None,
    }
}

fn selected_outline_error(
    selected: &docnav_adapter_contracts::AdapterDefinition<'_>,
    document: &TempDocument,
) -> docnav_protocol::ProtocolError {
    selected
        .execute_operation(&StandardOperationInput::Outline(outline_input(
            document, 1, 100, None,
        )))
        .expect_err("selected JSON outline should reject the current document")
        .protocol_error()
}

fn assert_protocol_error(
    error: &docnav_protocol::ProtocolError,
    expected_code: &str,
    expected_details: serde_json::Value,
) {
    assert_eq!(error.code().protocol_code(), expected_code);
    assert_eq!(
        serde_json::to_value(error.details()).expect("protocol error details should serialize"),
        expected_details,
    );
}

struct TempDocument {
    directory: PathBuf,
    path: PathBuf,
}

impl TempDocument {
    fn write(name: &str, bytes: &[u8]) -> Self {
        let document = Self::missing(name);
        fs::create_dir(&document.directory).expect("create temporary document directory");
        fs::write(&document.path, bytes).expect("write temporary document");
        document
    }

    fn missing(name: &str) -> Self {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let directory =
            std::env::temp_dir().join(format!("docnav-json-adapter-{}-{id}", std::process::id()));
        let path = directory.join(name);
        Self { directory, path }
    }

    fn path_str(&self) -> &str {
        self.path
            .to_str()
            .expect("temporary document path should be UTF-8")
    }
}

impl Drop for TempDocument {
    fn drop(&mut self) {
        remove_temporary_directory(&self.directory);
    }
}

fn remove_temporary_directory(path: &Path) {
    match fs::remove_dir_all(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("remove temporary document directory: {error}"),
    }
}
