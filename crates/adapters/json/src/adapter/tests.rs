use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use docnav_adapter_contracts::{
    FindInput, InfoInput, OutlineInput, ReadInput, StandardOperationInput,
};
use docnav_protocol::{
    positive_result, validate_protocol_response_value, Document, Entry, FindResult, Location,
    Operation, OperationArguments, OperationResult, OutlineArguments, OutlineResult,
    ProbeReasonCode, ProtocolDiagnosticCode, ProtocolResponse, ReadResult, RequestEnvelope,
    PROTOCOL_VERSION,
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
    assert_eq!(manifest.formats[0].extensions, [".json"]);
    assert_eq!(manifest.formats[0].content_types, ["application/json"]);

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
fn probe_checks_extension_case_insensitively_and_returns_ordered_success_evidence() {
    let document = TempDocument::write("settings.JSON", br#"{"enabled":true}"#);

    let expected = JsonAdapter.probe(document.path_str());
    let result = crate::json_adapter_definition().probe(document.path_str());

    assert_eq!(result, expected);
    result.validate_semantics().expect("probe semantics");
    assert!(result.supported);
    assert_eq!(result.adapter_id, "docnav-json");
    assert_eq!(result.path, document.path_str());
    assert_eq!(result.format.as_deref(), Some("json"));
    assert_eq!(result.confidence, 1.0);
    assert_eq!(
        result
            .reasons
            .iter()
            .map(|reason| reason.code)
            .collect::<Vec<_>>(),
        [
            ProbeReasonCode::ExtensionMatch,
            ProbeReasonCode::ContentMatch
        ]
    );
    assert!(result
        .reasons
        .iter()
        .all(|reason| !reason.detail.is_empty()));
}

#[test]
fn probe_short_circuits_extension_mismatch_before_io_but_reads_json_candidates() {
    let missing_markdown = TempDocument::missing("missing.md");
    let mismatch = JsonAdapter.probe(missing_markdown.path_str());

    assert!(!mismatch.supported);
    assert_eq!(mismatch.format, None);
    assert_eq!(mismatch.confidence, 0.0);
    assert_eq!(mismatch.reasons.len(), 1);
    assert_eq!(mismatch.reasons[0].code, ProbeReasonCode::ContentConflict);
    assert_eq!(
        mismatch.reasons[0].detail,
        "path extension is not declared for JSON"
    );

    let missing_json = TempDocument::missing("missing.json");
    let read_failure = JsonAdapter.probe(missing_json.path_str());

    assert!(!read_failure.supported);
    assert_eq!(
        read_failure
            .reasons
            .iter()
            .map(|reason| reason.code)
            .collect::<Vec<_>>(),
        [ProbeReasonCode::ExtensionMatch, ProbeReasonCode::ReadError]
    );
    assert!(!read_failure.reasons[1].detail.is_empty());
}

#[test]
fn probe_accepts_one_utf8_bom() {
    let document = TempDocument::write("bom.json", b"\xef\xbb\xbf [1, 2]\n");

    let result = JsonAdapter.probe(document.path_str());

    assert!(result.supported);
    assert_eq!(result.format.as_deref(), Some("json"));
    assert_eq!(result.confidence, 1.0);
}

#[test]
fn probe_maps_loader_failures_to_unsupported_diagnostics() {
    let invalid_utf8 = TempDocument::write("encoding.json", b"{\"value\":\"\xff\"}");
    assert_content_conflict(&invalid_utf8, "document is not valid UTF-8");

    let invalid_syntax = TempDocument::write("syntax.json", br#"{"value":}"#);
    assert_content_conflict(&invalid_syntax, "document is not valid JSON");

    let trailing = TempDocument::write("trailing.json", b"{} trailing");
    assert_content_conflict(&trailing, "document has trailing non-whitespace input");

    let duplicate = TempDocument::write("duplicate.json", br#"{"a":1,"\u0061":2}"#);
    assert_content_conflict(
        &duplicate,
        "document has duplicate decoded member name \"a\"",
    );

    let depth = TempDocument::write(
        "depth.json",
        format!("{}[]{}", "[".repeat(128), "]".repeat(128)).as_bytes(),
    );
    assert_content_conflict(
        &depth,
        "document maximum depth 128 exceeds supported maximum 127",
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
fn outline_maps_reload_failures_to_stable_document_diagnostics() {
    let missing = TempDocument::write("missing.json", b"{}");
    assert!(JsonAdapter.probe(missing.path_str()).supported);
    fs::remove_file(&missing.path).expect("remove selected document");
    assert_outline_error(
        &missing,
        ProtocolDiagnosticCode::DocumentNotFound,
        "path",
        json!(missing.path_str()),
    );

    let invalid_utf8 = TempDocument::write("encoding.json", b"{}");
    assert!(JsonAdapter.probe(invalid_utf8.path_str()).supported);
    fs::write(&invalid_utf8.path, b"{\"value\":\"\xff\"}")
        .expect("replace selected document with non-UTF-8 bytes");
    assert_outline_error(
        &invalid_utf8,
        ProtocolDiagnosticCode::DocumentEncodingUnsupported,
        "encoding",
        json!("non-utf-8"),
    );

    let changed_inputs = [
        br#"{"value":}"#.to_vec(),
        b"{} trailing".to_vec(),
        br#"{"a":1,"\u0061":2}"#.to_vec(),
        format!("{}[]{}", "[".repeat(128), "]".repeat(128)).into_bytes(),
    ];
    for bytes in changed_inputs {
        let changed = TempDocument::write("changed.json", b"{}");
        assert!(JsonAdapter.probe(changed.path_str()).supported);
        fs::write(&changed.path, bytes).expect("replace selected document with invalid JSON");
        assert_outline_error(
            &changed,
            ProtocolDiagnosticCode::InternalError,
            "error_id",
            json!("json-document-changed-after-probe"),
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
    ] {
        let error = JsonAdapter
            .read(&read_input(&document, ref_id, 1, 100))
            .expect_err("invalid ref should fail")
            .protocol_error();

        assert_eq!(error.code(), ProtocolDiagnosticCode::RefInvalid);
        assert_eq!(error.details().get("ref"), Some(&json!(ref_id)));
        assert_eq!(error.details().get("reason"), Some(&json!(expected_reason)));
    }

    let ref_id = "json:#/items/9";
    let error = JsonAdapter
        .read(&read_input(&document, ref_id, 1, 100))
        .expect_err("missing canonical ref should fail")
        .protocol_error();

    assert_eq!(error.code(), ProtocolDiagnosticCode::RefNotFound);
    assert_eq!(error.details().get("ref"), Some(&json!(ref_id)));
    assert_eq!(error.details().get("reason"), None);
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
fn read_uses_reload_diagnostics_when_the_selected_document_changes() {
    let missing = TempDocument::write("missing-read.json", b"{}");
    assert!(JsonAdapter.probe(missing.path_str()).supported);
    fs::remove_file(&missing.path).expect("remove selected document");
    let missing_error = JsonAdapter
        .read(&read_input(&missing, "json:#", 1, 100))
        .expect_err("missing selected document should fail")
        .protocol_error();
    assert_eq!(
        missing_error.code(),
        ProtocolDiagnosticCode::DocumentNotFound
    );
    assert_eq!(
        missing_error.details().get("path"),
        Some(&json!(missing.path_str()))
    );

    let changed = TempDocument::write("changed-read.json", b"{}");
    assert!(JsonAdapter.probe(changed.path_str()).supported);
    fs::write(&changed.path, br#"{"value":}"#)
        .expect("replace selected document with invalid JSON");
    let changed_error = JsonAdapter
        .read(&read_input(&changed, "json:#", 1, 100))
        .expect_err("invalid reloaded JSON should fail")
        .protocol_error();
    assert_eq!(changed_error.code(), ProtocolDiagnosticCode::InternalError);
    assert_eq!(
        changed_error.details().get("error_id"),
        Some(&json!("json-document-changed-after-probe"))
    );
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
fn find_uses_reload_diagnostics_when_the_selected_document_changes() {
    let missing = TempDocument::write("missing-find.json", b"{}");
    assert!(JsonAdapter.probe(missing.path_str()).supported);
    fs::remove_file(&missing.path).expect("remove selected document");
    let missing_error = JsonAdapter
        .find(&find_input(&missing, "needle", 1, 100))
        .expect_err("missing selected document should fail")
        .protocol_error();
    assert_eq!(
        missing_error.code(),
        ProtocolDiagnosticCode::DocumentNotFound
    );
    assert_eq!(
        missing_error.details().get("path"),
        Some(&json!(missing.path_str()))
    );

    let changed = TempDocument::write("changed-find.json", b"{}");
    assert!(JsonAdapter.probe(changed.path_str()).supported);
    fs::write(&changed.path, br#"{"value":}"#)
        .expect("replace selected document with invalid JSON");
    let changed_error = JsonAdapter
        .find(&find_input(&changed, "needle", 1, 100))
        .expect_err("invalid reloaded JSON should fail")
        .protocol_error();
    assert_eq!(changed_error.code(), ProtocolDiagnosticCode::InternalError);
    assert_eq!(
        changed_error.details().get("error_id"),
        Some(&json!("json-document-changed-after-probe"))
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
    let expected_source = " \r\n{\"text\":\"\\u96ea\",\"items\":[]}\n\t";
    let mut bytes = b"\xef\xbb\xbf".to_vec();
    bytes.extend_from_slice(expected_source.as_bytes());
    let document = TempDocument::write("full-read.json", &bytes);
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
    assert_eq!(result.content_type, "application/json");
    let full_cost = result
        .facts
        .cost
        .expect("full read should return cost facts");
    assert_selection_cost(&full_cost, expected_source);

    let expected_measured = JsonAdapter
        .measure_unstructured_full_read_cost(&request, &["tokens".to_owned(), "bytes".to_owned()])
        .expect("full-read cost measurement should succeed");
    let measured = definition
        .measure_unstructured_full_read_cost(&request, &["tokens".to_owned(), "bytes".to_owned()])
        .expect("definition full-read cost measurement should succeed");
    assert_eq!(measured, expected_measured);
    let mut expected_measurements = full_cost.measurements;
    expected_measurements
        .retain(|measurement| matches!(measurement.unit.as_str(), "bytes" | "tokens"));
    assert_eq!(measured.measurements, expected_measurements);
}

#[test]
fn info_and_full_read_hooks_reuse_reload_diagnostics() {
    let missing = TempDocument::write("missing-info.json", b"{}");
    assert!(JsonAdapter.probe(missing.path_str()).supported);
    fs::remove_file(&missing.path).expect("remove selected document");
    let missing_error = JsonAdapter
        .info(&info_input(&missing))
        .expect_err("missing selected document should fail")
        .protocol_error();
    assert_eq!(
        missing_error.code(),
        ProtocolDiagnosticCode::DocumentNotFound
    );
    assert_eq!(
        missing_error.details().get("path"),
        Some(&json!(missing.path_str()))
    );

    let invalid_utf8 = TempDocument::write("encoding-full-read.json", b"{}");
    assert!(JsonAdapter.probe(invalid_utf8.path_str()).supported);
    fs::write(&invalid_utf8.path, b"{\"value\":\"\xff\"}")
        .expect("replace selected document with non-UTF-8 bytes");
    let encoding_error = JsonAdapter
        .unstructured_full_read(&full_read_request(&invalid_utf8))
        .expect_err("invalid UTF-8 should fail full read")
        .protocol_error();
    assert_eq!(
        encoding_error.code(),
        ProtocolDiagnosticCode::DocumentEncodingUnsupported
    );
    assert_eq!(
        encoding_error.details().get("encoding"),
        Some(&json!("non-utf-8"))
    );

    let changed = TempDocument::write("changed-full-read.json", b"{}");
    assert!(JsonAdapter.probe(changed.path_str()).supported);
    fs::write(&changed.path, br#"{"value":}"#)
        .expect("replace selected document with invalid JSON");
    let changed_error = JsonAdapter
        .measure_unstructured_full_read_cost(&full_read_request(&changed), &["bytes".to_owned()])
        .expect_err("invalid reloaded JSON should fail cost measurement")
        .protocol_error();
    assert_eq!(changed_error.code(), ProtocolDiagnosticCode::InternalError);
    assert_eq!(
        changed_error.details().get("error_id"),
        Some(&json!("json-document-changed-after-probe"))
    );
}

fn assert_content_conflict(document: &TempDocument, expected_detail: &str) {
    let result = JsonAdapter.probe(document.path_str());

    result.validate_semantics().expect("probe semantics");
    assert!(!result.supported);
    assert_eq!(result.format, None);
    assert_eq!(result.confidence, 0.0);
    assert_eq!(
        result
            .reasons
            .iter()
            .map(|reason| reason.code)
            .collect::<Vec<_>>(),
        [
            ProbeReasonCode::ExtensionMatch,
            ProbeReasonCode::ContentConflict
        ]
    );
    assert_eq!(result.reasons[1].detail, expected_detail);
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

fn assert_outline_error(
    document: &TempDocument,
    expected_code: ProtocolDiagnosticCode,
    detail_key: &str,
    detail_value: serde_json::Value,
) {
    let error = JsonAdapter
        .outline(&outline_input(document, 1, 100, None))
        .expect_err("outline reload should fail");
    let error = error.protocol_error();

    assert_eq!(error.code(), expected_code);
    assert_eq!(error.details().get(detail_key), Some(&detail_value));
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
            std::env::temp_dir().join(format!("docnav-json-probe-{}-{id}", std::process::id()));
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
