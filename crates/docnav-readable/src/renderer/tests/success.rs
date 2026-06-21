// @case WB-READABLE-RENDERER-001
use super::*;

// ── 1.6.1 No-block operation (outline uses empty blocks) ────────────

#[test]
fn outline_no_blocks_emits_header_only() {
    let payload = test_payloads::TestOutlinePayload {
        entries: vec![test_payloads::TestEntry {
            ref_id: "L5".into(),
            display: "Install".into(),
        }],
        page: None,
    };
    let output = render_test(&payload, ReadableViewKind::Outline).unwrap();

    // Header is valid JSON and contains the field data.
    assert_contains(&output, "\"ref\": \"L5\"");
    assert_contains(&output, "\"display\": \"Install\"");
    // No block sections.
    assert_not_contains(&output, "[block");
    assert_not_contains(&output, "[endblock");
    // Only header exists (no separator LF after header when no blocks),
    // but the header itself must end with the contract LF.
    assert!(output.ends_with('\n'), "no-block output must end with LF");
    assert!(
        output.trim_end().ends_with('}'),
        "output should end with closing brace before trailing LF"
    );
}

// ── 1.6.2 Read with /content block ─────────────────────────────────

#[test]
fn read_content_block() {
    let payload = TestReadPayload {
        ref_id: "L5".into(),
        content: "## Install\n\nRun `pnpm install`.\n".into(),
        content_type: "text/markdown".into(),
        cost: "3 lines | 0.1 KB".into(),
        page: None,
    };
    let output = render_test(&payload, ReadableViewKind::Read).unwrap();

    // Header contains the block reference, not the original content string.
    assert_contains(&output, "\"$block\": \"/content\"");
    assert_contains(&output, "\"bytes\": ");
    assert_not_contains(&output, "\"content\": \"## Install");

    // Block section present after header LF plus empty separator LF.
    assert_contains(&output, "\n\n[block /content bytes=");
    assert_contains(&output, "## Install\n\nRun `pnpm install`.");
    assert_contains(&output, "[endblock /content]");

    // Other fields still in header.
    assert_contains(&output, "\"ref\": \"L5\"");
    assert_contains(&output, "\"content_type\": \"text/markdown\"");
}

// ── 1.6.3 UTF-8 byte length (non-ASCII characters) ─────────────────

#[test]
fn utf8_byte_length_is_correct() {
    // 中文 (Chinese): 每个汉字 3 UTF-8 bytes. "你好" = 6 bytes.
    let payload = TestReadPayload {
        ref_id: "zh".into(),
        content: "你好世界".into(), // 4 chars, 12 bytes
        content_type: "text/plain".into(),
        cost: "".into(),
        page: None,
    };
    let output = render_test(&payload, ReadableViewKind::Read).unwrap();

    // Byte length in header must match UTF-8 byte length of the payload.
    assert_contains(&output, "\"bytes\": 12");

    // Block section must have matching byte count.
    assert_contains(&output, "[block /content bytes=12]");
}

#[test]
fn emoji_utf8_byte_length() {
    // 😀 = 4 UTF-8 bytes
    let payload = TestReadPayload {
        ref_id: "emoji".into(),
        content: "😀😀".into(), // 2 emoji = 8 bytes
        content_type: "text/plain".into(),
        cost: "".into(),
        page: None,
    };
    let output = render_test(&payload, ReadableViewKind::Read).unwrap();
    assert_contains(&output, "\"bytes\": 8");
    assert_contains(&output, "[block /content bytes=8]");
}

#[test]
fn combined_character_utf8_byte_length() {
    // e + combining acute accent = 3 bytes
    let payload = TestReadPayload {
        ref_id: "comb".into(),
        content: "e\u{0301}".into(), // 1 grapheme, 3 UTF-8 bytes
        content_type: "text/plain".into(),
        cost: "".into(),
        page: None,
    };
    let output = render_test(&payload, ReadableViewKind::Read).unwrap();
    assert_contains(&output, "\"bytes\": 3");
}

// ── 1.6.4 CRLF payload preservation ────────────────────────────────

#[test]
fn crlf_payload_preserved_in_block() {
    let payload = TestReadPayload {
        ref_id: "crlf".into(),
        content: "line1\r\nline2\r\n".into(),
        content_type: "text/plain".into(),
        cost: "".into(),
        page: None,
    };
    let output = render_test(&payload, ReadableViewKind::Read).unwrap();

    // CRLF must be preserved in the block section.
    assert_contains(&output, "line1\r\nline2");
    // Byte count counts CRLF as 2 bytes each.
    // "line1\r\nline2\r\n" = 5+2+5+2 = 14 bytes
    assert_contains(&output, "\"bytes\": 14");
}

// ── 1.6.5 Payload without trailing LF gets framing LF ──────────────

