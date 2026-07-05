use super::*;

// @case WB-MD-ADAPTER-OUTLINE-001
#[test]
fn outline_is_flat_default_h1_to_h3_and_ignores_code_fences() {
    let path = write_doc(
        "nested.md",
        "# Guide\nIntro\n\n```md\n## Fake\n```\n\n## Install\nBody\n\n#### Hidden\nDeep\n",
    );
    let arguments = outline_args(6000, 1, None);
    let request = make_request(
        &path,
        Operation::Outline,
        OperationArguments::Outline(arguments.clone()),
    );

    let result = MarkdownAdapter
        .outline(&request, &arguments)
        .expect("outline result")
        .into_structured()
        .expect("structured outline result");

    assert_eq!(result.entries.len(), 2);
    // Guide: line 1, level 1
    // Install: line 8, level 2
    // 有效 outline entries 为 Guide 与 Install。
    assert_eq!(result.entries[0].ref_id, "H:L1:H1");
    assert_eq!(result.entries[1].ref_id, "H:L8:H2");
    for entry in &result.entries {
        assert_canonical_ref(&entry.ref_id);
    }
    assert!(!result
        .entries
        .iter()
        .any(|entry| entry.ref_id.contains("Fake")));
    assert!(!result
        .entries
        .iter()
        .any(|entry| entry.ref_id.contains("Hidden")));
}

#[test]
fn outline_falls_back_to_full_document_for_no_visible_heading() {
    for content in ["plain body\nwith no heading\n", "#### Deep\nBody\n"] {
        let path = write_doc("fallback.md", content);
        let arguments = outline_args(6000, 1, Some(3));
        let request = make_request(
            &path,
            Operation::Outline,
            OperationArguments::Outline(arguments.clone()),
        );
        let outline = MarkdownAdapter
            .outline(&request, &arguments)
            .expect("outline result")
            .into_structured()
            .expect("structured outline result");
        assert_eq!(outline.entries[0].ref_id, "doc:full");

        let read_arguments = ReadArguments {
            ref_id: outline.entries[0].ref_id.clone(),
            limit: positive(6000),
            page: positive(1),
            options: None,
        };
        let read_request = make_request(
            &path,
            Operation::Read,
            OperationArguments::Read(read_arguments.clone()),
        );
        let read = MarkdownAdapter
            .read(&read_request, &read_arguments)
            .expect("read full document");
        assert_eq!(read.content, content);
        assert_eq!(read.content_type, "text/markdown");
    }
}

// @case WB-MD-REF-001
#[test]
fn duplicate_heading_paths_generate_unique_refs_and_read_unique_sections() {
    let path = write_doc("duplicates.md", "# A\n## B\nfirst\n# A\n## B\nsecond\n");
    let arguments = outline_args(6000, 1, Some(3));
    let request = make_request(
        &path,
        Operation::Outline,
        OperationArguments::Outline(arguments.clone()),
    );

    let outline = MarkdownAdapter
        .outline(&request, &arguments)
        .expect("outline result")
        .into_structured()
        .expect("structured outline result");

    let all_refs: Vec<String> = outline
        .entries
        .iter()
        .map(|entry| entry.ref_id.clone())
        .collect();
    // # A (line 1, H1)
    // ## B (line 2, H2)
    // # A (line 4, H1)
    // ## B (line 5, H2)
    assert_eq!(all_refs, vec!["H:L1:H1", "H:L2:H2", "H:L4:H1", "H:L5:H2",]);
    for ref_id in &all_refs {
        assert_canonical_ref(ref_id);
    }

    // 读取第一个 B section（包含 "first"）
    let first = read_ref(&path, "H:L2:H2");
    assert!(first.content.contains("first"));
    assert!(!first.content.contains("second"));

    // 读取第二个 B section（包含 "second"）
    let second = read_ref(&path, "H:L5:H2");
    assert!(second.content.contains("second"));
    assert!(!second.content.contains("first"));

    // 读取第一个 A section
    let first_a = read_ref(&path, "H:L1:H1");
    assert!(first_a.content.contains("first"));
    assert!(!first_a.content.contains("second"));
}

// @case WB-MD-REF-002
#[test]
fn read_reports_ref_invalid_for_grammar_outside_refs() {
    let path = write_doc(
        "invalid-ref-formats.md",
        "# A\n## B\nfirst\n# A\n## B\nsecond\n",
    );

    let grammar_outside_refs = [
        "P:A > B",
        "H:L01:H1",
        "H:L1:H0",
        "H:L1",
        "H:L1:H1:extra",
        "not-a-ref",
        "",
    ];
    for ref_id in &grammar_outside_refs {
        if ref_id.is_empty() {
            // 空字符串可能触发共享层校验（非空字符串要求）
            continue;
        }
        let error = read_ref_error(&path, ref_id);
        assert_ref_invalid(&error, ref_id);
    }
}

#[test]
fn read_reports_ref_not_found_for_canonical_no_match() {
    let path = write_doc("nofound.md", "# Guide\nBody\n");

    // Canonical grammar 但无匹配 → REF_NOT_FOUND
    let error = read_ref_error(&path, "H:L99:H1");
    assert_ref_not_found(&error, "H:L99:H1");

    let error = read_ref_error(&path, "H:L1:H2");
    assert_ref_not_found(&error, "H:L1:H2");
}

#[test]
fn structure_snapshot_ref_is_evaluated_against_current_document() {
    let path1 = write_doc("snap1.md", "# A\nBody\n## B\nMore\n");
    let arguments = outline_args(6000, 1, Some(3));
    let outline1 = outline_result(&path1, &arguments);
    let ref_a = &outline1.entries[0].ref_id;

    // 原文档中可以正常读取
    let read1 = read_ref(&path1, ref_a);
    assert!(read1.content.contains("# A"));

    // 文档变化后重新解析，使用先前生成的 ref
    let path2 = write_doc("snap2.md", "No headings\nJust text\n");
    let error = read_ref_error(&path2, ref_a);
    // 结构坐标变化后的 canonical ref 返回 REF_NOT_FOUND。
    assert_ref_not_found(&error, ref_a);
}
