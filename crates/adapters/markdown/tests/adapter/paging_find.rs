use super::*;

#[test]
fn read_paginates_unicode_without_splitting_characters() {
    let selected = "# A\n界界界abc\n";
    let path = write_doc("unicode.md", selected);
    let ref_id = "H:L1:H1";
    let input = ReadInput {
        document_path: path_string(&path),
        ref_id: ref_id.to_owned(),
        limit: positive(5),
        page: positive(1),
    };

    let first = adapter_document(&input.document_path)
        .read(&input)
        .expect("first page");
    assert_eq!(first.ref_id, ref_id);
    assert_eq!(first.content, "# A\n界");
    assert_cost_measurements(&first.cost, "selection", selected);
    assert_eq!(first.page, Some(positive(2)));

    let second_input = ReadInput {
        page: positive(2),
        ..input
    };
    let second = adapter_document(&second_input.document_path)
        .read(&second_input)
        .expect("second page");
    assert!(second.content.starts_with("界界"));
}

#[test]
fn find_ref_targets_current_visible_region_and_read_contains_match() {
    let path = write_doc(
        "find-current-region.md",
        "# Current\nintro\n\n#### Hidden\ntarget\n\n# Next\nother\n",
    );
    let input = find_input(&path, "target", 6000, 1, Some(3));
    let result = find_result(&input);

    assert_eq!(result.matches.len(), 1);
    // target 在 H4 "Hidden" 下，但 Hidden 不 visible (max=3)，
    // 最近 visible heading 是 "Current" (line 1, H1)
    assert_eq!(result.matches[0].ref_id, "H:L1:H1");
    assert_canonical_ref(&result.matches[0].ref_id);
    assert!(result.matches[0].label.contains("target"));

    let read = read_ref(&path, &result.matches[0].ref_id);
    assert!(read.content.contains("target"));
    assert!(!read.content.contains("# Next"));
}

#[test]
fn find_match_before_first_visible_heading_uses_document_head_ref() {
    let path = write_doc("find-before-heading.md", "target before\n\n# Later\nbody\n");
    let input = find_input(&path, "target before", 6000, 1, Some(3));
    let result = find_result(&input);

    assert_eq!(result.matches.len(), 1);
    assert_eq!(result.matches[0].ref_id, "HEAD:leading");

    let read = read_ref(&path, &result.matches[0].ref_id);
    assert_eq!(read.content, "target before\n\n");
    assert_eq!(read.content_type, "text/markdown");
    assert!(read.content.contains("target before"));
}

#[test]
fn find_falls_back_to_full_document_when_no_heading_is_visible() {
    let path = write_doc("fallback-find.md", "target before\n\n#### Deep\nbody\n");
    let input = find_input(&path, "target", 6000, 1, Some(3));
    let definition = markdown_adapter_definition();
    let mut document = definition.create_document(input.document_path.clone());
    let result = document.find(&input).expect("find result");

    assert_eq!(result.matches.len(), 1);
    assert_eq!(result.matches[0].ref_id, "doc:full");
    let read_input = read_input(&path, &result.matches[0].ref_id, 6000, 1);
    let (same, fresh) = assert_ref_round_trip(&definition, document.as_mut(), &read_input);
    for read in [same, fresh] {
        assert!(read.content.contains("target before"));
    }
}

#[test]
fn find_refs_across_terminal_pages_round_trip_on_same_and_fresh_documents() {
    let path = write_doc(
        "find-ref-conformance.md",
        "target head\n\n# First\ntarget one\ntarget two\n\n#### Hidden\ntarget hidden\n\n# Next\ntarget next\n",
    );
    let definition = markdown_adapter_definition();
    let mut page = positive(1);
    let mut refs = Vec::new();

    loop {
        let input = find_input(&path, "target", 1, page.get(), Some(3));
        let mut document = definition.create_document(input.document_path.clone());
        let result = document.find(&input).expect("find result");
        assert_eq!(result.matches.len(), 1, "tiny budget must still advance");

        for matched in result.matches {
            assert_eq!(matched.label, ".");
            let read_input = read_input(&path, &matched.ref_id, 6000, 1);
            let (same, fresh) = assert_ref_round_trip(&definition, document.as_mut(), &read_input);
            assert_eq!(same, fresh);
            assert!(same.content.contains("target"));
            refs.push(matched.ref_id);
        }

        let Some(next_page) = result.page else {
            break;
        };
        page = next_page;
    }

    assert_eq!(refs.len(), 5);
    assert_eq!(refs[0], "HEAD:leading");
    assert_eq!(refs.iter().filter(|ref_id| *ref_id == "H:L3:H1").count(), 3);
    assert_eq!(refs.last().map(String::as_str), Some("H:L10:H1"));
}

