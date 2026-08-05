use super::*;

#[test]
fn outline_is_flat_default_h1_to_h3_and_ignores_code_fences() {
    let path = write_doc(
        "nested.md",
        "# Guide\nIntro\n\n```md\n## Fake\n```\n\n## Install\nBody\n\n#### Hidden\nDeep\n",
    );
    let input = outline_input(&path, 6000, 1, None);
    let result = outline_result(&input);

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
    for content in [
        "plain body\nwith no heading\n",
        "---\ntitle: Only frontmatter\n---\n",
        "Lead text.\n\n#### Deep\nBody\n",
    ] {
        let path = write_doc("fallback.md", content);
        let input = outline_input(&path, 6000, 1, Some(3));
        let definition = markdown_adapter_definition();
        let mut document = definition.create_document(input.document_path.clone());
        let outline = document
            .outline(&input)
            .expect("outline result")
            .into_structured()
            .expect("structured outline result");
        assert_eq!(outline.entries[0].ref_id, "doc:full");

        let read_input = read_input(&path, &outline.entries[0].ref_id, 6000, 1);
        let (same, fresh) = assert_ref_round_trip(&definition, document.as_mut(), &read_input);
        for read in [same, fresh] {
            assert_eq!(read.content, content);
            assert_eq!(read.content_type, "text/markdown");
        }
    }
}

#[test]
fn outline_refs_across_terminal_pages_round_trip_on_same_and_fresh_documents() {
    let mut source =
        "Lead text.\n\n# First heading with an intentionally long label\nfirst body\n".to_owned();
    for _ in 0..105 {
        source.push_str("filler\n");
    }
    source.push_str("# Late heading with another intentionally long label\nlate body\n");
    let path = write_doc("outline-ref-conformance.md", &source);
    let definition = markdown_adapter_definition();
    let mut page = positive(1);
    let mut refs = Vec::new();

    loop {
        let input = outline_input(&path, 20, page.get(), Some(3));
        let mut document = definition.create_document(input.document_path.clone());
        let result = document
            .outline(&input)
            .expect("outline result")
            .into_structured()
            .expect("structured outline result");
        assert_eq!(result.entries.len(), 1, "tiny budget must still advance");

        for entry in result.entries {
            assert!(entry.label.ends_with("..."));
            assert!(entry.cost.is_none());
            let read_input = read_input(&path, &entry.ref_id, 6000, 1);
            let (same, fresh) = assert_ref_round_trip(&definition, document.as_mut(), &read_input);
            assert_eq!(same, fresh);

            match entry.ref_id.as_str() {
                "HEAD:leading" => assert_eq!(same.content, "Lead text.\n\n"),
                "H:L3:H1" => {
                    assert!(same
                        .content
                        .starts_with("# First heading with an intentionally long label\n"));
                    assert!(!same.content.contains("# Late heading"));
                }
                "H:L110:H1" => assert_eq!(
                    same.content,
                    "# Late heading with another intentionally long label\nlate body\n"
                ),
                other => panic!("unexpected outline ref: {other}"),
            }
            refs.push(entry.ref_id);
        }

        let Some(next_page) = result.page else {
            break;
        };
        page = next_page;
    }

    assert_eq!(refs, vec!["HEAD:leading", "H:L3:H1", "H:L110:H1"]);
}

#[test]
fn outline_exposes_document_head_before_visible_headings_when_nonblank() {
    let path = write_doc(
        "document-head.md",
        "---\ntitle: Sample\n---\n\nLead text.\n\n# Real\nBody\n",
    );
    let input = outline_input(&path, 6000, 1, Some(3));
    let outline = outline_result(&input);

    assert_eq!(
        entry_refs(&outline.entries),
        vec!["HEAD:leading", "H:L7:H1"]
    );
    assert_eq!(outline.entries[0].label, "document head");
    assert_eq!(outline.entries[0].kind.as_deref(), Some("document_head"));
    assert_eq!(
        outline.entries[0].metadata.as_ref().unwrap()["document_region"],
        serde_json::json!("leading")
    );
}

#[test]
fn outline_exposes_document_head_when_leading_region_is_frontmatter_only() {
    let path = write_doc(
        "document-head-frontmatter.md",
        "---\ntitle: Sample\n---\n\n# Real\nBody\n",
    );
    let input = outline_input(&path, 6000, 1, Some(3));
    let outline = outline_result(&input);

    assert_eq!(
        entry_refs(&outline.entries),
        vec!["HEAD:leading", "H:L5:H1"]
    );
}

#[test]
fn outline_omits_document_head_for_empty_or_whitespace_only_prefix() {
    for content in ["# Real\nBody\n", "\n \t\n# Real\nBody\n"] {
        let path = write_doc("empty-head.md", content);
        let input = outline_input(&path, 6000, 1, Some(3));
        let outline = outline_result(&input);

        assert!(!entry_refs(&outline.entries).contains(&"HEAD:leading"));
        assert_eq!(outline.entries.len(), 1);
        assert_eq!(outline.entries[0].kind.as_deref(), Some("heading"));
    }
}

#[test]
fn outline_keeps_frontmatter_pseudo_heading_fence_pseudo_heading_and_hr_in_document_head() {
    let path = write_doc(
        "document-head-boundaries.md",
        "---\ntitle: Sample\n# not a heading\n---\n\n---\nLead.\n\n```md\n# not a heading\n```\n\n# Real\nBody\n",
    );
    let input = outline_input(&path, 6000, 1, Some(3));
    let outline = outline_result(&input);

    assert_eq!(
        entry_refs(&outline.entries),
        vec!["HEAD:leading", "H:L13:H1"]
    );
}

