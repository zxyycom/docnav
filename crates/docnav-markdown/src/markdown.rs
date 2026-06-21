mod document;
mod format;
mod options;
mod parse;
mod refs;
mod text;

pub use document::{MarkdownDocument, ResolvedRef};
pub use format::{is_markdown_extension, is_utf8_markdown_candidate};
pub use options::max_heading_level_from_options;
pub use text::cost_for;

#[cfg(test)]
mod tests {
    use super::refs::FULL_DOCUMENT_REF;
    use super::*;

    // @case WB-MD-PARSE-001
    #[test]
    fn parser_ignores_code_fence_pseudo_heading_and_invalid_heading() {
        let document = MarkdownDocument::parse(
            "# Real\n\n```\n# Not real\n```\n\n#NoSpace\n\n## Child\n".to_owned(),
        );

        let titles: Vec<&str> = document
            .headings()
            .iter()
            .map(|heading| heading.title.as_str())
            .collect();
        assert_eq!(titles, vec!["Real", "Child"]);
    }

    #[test]
    fn frontmatter_is_excluded_from_outline_headings() {
        let document = MarkdownDocument::parse("---\ntitle: Sample\n---\n\n# Real\n".to_owned());

        assert_eq!(document.headings().len(), 1);
        assert_eq!(document.headings()[0].title, "Real");
    }

    #[test]
    fn read_section_ends_at_next_same_or_higher_heading() {
        let document = MarkdownDocument::parse("# A\nIntro\n## B\nNested\n# C\nEnd\n".to_owned());
        let heading = &document.headings()[0];

        assert_eq!(
            document.section_content(heading),
            "# A\nIntro\n## B\nNested\n"
        );
    }

    // @case WB-MD-OUTLINE-001
    #[test]
    fn outline_generates_canonical_heading_refs() {
        let document = MarkdownDocument::parse("# Guide\n\n## Install\n".to_owned());
        let entries = document.outline_entries(3);
        let refs: Vec<&str> = entries.iter().map(|entry| entry.ref_id.as_str()).collect();

        // Guide (line 1, level 1)
        // Install (line 3, level 2)
        assert_eq!(refs, vec!["H:L1:H1", "H:L3:H2"]);
    }

    #[test]
    fn duplicate_headings_receive_unique_canonical_refs() {
        let document = MarkdownDocument::parse("# A\n## B\n# A\n## B\n".to_owned());

        let entries = document.outline_entries(3);
        let refs: Vec<&str> = entries.iter().map(|entry| entry.ref_id.as_str()).collect();

        // All four headings have different structural coordinates.
        assert_eq!(refs, vec!["H:L1:H1", "H:L2:H2", "H:L3:H1", "H:L4:H2",]);
    }

    #[test]
    fn outline_ref_is_independent_of_path_and_title() {
        let document = MarkdownDocument::parse("# Same\n## Same\n# Same\n".to_owned());
        let entries = document.outline_entries(3);
        let refs: Vec<&str> = entries.iter().map(|entry| entry.ref_id.as_str()).collect();

        // 重复 title/path 的 entries 使用各自结构坐标生成 ref。
        assert_eq!(refs, vec!["H:L1:H1", "H:L2:H2", "H:L3:H1"]);
    }

    #[test]
    fn outline_refs_consistent_under_different_max_heading_level() {
        let document =
            MarkdownDocument::parse("# Top\n\n## A\n\n### Deep\n\n#### Hidden\n".to_owned());

        let entries_h2 = document.outline_entries(2);
        let entries_h3 = document.outline_entries(3);
        let entries_h4 = document.outline_entries(4);

        // 可见性过滤保持同一 heading 的 line/level ref 稳定。
        let top_ref = "H:L1:H1";
        assert_eq!(entries_h2[0].ref_id, top_ref);
        assert_eq!(entries_h3[0].ref_id, top_ref);
        assert_eq!(entries_h4[0].ref_id, top_ref);

        let a_ref = "H:L3:H2";
        // H2 可见，H3 可见，H4 可见时 A 都在
        assert_eq!(entries_h2[1].ref_id, a_ref);
        assert_eq!(entries_h3[1].ref_id, a_ref);
        assert_eq!(entries_h4[1].ref_id, a_ref);

        let deep_ref = "H:L5:H3";
        // level >= 3 时包含 H3。
        assert!(!entries_h2.iter().any(|e| e.ref_id == deep_ref));
        assert_eq!(entries_h3[2].ref_id, deep_ref);
        assert_eq!(entries_h4[2].ref_id, deep_ref);

        let hidden_ref = "H:L7:H4";
        // level >= 4 时包含 H4。
        assert_eq!(entries_h4[3].ref_id, hidden_ref);
        assert!(!entries_h3.iter().any(|e| e.ref_id == hidden_ref));
    }

    #[test]
    fn outline_display_includes_title_and_cost() {
        let document = MarkdownDocument::parse("# Guide\nContent here\n".to_owned());
        let entries = document.outline_entries(3);

        assert!(entries[0].display.contains("Guide"));
        assert!(entries[0].display.contains("H1"));
        // 仍包含 cost 信息
        assert!(entries[0].display.contains("line") || entries[0].display.contains("KB"));
    }

    #[test]
    fn outline_display_handles_whitespace_only_title() {
        let document = MarkdownDocument::parse("# \nContent\n".to_owned());
        let entries = document.outline_entries(3);

        // 仅空白标题经 compact_text 归一化为 "."，仍包含 H1 和 cost
        assert!(entries[0].display.contains("H1"));
        assert!(entries[0].display.contains("."));
        // ref 仍为 canonical line/level 格式。
        assert_eq!(entries[0].ref_id, "H:L1:H1");
    }

