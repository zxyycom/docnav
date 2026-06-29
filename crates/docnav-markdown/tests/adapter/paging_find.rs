use super::*;

// @case WB-MD-PAGE-001
#[test]
fn read_paginates_unicode_without_splitting_characters() {
    let path = write_doc("unicode.md", "# A\n界界界abc\n");
    let ref_id = "H:L1:H1";
    let arguments = ReadArguments {
        ref_id: ref_id.to_owned(),
        limit: positive(5),
        page: positive(1),
        options: None,
    };
    let request = make_request(
        &path,
        Operation::Read,
        OperationArguments::Read(arguments.clone()),
    );

    let first = MarkdownAdapter
        .read(&request, &arguments)
        .expect("first page");
    assert_eq!(first.ref_id, ref_id);
    assert_eq!(first.content, "# A\n界");
    assert_eq!(first.page, Some(positive(2)));

    let second_arguments = ReadArguments {
        page: positive(2),
        ..arguments
    };
    let second_request = make_request(
        &path,
        Operation::Read,
        OperationArguments::Read(second_arguments.clone()),
    );
    let second = MarkdownAdapter
        .read(&second_request, &second_arguments)
        .expect("second page");
    assert!(second.content.starts_with("界界"));
}

// @case WB-MD-FIND-001
#[test]
fn find_ref_targets_current_visible_region_and_read_contains_match() {
    let path = write_doc(
        "find-current-region.md",
        "# Current\nintro\n\n#### Hidden\ntarget\n\n# Next\nother\n",
    );
    let arguments = find_args("target", 6000, 1, Some(3));

    let result = find_result(&path, &arguments);

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
fn find_match_before_first_visible_heading_falls_back_to_full_document() {
    let path = write_doc("find-before-heading.md", "target before\n\n# Later\nbody\n");
    let arguments = find_args("target before", 6000, 1, Some(3));

    let result = find_result(&path, &arguments);

    assert_eq!(result.matches.len(), 1);
    assert_eq!(result.matches[0].ref_id, "doc:full");

    let read = read_ref(&path, &result.matches[0].ref_id);
    assert!(read.content.contains("target before"));
}

#[test]
fn find_falls_back_to_full_document_when_no_heading_is_visible() {
    let path = write_doc("fallback-find.md", "#### Deep\ntarget\n");
    let arguments = find_args("target", 6000, 1, Some(3));

    let result = find_result(&path, &arguments);

    assert_eq!(result.matches.len(), 1);
    assert_eq!(result.matches[0].ref_id, "doc:full");
}

// @case WB-MD-PAGE-002
#[test]
fn outline_paginates_with_response_page_until_end_and_past_end() {
    let path = write_doc("outline-pages.md", "# A\none\n# B\ntwo\n# C\nthree\n");
    let first_arguments = outline_args(10, 1, Some(3));

    let first = outline_result(&path, &first_arguments);
    assert_eq!(entry_refs(&first.entries), vec!["H:L1:H1"]);
    let second_page = first.page.expect("second page");

    let second_arguments = OutlineArguments {
        page: second_page,
        ..first_arguments.clone()
    };
    let second = outline_result(&path, &second_arguments);
    assert_eq!(entry_refs(&second.entries), vec!["H:L3:H1"]);
    let third_page = second.page.expect("third page");

    let third_arguments = OutlineArguments {
        page: third_page,
        ..first_arguments.clone()
    };
    let third = outline_result(&path, &third_arguments);
    assert_eq!(entry_refs(&third.entries), vec!["H:L5:H1"]);
    assert_eq!(third.page, None);

    let past_end_arguments = OutlineArguments {
        page: positive(4),
        ..first_arguments
    };
    let past_end = outline_result(&path, &past_end_arguments);
    assert!(past_end.entries.is_empty());
    assert_eq!(past_end.page, None);
}

#[test]
fn find_paginates_with_response_page_until_end_and_past_end() {
    let path = write_doc(
        "find-pages.md",
        "# A\ntarget 1\n# B\ntarget 2\n# C\ntarget 3\n",
    );
    let first_arguments = find_args("target", 10, 1, Some(3));

    let first = find_result(&path, &first_arguments);
    assert_eq!(entry_refs(&first.matches), vec!["H:L1:H1"]);
    let second_page = first.page.expect("second page");

    let second_arguments = FindArguments {
        page: second_page,
        ..first_arguments.clone()
    };
    let second = find_result(&path, &second_arguments);
    assert_eq!(entry_refs(&second.matches), vec!["H:L3:H1"]);
    let third_page = second.page.expect("third page");

    let third_arguments = FindArguments {
        page: third_page,
        ..first_arguments.clone()
    };
    let third = find_result(&path, &third_arguments);
    assert_eq!(entry_refs(&third.matches), vec!["H:L5:H1"]);
    assert_eq!(third.page, None);

    let past_end_arguments = FindArguments {
        page: positive(4),
        ..first_arguments
    };
    let past_end = find_result(&path, &past_end_arguments);
    assert!(past_end.matches.is_empty());
    assert_eq!(past_end.page, None);
}