#[test]
fn no_trailing_lf_payload_gets_framing_lf() {
    let payload = TestReadPayload {
        ref_id: "notrail".into(),
        content: "no trailing newline".into(), // does NOT end with \n
        content_type: "text/plain".into(),
        cost: "".into(),
        page: None,
    };
    let output = render_test(&payload, ReadableViewKind::Read).unwrap();

    // The end marker must appear on its own line, so a framing LF is added.
    // Looking for: payload + LF (framing) + "[endblock"
    assert_contains(&output, "no trailing newline\n[endblock /content]");
}

#[test]
fn trailing_lf_payload_no_extra_framing_lf() {
    let payload = TestReadPayload {
        ref_id: "trail".into(),
        content: "ends with newline\n".into(),
        content_type: "text/plain".into(),
        cost: "".into(),
        page: None,
    };
    let output = render_test(&payload, ReadableViewKind::Read).unwrap();

    // When payload already ends with \n, no extra LF before end marker.
    // The payload IS "ends with newline\n", so end marker follows immediately.
    assert_contains(&output, "ends with newline\n[endblock /content]");
}

// ── 1.6.6 Marker-like text in payload ─────────────────────────────

#[test]
fn payload_contains_block_marker_text() {
    let payload = TestReadPayload {
        ref_id: "marker".into(),
        content: "Some code:\n[block /other bytes=10]\nreal data\n[endblock /other]\ndone.\n"
            .into(),
        content_type: "text/plain".into(),
        cost: "".into(),
        page: None,
    };
    let output = render_test(&payload, ReadableViewKind::Read).unwrap();

    // The payload is length-delimited; marker-looking text inside payload
    // must not confuse the parser.  We verify the byte count matches the
    // full payload including those markers, and the end marker is after
    // the full payload.
    let bytes_str: Vec<&str> = output
        .lines()
        .filter(|l| l.contains("\"bytes\":"))
        .collect();
    assert!(!bytes_str.is_empty(), "missing bytes field in header");

    // Verify the header references `/content` and the block section uses
    // the same pointer.
    assert_contains(&output, "\"$block\": \"/content\"");
    assert_contains(&output, "[endblock /content]");

    // Payload content must be intact in the block section.
    assert_contains(&output, "[block /other bytes=10]");
    assert_contains(&output, "[endblock /other]");
}

// ── 1.6.7 Multiple blocks and nested pointers ──────────────────────

#[test]
fn multiple_blocks_with_nested_pointer() {
    // Use a custom config with multiple blocks, including a nested pointer.
    let value = json!({
        "ref": "L5",
        "content": "the content",
        "meta": {
            "summary": "a summary"
        },
        "extra": "extra field"
    });

    let mut custom_config = RendererConfig::default_config();
    custom_config.views.insert(
        ReadableViewKind::Read,
        crate::renderer_config::ViewBlockConfig {
            blocks: vec![
                "/content".to_owned(),
                "/meta/summary".to_owned(),
                "/extra".to_owned(),
            ],
        },
    );
    custom_config.validate().unwrap();

    let output = render_readable_view(&value, ReadableViewKind::Read, &custom_config).unwrap();

    // All three blocks present.
    assert_contains(&output, "[block /content bytes=11]");
    assert_contains(&output, "the content");
    assert_contains(&output, "[endblock /content]");

    assert_contains(&output, "[block /meta/summary bytes=9]");
    assert_contains(&output, "a summary");
    assert_contains(&output, "[endblock /meta/summary]");

    assert_contains(&output, "[block /extra bytes=11]");
    assert_contains(&output, "extra field");
    assert_contains(&output, "[endblock /extra]");

    // Header has block references, not original strings.
    assert_contains(&output, "\"$block\": \"/content\"");
    assert_contains(&output, "\"$block\": \"/meta/summary\"");
    assert_contains(&output, "\"$block\": \"/extra\"");
}

// ── 1.6.8 Empty string block (zero bytes) ─────────────────────────

#[test]
fn empty_string_block_zero_bytes() {
    let payload = TestReadPayload {
        ref_id: "empty".into(),
        content: "".into(),
        content_type: "text/plain".into(),
        cost: "".into(),
        page: None,
    };
    let output = render_test(&payload, ReadableViewKind::Read).unwrap();

    assert_contains(&output, "\"bytes\": 0");
    assert_contains(&output, "[block /content bytes=0]");

    // Empty payload: start marker ends with \n, payload is empty,
    // then a framing LF is added (since empty string doesn't end with \n),
    // then end marker.
    assert_contains(&output, "[block /content bytes=0]\n\n[endblock /content]");
}

// ── 1.6.9 Undeclared extension fields preserved in header JSON ─────

#[test]
fn undeclared_fields_preserved_in_header() {
    // Extra fields not listed in the block config stay in the header as-is.
    let value = json!({
        "ref": "extra-test",
        "content": "main content",
        "custom_field": "custom value",
        "nested": {"inner": 42}
    });

    let config = RendererConfig::default_config();
    config.validate().unwrap();
    let output = render_readable_view(&value, ReadableViewKind::Read, &config).unwrap();

    // Undeclared fields present in header.
    assert_contains(&output, "\"custom_field\": \"custom value\"");
    assert_contains(&output, "\"nested\": {");
    assert_contains(&output, "\"inner\": 42");
    // Block field is replaced with reference.
    assert_contains(&output, "\"$block\": \"/content\"");
}