#[test]
fn read_document_head_returns_original_markdown_and_paginates_unicode() {
    let path = write_doc(
        "read-document-head.md",
        "---\ntitle: Sample\n---\n\n界界界abc\n\n# Later\nbody\n",
    );

    let first = read_ref_with_page(&path, "HEAD:leading", 7, 1);

    assert_eq!(first.ref_id, "HEAD:leading");
    assert_eq!(first.content, "---\ntit");
    assert_eq!(first.content_type, "text/markdown");
    assert_cost_measurements(
        &first.cost,
        "selection",
        "---\ntitle: Sample\n---\n\n界界界abc\n\n",
    );
    assert_eq!(first.page, Some(positive(2)));

    let unicode_page = read_ref_with_page(&path, "HEAD:leading", 7, 4);
    assert!(unicode_page.content.contains("界界界"));
}

#[test]
fn read_document_head_preserves_yaml_delimiters_and_leading_text() {
    let path = write_doc(
        "read-document-head-frontmatter.md",
        "---\ntitle: Sample\n---\n\nLead text.\n\n# Later\nbody\n",
    );

    let read = read_ref(&path, "HEAD:leading");

    assert_eq!(read.content, "---\ntitle: Sample\n---\n\nLead text.\n\n");
    assert_eq!(read.content_type, "text/markdown");
}

#[test]
fn outline_paginates_with_response_page_until_end_and_past_end() {
    let path = write_doc("outline-pages.md", "# A\none\n# B\ntwo\n# C\nthree\n");
    let first_input = outline_input(&path, 10, 1, Some(3));
    let first = outline_result(&first_input);
    assert_eq!(entry_refs(&first.entries), vec!["H:L1:H1"]);
    let second_page = first.page.expect("second page");

    let second_input = OutlineInput {
        page: second_page,
        ..first_input.clone()
    };
    let second = outline_result(&second_input);
    assert_eq!(entry_refs(&second.entries), vec!["H:L3:H1"]);
    let third_page = second.page.expect("third page");

    let third_input = OutlineInput {
        page: third_page,
        ..first_input.clone()
    };
    let third = outline_result(&third_input);
    assert_eq!(entry_refs(&third.entries), vec!["H:L5:H1"]);
    assert_eq!(third.page, None);

    let past_end_input = OutlineInput {
        page: positive(4),
        ..first_input
    };
    let past_end = outline_result(&past_end_input);
    assert!(past_end.entries.is_empty());
    assert_eq!(past_end.page, None);
}

#[test]
fn find_paginates_with_response_page_until_end_and_past_end() {
    let path = write_doc(
        "find-pages.md",
        "# A\ntarget 1\n# B\ntarget 2\n# C\ntarget 3\n",
    );
    let first_input = find_input(&path, "target", 10, 1, Some(3));
    let first = find_result(&first_input);
    assert_eq!(entry_refs(&first.matches), vec!["H:L1:H1"]);
    let second_page = first.page.expect("second page");

    let second_input = FindInput {
        page: second_page,
        ..first_input.clone()
    };
    let second = find_result(&second_input);
    assert_eq!(entry_refs(&second.matches), vec!["H:L3:H1"]);
    let third_page = second.page.expect("third page");

    let third_input = FindInput {
        page: third_page,
        ..first_input.clone()
    };
    let third = find_result(&third_input);
    assert_eq!(entry_refs(&third.matches), vec!["H:L5:H1"]);
    assert_eq!(third.page, None);

    let past_end_input = FindInput {
        page: positive(4),
        ..first_input
    };
    let past_end = find_result(&past_end_input);
    assert!(past_end.matches.is_empty());
    assert_eq!(past_end.page, None);
}
