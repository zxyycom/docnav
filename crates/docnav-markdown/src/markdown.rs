mod document;
mod options;
mod parse;
mod refs;
mod text;

pub use document::{
    is_markdown_extension, is_utf8_markdown_candidate, MarkdownDocument, ResolvedRef,
};
pub use options::max_heading_level_from_options;
pub use text::cost_for;

#[cfg(test)]
mod tests {
    use super::refs::{ParsedRef, FULL_DOCUMENT_REF};
    use super::*;

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
    fn duplicate_full_paths_receive_unique_refs() {
        let document = MarkdownDocument::parse("# A\n## B\n# A\n## B\n".to_owned());

        let entries = document.outline_entries(3);
        let duplicate_refs: Vec<&str> = entries
            .iter()
            .filter(|entry| entry.ref_id.contains("A > B"))
            .map(|entry| entry.ref_id.as_str())
            .collect();

        assert_eq!(duplicate_refs, vec!["L2:A > B", "L4#2:A > B"]);
    }

    #[test]
    fn heading_refs_omit_default_occurrence() {
        let document = MarkdownDocument::parse("# Guide\n\n## Install\n".to_owned());
        let entries = document.outline_entries(3);
        let refs: Vec<&str> = entries.iter().map(|entry| entry.ref_id.as_str()).collect();

        assert_eq!(refs, vec!["L1:Guide", "L3:Guide > Install"]);
    }

    #[test]
    fn explicit_default_occurrence_resolves_first_heading() {
        let document = MarkdownDocument::parse("# Guide\nOne\n# Guide\nTwo\n".to_owned());

        let resolved = document.resolve_ref("L1#1:Guide").unwrap();

        assert_eq!(resolved, ResolvedRef::Heading(&document.headings()[0]));
    }

    #[test]
    fn canonical_duplicate_occurrence_resolves_matching_heading() {
        let document = MarkdownDocument::parse("# Repeat\nOne\n# Repeat\nTwo\n".to_owned());

        let resolved = document.resolve_ref("L3#2:Repeat").unwrap();

        assert_eq!(resolved, ResolvedRef::Heading(&document.headings()[1]));
    }

    #[test]
    fn old_bracketed_occurrence_suffix_is_path_text_not_metadata() {
        let document = MarkdownDocument::parse("# Guide\nBody\n".to_owned());
        let suffix = [" [", "docnav", ":", "1", "]"].concat();
        let old_ref = format!("L1:Guide{suffix}");

        assert_eq!(
            ParsedRef::parse(&old_ref),
            Some(ParsedRef::Heading {
                line: 1,
                path: format!("Guide{suffix}"),
                occurrence: 1,
            })
        );
        assert!(document.resolve_ref(&old_ref).is_err());
    }

    #[test]
    fn heading_path_may_end_with_legacy_marker_like_text() {
        let suffix = [" [", "docnav", ":", "1", "]"].concat();
        let title = format!("Guide{suffix}");
        let document = MarkdownDocument::parse(format!("# {title}\nBody\n"));
        let entries = document.outline_entries(3);

        assert_eq!(entries[0].ref_id, format!("L1:{title}"));
        assert_eq!(
            document.resolve_ref(&entries[0].ref_id).unwrap(),
            ResolvedRef::Heading(&document.headings()[0])
        );
    }

    #[test]
    fn parsed_ref_accepts_extra_colons_in_path() {
        assert_eq!(
            ParsedRef::parse("L7#2:Guide: Install"),
            Some(ParsedRef::Heading {
                line: 7,
                path: "Guide: Install".to_owned(),
                occurrence: 2,
            })
        );
    }

    #[test]
    fn parsed_ref_rejects_invalid_line_ordinal_and_path() {
        for ref_id in [
            "L0:Guide",
            "Lx:Guide",
            "L1#:Guide",
            "L1#0:Guide",
            "L1#x:Guide",
            "L1:",
        ] {
            assert_eq!(ParsedRef::parse(ref_id), None, "{ref_id}");
        }
    }

    #[test]
    fn doc_full_still_resolves_to_full_document() {
        let document = MarkdownDocument::parse("# Guide\nBody\n".to_owned());

        assert_eq!(
            document.resolve_ref(FULL_DOCUMENT_REF).unwrap(),
            ResolvedRef::FullDocument
        );
    }

    #[test]
    fn deep_heading_can_be_filtered_to_full_document() {
        let document = MarkdownDocument::parse("Intro\n\n#### Deep\nBody\n".to_owned());

        let entries = document.outline_entries(3);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].ref_id, FULL_DOCUMENT_REF);
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

    #[test]
    fn frontmatter_does_not_create_outline_heading() {
        let document = MarkdownDocument::parse("---\ntitle: Sample\n---\n\n# Real\n".to_owned());

        assert_eq!(document.headings().len(), 1);
        assert_eq!(document.headings()[0].title, "Real");
    }
}
