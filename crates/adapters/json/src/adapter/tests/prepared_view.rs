use super::*;

#[test]
fn empty_find_rejection_does_not_prepare_the_document() {
    let document = TempDocument::missing("empty-query-lazy.json");
    let mut selected = create_adapter_document(&document);

    let error = selected
        .find(&find_input(&document, "", 1, 100))
        .expect_err("empty query should fail before document access")
        .protocol_error();
    assert_eq!(error.code(), ProtocolDiagnosticCode::InvalidRequest);

    document.write_in_place(br#"{"value":"hit"}"#);
    let matches = selected
        .find(&find_input(&document, "hit", 1, 100))
        .expect("empty-query rejection must not prepare the missing document")
        .matches;
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].ref_id, "json:#/value");
}

#[test]
fn prepared_document_keeps_first_success_across_source_changes() {
    let initial_source = br#"{"value": /* old-direct */ "old"}"#;
    let document = TempDocument::missing("stable-view.jsonc");
    let definition = crate::json_adapter_definition();
    let mut selected = definition.create_document(document.path_str().to_owned());

    document.write_in_place(initial_source);
    let outline = selected
        .outline(&outline_input(&document, 1, 100, None))
        .expect("first document access should observe the source written after factory creation")
        .into_structured()
        .expect("JSON outline should be structured");
    assert_eq!(outline.entries[0].ref_id, "json:comments:#/value");
    assert_eq!(
        selected
            .read(&read_input(&document, "json:#/value", 1, 100))
            .expect("prepared base read")
            .content,
        r#""old""#,
    );

    document.write_in_place(br#"{"value":"new"}"#);
    assert_eq!(
        selected
            .read(&read_input(&document, "json:#/value", 1, 100))
            .expect("same prepared document must not refresh after replacement")
            .content,
        r#""old""#,
    );
    assert_eq!(
        selected
            .read(&read_input(&document, "json:comments:#/value", 1, 100,))
            .expect("same prepared document must retain comment attribution")
            .content,
        "/* old-direct */\n\"old\"",
    );
    assert_eq!(
        execute_read(read_input(&document, "json:#/value", 1, 100)).content,
        r#""new""#,
    );
    let stale_comment = fresh_read_error(
        &definition,
        &read_input(&document, "json:comments:#/value", 1, 100),
    );
    assert_protocol_error(
        &stale_comment,
        "REF_NOT_FOUND",
        json!({ "ref": "json:comments:#/value" }),
    );

    document.replace_path(br#"{"value":"path-replacement"}"#);
    assert_eq!(
        selected
            .read(&read_input(&document, "json:#/value", 1, 100))
            .expect("same prepared document must ignore a renamed path replacement")
            .content,
        r#""old""#,
    );
    assert_eq!(
        execute_read(read_input(&document, "json:#/value", 1, 100)).content,
        r#""path-replacement""#,
    );

    let old_info = selected
        .info(&info_input(&document))
        .expect("info should reuse the prepared view");
    assert_eq!(
        old_info
            .document
            .as_ref()
            .and_then(|facts| facts.size.as_ref())
            .map(|size| size.value),
        Some(initial_source.len() as u64),
    );
    let old_full_read = selected
        .unstructured_full_read(&full_read_request(&document))
        .expect("full read should reuse the prepared view");
    assert_eq!(
        old_full_read.content,
        String::from_utf8_lossy(initial_source)
    );
    let measured = selected
        .measure_unstructured_full_read_cost(&full_read_request(&document), &["bytes".to_owned()])
        .expect("cost should reuse the prepared view");
    assert_eq!(measured.measurements[0].value, initial_source.len() as u64);

    document.remove();
    assert_eq!(
        selected
            .read(&read_input(&document, "json:#/value", 1, 100))
            .expect("same prepared document must survive path deletion")
            .content,
        r#""old""#,
    );
    let missing = fresh_read_error(&definition, &read_input(&document, "json:#/value", 1, 100));
    assert_protocol_error(
        &missing,
        "DOCUMENT_NOT_FOUND",
        json!({ "path": document.path_str() }),
    );

    document.write_in_place(br#"{"value":"repaired"}"#);
    assert_eq!(
        execute_read(read_input(&document, "json:#/value", 1, 100)).content,
        r#""repaired""#,
    );

    document.write_in_place(br#"{"value": }"#);
    assert_eq!(
        selected
            .read(&read_input(&document, "json:#/value", 1, 100))
            .expect("same prepared document must ignore invalid replacement")
            .content,
        r#""old""#,
    );
    let invalid = fresh_read_error(&definition, &read_input(&document, "json:#/value", 1, 100));
    assert_protocol_error(
        &invalid,
        "DOCUMENT_CONTENT_INVALID",
        json!({
            "path": document.path_str(),
            "reason": "JSON_SYNTAX_INVALID",
        }),
    );

    document.replace_path(&[0xff]);
    assert_eq!(
        selected
            .read(&read_input(&document, "json:#/value", 1, 100))
            .expect("same prepared document must ignore an encoding-changing replacement")
            .content,
        r#""old""#,
    );
    let encoding = fresh_read_error(&definition, &read_input(&document, "json:#/value", 1, 100));
    assert_protocol_error(
        &encoding,
        "DOCUMENT_ENCODING_UNSUPPORTED",
        json!({
            "path": document.path_str(),
            "encoding": "non-utf-8",
        }),
    );
}

#[test]
fn prepared_document_caches_first_failure_and_fresh_document_observes_repair() {
    let document = TempDocument::missing("cached-failure.json");
    let definition = crate::json_adapter_definition();
    let mut selected = definition.create_document(document.path_str().to_owned());
    let input = outline_input(&document, 1, 100, None);

    let first = selected
        .outline(&input)
        .expect_err("missing document should fail first preparation")
        .protocol_error();
    assert_protocol_error(
        &first,
        "DOCUMENT_NOT_FOUND",
        json!({ "path": document.path_str() }),
    );

    document.write_in_place(br#"{"value":true}"#);
    let cached = selected
        .outline(&input)
        .expect_err("same document must retain its first preparation failure")
        .protocol_error();
    assert_eq!(cached, first);

    let fresh = execute_outline(input)
        .into_structured()
        .expect("fresh document should observe the repaired source");
    assert_eq!(fresh.entries, [entry("json:#/value", "value", "boolean")]);
}

fn fresh_read_error(
    definition: &AdapterDefinition<'_>,
    input: &ReadInput,
) -> docnav_protocol::ProtocolError {
    definition
        .create_document(input.document_path.clone())
        .read(input)
        .expect_err("fresh read should fail")
        .protocol_error()
}

impl TempDocument {
    fn write_in_place(&self, bytes: &[u8]) {
        fs::create_dir_all(&self.directory).expect("create temporary document directory");
        fs::write(&self.path, bytes).expect("write temporary document in place");
    }

    fn replace_path(&self, bytes: &[u8]) {
        fs::create_dir_all(&self.directory).expect("create temporary document directory");
        let replacement = self.directory.join("replacement.tmp");
        fs::write(&replacement, bytes).expect("write sibling replacement document");
        fs::rename(&replacement, &self.path).expect("rename sibling over document path");
    }

    fn remove(&self) {
        match fs::remove_file(&self.path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => panic!("remove temporary document: {error}"),
        }
    }
}