#[test]
fn duplicate_heading_paths_generate_unique_refs_and_read_unique_sections() {
    let path = write_doc("duplicates.md", "# A\n## B\nfirst\n# A\n## B\nsecond\n");
    let input = outline_input(&path, 6000, 1, Some(3));
    let outline = outline_result(&input);

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

#[test]
fn read_reports_ref_invalid_for_grammar_outside_refs() {
    let path = write_doc(
        "invalid-ref-formats.md",
        "# A\n## B\nfirst\n# A\n## B\nsecond\n",
    );

    let ref_id = "not-a-ref";
    let error = read_ref_error(&path, ref_id);
    assert_ref_invalid(&error, ref_id);
}

#[test]
fn read_reports_ref_not_found_for_canonical_no_match() {
    let path = write_doc("nofound.md", "# Guide\nBody\n");

    // Canonical grammar 但无匹配 → REF_NOT_FOUND
    let ref_id = "H:L99:H1";
    let error = read_ref_error(&path, ref_id);
    assert_ref_not_found(&error, ref_id);
}

#[test]
fn structure_snapshot_ref_is_evaluated_against_current_document() {
    let path1 = write_doc("snap1.md", "# A\nBody\n## B\nMore\n");
    let input = outline_input(&path1, 6000, 1, Some(3));
    let outline1 = outline_result(&input);
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

#[test]
fn prepared_document_keeps_successful_view_after_path_mutation_and_deletion() {
    let path = write_doc("prepared-success.md", "# Stable\nold body\n");
    let definition = markdown_adapter_definition();
    let outline_input = outline_input(&path, 6000, 1, Some(3));
    let mut document = definition.create_document(outline_input.document_path.clone());
    let outline = document
        .outline(&outline_input)
        .expect("outline result")
        .into_structured()
        .expect("structured outline result");
    let ref_id = outline.entries[0].ref_id.clone();
    let read_input = read_input(&path, &ref_id, 6000, 1);
    let replace_path = |name: &str, bytes: &[u8]| {
        let replacement = path.with_file_name(name);
        fs::write(&replacement, bytes).expect("write replacement document");
        fs::remove_file(&path).expect("remove replaced document");
        fs::rename(&replacement, &path).expect("install replacement document");
    };

    replace_path("prepared-success-plain.md", b"plain replacement\n");
    let same_after_replacement = document.read(&read_input).expect("read captured view");
    assert_eq!(same_after_replacement.content, "# Stable\nold body\n");

    let mut replacement_document = definition.create_document(path_string(&path));
    let replacement_error = replacement_document
        .read(&read_input)
        .expect_err("fresh replacement view must not contain the old heading")
        .protocol_error();
    assert_ref_not_found(&replacement_error, &ref_id);

    fs::write(&path, "# Mutated\nnew body\n").expect("mutate replacement in place");
    let same_after_mutation = document.read(&read_input).expect("read captured view");
    assert_eq!(same_after_mutation.content, "# Stable\nold body\n");
    let mut mutated_document = definition.create_document(path_string(&path));
    let mutated = mutated_document
        .read(&read_input)
        .expect("fresh view should resolve the current coordinate");
    assert_eq!(mutated.content, "# Mutated\nnew body\n");

    replace_path("prepared-success-invalid.md", &[0xFF, 0xFE, 0x00]);
    let same_after_encoding_change = document.read(&read_input).expect("read captured view");
    assert_eq!(same_after_encoding_change.content, "# Stable\nold body\n");
    let mut encoding_document = definition.create_document(path_string(&path));
    let encoding_error = encoding_document
        .read(&read_input)
        .expect_err("fresh invalid-encoding view must fail")
        .protocol_error();
    assert_eq!(
        encoding_error.code(),
        ProtocolDiagnosticCode::DocumentEncodingUnsupported
    );

    fs::remove_file(&path).expect("delete document");
    let same_after_deletion = document.read(&read_input).expect("read captured view");
    assert_eq!(same_after_deletion.content, "# Stable\nold body\n");

    let mut deleted_document = definition.create_document(path_string(&path));
    let deleted_error = deleted_document
        .read(&read_input)
        .expect_err("fresh deleted view must fail")
        .protocol_error();
    assert_eq!(
        deleted_error.code(),
        ProtocolDiagnosticCode::DocumentNotFound
    );
}

#[test]
fn prepared_document_caches_initial_encoding_failure_after_path_repair() {
    let path = write_bytes("prepared-failure.md", &[0xFF, 0xFE, 0x00]);
    let definition = markdown_adapter_definition();
    let input = read_input(&path, "doc:full", 6000, 1);
    let mut document = definition.create_document(input.document_path.clone());

    let first = document
        .read(&input)
        .expect_err("initial encoding failure")
        .protocol_error();
    assert_eq!(
        first.code(),
        ProtocolDiagnosticCode::DocumentEncodingUnsupported
    );

    fs::write(&path, "# Repaired\nnew body\n").expect("repair document");
    let repeated = document
        .read(&input)
        .expect_err("same document must retain the initial failure")
        .protocol_error();
    assert_eq!(
        repeated.code(),
        ProtocolDiagnosticCode::DocumentEncodingUnsupported
    );

    let mut fresh_document = definition.create_document(input.document_path.clone());
    let fresh = fresh_document.read(&input).expect("fresh repaired view");
    assert_eq!(fresh.content, "# Repaired\nnew body\n");
}
