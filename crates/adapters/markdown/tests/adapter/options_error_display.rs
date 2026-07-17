use super::*;

// @case WB-MD-OPTIONS-001
#[test]
fn adapter_owned_options_shape_outline_and_find_granularity() {
    let path = write_doc("adapter-owned-options.md", "# Top\n\n#### Deep\nneedle\n");
    let default_outline = outline_input(&path, 6000, 1, Some(3));
    let expanded_outline = OutlineInput {
        max_heading_level: Some(4),
        ..default_outline.clone()
    };

    let default = outline_result(&default_outline);
    assert_eq!(entry_refs(&default.entries), vec!["H:L1:H1"]);

    let expanded = outline_result(&expanded_outline);
    assert_eq!(entry_refs(&expanded.entries), vec!["H:L1:H1", "H:L3:H4"]);

    let default_find = find_input(&path, "needle", 6000, 1, Some(3));
    let expanded_find = FindInput {
        max_heading_level: Some(4),
        ..default_find.clone()
    };

    let default_matches = find_result(&default_find);
    assert_eq!(entry_refs(&default_matches.matches), vec!["H:L1:H1"]);

    let expanded_matches = find_result(&expanded_find);
    assert_eq!(entry_refs(&expanded_matches.matches), vec!["H:L3:H4"]);
}

#[test]
fn outline_consumes_max_heading_level_from_standard_input() {
    let path = write_doc("typed-input.md", "# Top\n\n#### Deep\n");
    let input = outline_input(&path, 6000, 1, Some(4));
    let result = outline_result(&input);

    assert_eq!(entry_refs(&result.entries), vec!["H:L1:H1", "H:L3:H4"]);
}

#[test]
fn outline_does_not_default_a_missing_max_heading_level() {
    let path = write_doc("missing-heading-level.md", "# Top\n");
    let mut input = outline_input(&path, 6000, 1, Some(3));
    input.max_heading_level = None;

    let error = MarkdownAdapter
        .outline(&input)
        .expect_err("missing typed input must not be defaulted");

    assert_eq!(
        error.protocol_error().code(),
        ProtocolDiagnosticCode::InternalError
    );
}

#[test]
fn outline_rejects_out_of_range_max_heading_level_at_adapter_boundary() {
    let path = write_doc("max-heading-level-invalid.md", "# Top\n");
    for level in [0, 7] {
        let input = outline_input(&path, 6000, 1, Some(level));
        let error = MarkdownAdapter
            .outline(&input)
            .expect_err("adapter must reject an out-of-range heading level");
        let protocol_error = error.protocol_error();

        assert_eq!(
            protocol_error.code(),
            ProtocolDiagnosticCode::InvalidRequest
        );
        assert_eq!(protocol_error.owner(), "adapter_options");
        assert_eq!(
            protocol_error
                .details()
                .get("reason")
                .and_then(serde_json::Value::as_str),
            Some("range_invalid")
        );
        assert_eq!(
            protocol_error
                .details()
                .get("field")
                .and_then(serde_json::Value::as_str),
            Some("arguments.options.max_heading_level")
        );
        assert_eq!(
            protocol_error.expected(),
            Some(&serde_json::json!("integer in range 1..6"))
        );
        assert_eq!(
            protocol_error.received(),
            Some(&serde_json::json!(level.to_string()))
        );
    }
}

// @case WB-MD-ERROR-001
#[test]
fn non_utf8_document_returns_stable_encoding_error() {
    let path = write_bytes("bad.md", &[0xFF, 0xFE, 0x00]);
    let input = outline_input(&path, 6000, 1, None);

    let error = MarkdownAdapter
        .outline(&input)
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
    let input = outline_input(&path, 6000, 1, None);
    let result = outline_result(&input);

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
    let input = find_input(&path, "needle", 6000, 1, Some(3));
    let result = find_result(&input);

    assert_eq!(result.matches.len(), 1);
    assert!(result.matches[0].label.contains("needle"));
    // ref 不受 label 内容影响
    assert_canonical_ref(&result.matches[0].ref_id);
}
