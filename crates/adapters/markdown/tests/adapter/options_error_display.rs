use super::*;

// @case WB-MD-OPTIONS-001
#[test]
fn adapter_owned_options_shape_outline_and_find_granularity() {
    let path = write_doc("adapter-owned-options.md", "# Top\n\n#### Deep\nneedle\n");
    let default_outline = OutlineArguments {
        limit: positive(6000),
        page: positive(1),
        options: Some(max_heading_level_options(3)),
    };
    let expanded_outline = OutlineArguments {
        options: Some(max_heading_level_options(4)),
        ..default_outline.clone()
    };

    let default = outline_result(&path, &default_outline);
    assert_eq!(entry_refs(&default.entries), vec!["H:L1:H1"]);

    let expanded = outline_result(&path, &expanded_outline);
    assert_eq!(entry_refs(&expanded.entries), vec!["H:L1:H1", "H:L3:H4"]);

    let default_find = FindArguments {
        query: "needle".to_owned(),
        limit: positive(6000),
        page: positive(1),
        options: Some(max_heading_level_options(3)),
    };
    let expanded_find = FindArguments {
        options: Some(max_heading_level_options(4)),
        ..default_find.clone()
    };

    let default_matches = find_result(&path, &default_find);
    assert_eq!(entry_refs(&default_matches.matches), vec!["H:L1:H1"]);

    let expanded_matches = find_result(&path, &expanded_find);
    assert_eq!(entry_refs(&expanded_matches.matches), vec!["H:L3:H4"]);
}

// @case WB-MD-ERROR-001
#[test]
fn non_utf8_document_returns_stable_encoding_error() {
    let path = write_bytes("bad.md", &[0xFF, 0xFE, 0x00]);
    let arguments = outline_args(6000, 1, None);
    let request = make_request(
        &path,
        Operation::Outline,
        OperationArguments::Outline(arguments.clone()),
    );

    let error = MarkdownAdapter
        .outline(&request, &arguments)
        .expect_err("non UTF-8 fails");

    assert_eq!(
        error.protocol_error().code(),
        ProtocolDiagnosticCode::DocumentEncodingUnsupported
    );
}

// @case WB-MD-DISPLAY-001
#[test]
fn outline_entries_include_heading_title() {
    let path = write_doc("display.md", "# Installation Guide\n\n## Setup\nBody\n");
    let arguments = outline_args(6000, 1, None);
    let result = outline_result(&path, &arguments);

    assert_eq!(result.entries.len(), 2);
    assert_eq!(result.entries[0].label, "Installation Guide");
    assert_eq!(result.entries[1].label, "Setup");
    // ref 不包含 title
    assert!(!result.entries[0].ref_id.contains("Installation"));
}

#[test]
fn find_entry_contains_match_snippet() {
    let path = write_doc(
        "find-display.md",
        "# Top\nfind this needle here\n## Other\nmore text\n",
    );
    let arguments = find_args("needle", 6000, 1, Some(3));
    let result = find_result(&path, &arguments);

    assert_eq!(result.matches.len(), 1);
    assert!(result.matches[0].label.contains("needle"));
    // ref 不受 label 内容影响
    assert_canonical_ref(&result.matches[0].ref_id);
}
