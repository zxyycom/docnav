use super::*;

fn assert_cost_measurements(cost: &docnav_protocol::Cost, scope: &str, text: &str) {
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
                docnav_text_cost::line_cost(text).value,
                Some(scope)
            ),
            (
                "bytes",
                docnav_text_cost::byte_cost(text).value,
                Some(scope)
            ),
            (
                "tokens",
                docnav_text_cost::token_cost(text).value,
                Some(scope)
            ),
        ]
    );
}

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
fn outline_refs_consistent_under_different_max_heading_level() {
    let document = MarkdownDocument::parse("# Top\n\n## A\n\n### Deep\n\n#### Hidden\n".to_owned());

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
fn outline_entry_includes_title_level_and_cost() {
    let source = "# Guide\nContent here\n";
    let document = MarkdownDocument::parse(source.to_owned());
    let entries = document.outline_entries(3);

    assert_eq!(entries[0].label, "Guide");
    assert_eq!(entries[0].metadata.as_ref().unwrap()["heading_level"], 1);
    assert_cost_measurements(entries[0].cost.as_ref().unwrap(), "entry", source);
}

#[test]
fn outline_entry_handles_whitespace_only_title() {
    let document = MarkdownDocument::parse("# \nContent\n".to_owned());
    let entries = document.outline_entries(3);

    assert!(!entries[0].label.trim().is_empty());
    assert_eq!(entries[0].metadata.as_ref().unwrap()["heading_level"], 1);
    assert_eq!(entries[0].ref_id, "H:L1:H1");
}