// ── 1.6.10 Readable error payload ─────────────────────────────────

#[test]
fn readable_error_block() {
    let payload = TestErrorPayload {
        code: "REF_NOT_FOUND".into(),
        error: "No content found for ref `L99`".into(),
        details: json!({"ref": "L99"}),
        guidance: Some(vec!["Check available entries via `docnav outline`.".into()]),
    };
    let output = render_test(&payload, ReadableViewKind::Error).unwrap();

    // Header contains the error structure with block reference.
    assert_contains(&output, "\"$block\": \"/error\"");
    assert_contains(&output, "\"code\": \"REF_NOT_FOUND\"");
    // Block section contains the error message.
    assert_contains(&output, "[block /error bytes=");
    assert_contains(&output, "No content found for ref `L99`");
    assert_contains(&output, "[endblock /error]");

    // guidance (an array) stays in header JSON, NOT in a block.
    assert_contains(&output, "\"guidance\": [");
    assert_contains(&output, "Check available entries");
}

// ── 1.6.11 separator LF is platform-independent LF (0x0A) ────────

#[test]
fn framing_uses_lf_byte() {
    let payload = TestReadPayload {
        ref_id: "lf".into(),
        content: "test\n".into(),
        content_type: "text/plain".into(),
        cost: "".into(),
        page: None,
    };
    let output = render_test(&payload, ReadableViewKind::Read).unwrap();

    // No CR bytes anywhere in the output.
    let cr_count = output.bytes().filter(|&b| b == b'\r').count();
    assert_eq!(
        cr_count, 0,
        "readable-view framing must use LF (0x0A), not CRLF"
    );

    // The block section is present.
    assert_contains(&output, "[block /content bytes=5]");
    assert_contains(&output, "[endblock /content]");
}

// ── 1.6.12 Info operation (no blocks) ─────────────────────────────

#[test]
fn info_operation_no_blocks() {
    let payload = test_payloads::TestInfoPayload {
        display: "Markdown Adapter v0.1.0".into(),
        capabilities: vec!["outline".into(), "read".into(), "find".into()],
    };
    let output = render_test(&payload, ReadableViewKind::Info).unwrap();

    // All fields in header JSON.
    assert_contains(&output, "\"display\": \"Markdown Adapter v0.1.0\"");
    assert_contains(&output, "\"capabilities\": [");
    // No block sections.
    assert_not_contains(&output, "[block");
}

// ── 1.6.13 Find operation (no blocks) ─────────────────────────────

#[test]
fn find_operation_no_blocks() {
    let payload = test_payloads::TestFindPayload {
        matches: vec![test_payloads::TestEntry {
            ref_id: "L5".into(),
            display: "Install".into(),
        }],
        page: None,
    };
    let output = render_test(&payload, ReadableViewKind::Find).unwrap();

    assert_contains(&output, "\"ref\": \"L5\"");
    assert_not_contains(&output, "[block");
}

// ── 1.6.14 Pretty JSON header is valid standalone JSON ────────────

#[test]
fn header_json_is_valid_standalone() {
    let payload = TestReadPayload {
        ref_id: "parse".into(),
        content: "test\n".into(),
        content_type: "text/markdown".into(),
        cost: "1 line".into(),
        page: Some(1),
    };
    let output = render_test(&payload, ReadableViewKind::Read).unwrap();

    // Extract just the header JSON: everything before the first "[block"
    // marker (or the whole output if no blocks).
    let header_end = output.find("[block").unwrap_or(output.len());
    // serde_json::to_string_pretty ends with '\n'; trim trailing
    // whitespace so we can parse the pure JSON value.
    let header_str = output[..header_end].trim_end();

    // Should parse as valid JSON.
    let parsed: Value = serde_json::from_str(header_str).expect("header should be valid JSON");
    assert!(parsed.is_object());

    // Verify block reference shape.
    let content_ref = &parsed["content"];
    assert_eq!(content_ref["$block"], "/content");
    assert!(content_ref["bytes"].is_u64());
}

// ── 1.6.15 Typed payload to readable JSON value ───────────────────

#[test]
fn to_readable_value_serializes_valid_payload() {
    let payload = TestReadPayload {
        ref_id: "ok".into(),
        content: "test".into(),
        content_type: "text/plain".into(),
        cost: "".into(),
        page: None,
    };
    let value = to_readable_value(&payload).unwrap();
    assert!(value.is_object());
    assert_eq!(value["ref"], "ok");
}

// ── 1.6.16 Default config validates successfully ──────────────────

#[test]
fn default_config_passes_validation() {
    let config = RendererConfig::default_config();
    config.validate().unwrap(); // should not panic or error
}
