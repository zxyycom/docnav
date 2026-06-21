use super::*;

// @case WB-MD-OPTIONS-001
#[test]
fn adapter_owned_options_shape_outline_and_find_granularity() {
    let path = write_doc("adapter-owned-options.md", "# Top\n\n#### Deep\nneedle\n");
    let default_outline = OutlineArguments {
        limit_chars: positive(6000),
        page: positive(1),
        options: None,
    };
    let expanded_outline = OutlineArguments {
        options: Some(max_heading_level_options(4)),
        ..default_outline.clone()
    };

    let default = outline_result(&path, &default_outline);
    assert_eq!(entry_refs(&default.entries), vec!["H:L1:H1:I1"]);

    let expanded = outline_result(&path, &expanded_outline);
    assert_eq!(
        entry_refs(&expanded.entries),
        vec!["H:L1:H1:I1", "H:L3:H4:I2"]
    );

    let default_find = FindArguments {
        query: "needle".to_owned(),
        limit_chars: positive(6000),
        page: positive(1),
        options: None,
    };
    let expanded_find = FindArguments {
        options: Some(max_heading_level_options(4)),
        ..default_find.clone()
    };

    let default_matches = find_result(&path, &default_find);
    assert_eq!(entry_refs(&default_matches.matches), vec!["H:L1:H1:I1"]);

    let expanded_matches = find_result(&path, &expanded_find);
    assert_eq!(entry_refs(&expanded_matches.matches), vec!["H:L3:H4:I2"]);
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
        error.error().code,
        StableErrorCode::DocumentEncodingUnsupported
    );
}

// @case WB-MD-INVOKE-001
#[test]
fn invoke_writes_protocol_envelope() {
    let path = write_doc("invoke.md", "# A\nBody\n");
    let request = make_request(
        &path,
        Operation::Outline,
        OperationArguments::Outline(outline_args(6000, 1, None)),
    );
    let input = serde_json::to_vec(&request).expect("request JSON");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = invoke_once(&MarkdownAdapter, input.as_slice(), &mut stdout, &mut stderr);

    assert_eq!(exit, AdapterExitCode::Success.code());
    assert!(stderr.is_empty());
    let response: ProtocolResponse = serde_json::from_slice(&stdout).expect("protocol response");
    match response {
        ProtocolResponse::Success(response) => {
            assert_eq!(response.operation, Operation::Outline);
            assert!(response.ok);
        }
        ProtocolResponse::Failure(_) => panic!("expected success"),
    }
}

// @case WB-MD-DISPLAY-001
#[test]
fn outline_display_includes_heading_title() {
    let path = write_doc("display.md", "# Installation Guide\n\n## Setup\nBody\n");
    let arguments = outline_args(6000, 1, None);
    let result = outline_result(&path, &arguments);

    assert_eq!(result.entries.len(), 2);
    // display 包含 heading title
    assert!(result.entries[0].display.contains("Installation Guide"));
    assert!(result.entries[1].display.contains("Setup"));
    // ref 不包含 title
    assert!(!result.entries[0].ref_id.contains("Installation"));
}

#[test]
fn find_display_contains_match_snippet() {
    let path = write_doc(
        "find-display.md",
        "# Top\nfind this needle here\n## Other\nmore text\n",
    );
    let arguments = find_args("needle", 6000, 1, Some(3));
    let result = find_result(&path, &arguments);

    assert_eq!(result.matches.len(), 1);
    // find display 保留匹配片段
    assert!(result.matches[0].display.contains("needle"));
    // ref 不受 display 内容影响
    assert_canonical_ref(&result.matches[0].ref_id);
}