    #[test]
    fn deep_heading_can_be_filtered_to_full_document() {
        let document = MarkdownDocument::parse("Intro\n\n#### Deep\nBody\n".to_owned());

        let entries = document.outline_entries(3);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].ref_id, FULL_DOCUMENT_REF);
    }

    #[test]
    fn ref_uses_structural_coordinates_for_textual_title() {
        let document = MarkdownDocument::parse("# Long Title Here\nBody\n".to_owned());
        let entries = document.outline_entries(3);

        assert_eq!(entries[0].ref_id, "H:L1:H1");
    }

    // @case WB-MD-READ-001
    #[test]
    fn read_canonical_ref_resolves_matching_heading() {
        let document = MarkdownDocument::parse("# Guide\nIntro\n## Install\nBody\n".to_owned());

        // Guide: line=1, level=1
        let resolved = document.resolve_ref("H:L1:H1").unwrap();
        assert_eq!(resolved, ResolvedRef::Heading(&document.headings()[0]));

        // Install: line=3, level=2
        let resolved = document.resolve_ref("H:L3:H2").unwrap();
        assert_eq!(resolved, ResolvedRef::Heading(&document.headings()[1]));
    }

    #[test]
    fn read_canonical_ref_returns_ref_not_found_for_no_match() {
        let document = MarkdownDocument::parse("# Guide\nBody\n".to_owned());

        // Canonical grammar but wrong line
        let error = document
            .resolve_ref("H:L99:H1")
            .expect_err("no such heading");
        assert_eq!(
            error.error().code,
            docnav_protocol::StableErrorCode::RefNotFound
        );

        // Canonical grammar but wrong level
        let error = document.resolve_ref("H:L1:H2").expect_err("wrong level");
        assert_eq!(
            error.error().code,
            docnav_protocol::StableErrorCode::RefNotFound
        );
    }

    #[test]
    fn read_returns_ref_invalid_for_grammar_outside_input() {
        let document = MarkdownDocument::parse("# Guide\nBody\n".to_owned());

        for ref_id in [
            // 字段缺失/错误
            "H:L1",
            "H:L1:H2:extra",
            "X:L1:H1",
            // 前导零
            "H:L01:H1",
            "H:L1:H02",
            // 非法 level
            "H:L1:H0",
            "H:L1:H7",
            // line=0 位于 grammar 外。
            "H:L0:H1",
            // grammar 外字符串
            "not-a-ref",
        ] {
            let error = document
                .resolve_ref(ref_id)
                .expect_err(&format!("should be REF_INVALID: {ref_id}"));
            assert_eq!(
                error.error().code,
                docnav_protocol::StableErrorCode::RefInvalid,
                "{ref_id}"
            );
            // details 包含 ref
            assert_eq!(
                error.error().details.get("ref").and_then(|v| v.as_str()),
                Some(ref_id),
                "{ref_id}"
            );
            // details 包含非空 reason
            let reason = error
                .error()
                .details
                .get("reason")
                .and_then(|v| v.as_str())
                .expect("reason field");
            assert!(!reason.is_empty(), "{ref_id}");
        }
    }

    #[test]
    fn read_ref_not_found_vs_ref_invalid_boundary() {
        let document = MarkdownDocument::parse("# A\nBody\n".to_owned());

        // 合法 canonical ref，但 heading 不存在 → REF_NOT_FOUND
        let error = document.resolve_ref("H:L5:H2").expect_err("not found");
        assert_eq!(
            error.error().code,
            docnav_protocol::StableErrorCode::RefNotFound
        );

        // 当前 grammar 外输入 → REF_INVALID。
        let error = document
            .resolve_ref("H:L1:H1:extra")
            .expect_err("grammar outside");
        assert_eq!(
            error.error().code,
            docnav_protocol::StableErrorCode::RefInvalid
        );
        assert_eq!(
            error.error().details.get("ref").and_then(|v| v.as_str()),
            Some("H:L1:H1:extra")
        );
    }

    #[test]
    fn doc_full_still_resolves_to_full_document() {
        let document = MarkdownDocument::parse("# Guide\nBody\n".to_owned());

        assert_eq!(
            document.resolve_ref(FULL_DOCUMENT_REF).unwrap(),
            ResolvedRef::FullDocument
        );
    }

    // @case WB-MD-LINK-001
    #[test]
    fn outline_to_read_roundtrip_with_canonical_ref() {
        let document =
            MarkdownDocument::parse("# Top\nintro\n## Sub\ndetail\n### Deep\nmore\n".to_owned());

        // outline → ref → read roundtrip
        let entries = document.outline_entries(3);
        for entry in &entries {
            if entry.ref_id == FULL_DOCUMENT_REF {
                continue;
            }
            let resolved = document.resolve_ref(&entry.ref_id);
            assert!(
                resolved.is_ok(),
                "outline ref {} should resolve: {:?}",
                entry.ref_id,
                resolved.err()
            );
        }
    }

    #[test]
    fn find_to_read_roundtrip_with_canonical_ref() {
        let document =
            MarkdownDocument::parse("# Top\nintro target here\n## Sub\ndetail\n".to_owned());

        let entries = document.find_entries("target", 3);
        for entry in &entries {
            if entry.ref_id == FULL_DOCUMENT_REF {
                continue;
            }
            let resolved = document.resolve_ref(&entry.ref_id);
            assert!(
                resolved.is_ok(),
                "find ref {} should resolve: {:?}",
                entry.ref_id,
                resolved.err()
            );
        }
    }
}
