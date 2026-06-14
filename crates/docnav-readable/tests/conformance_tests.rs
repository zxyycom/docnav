//! Integration tests that load committed conformance vector fixtures and
//! verify renderer output against order-independent assertions.
//!
//! Each fixture file under `tests/fixtures/conformance/` is loaded at compile
//! time via `include_str!`.  The test harness:
//!
//! 1. Deserializes the fixture into a [`ConformanceVector`].
//! 2. Builds a renderer config (default or overridden).
//! 3. Runs the renderer or verifies the expected failure.
//! 4. Checks every assertion against the output.
//!
//! # Stable assertion scope
//!
//! Assertions focus on **block pointer**, **byte length**, and **block
//! payload**.  Header key order, multi-block output order, and byte-for-byte
//! consistency are intentionally excluded from the stable contract.

use docnav_readable::config::{RendererConfig, ViewBlockConfig};
use docnav_readable::conformance::{Assertion, ConformanceVector};
use docnav_readable::renderer::render_readable_view;
use docnav_readable::view_kind::ReadableViewKind;
use serde_json::Value;

// ── Fixture loading ──────────────────────────────────────────────────────

/// Macro to load a fixture file at compile time and deserialize it.
macro_rules! load_vector {
    ($path:literal) => {
        serde_json::from_str::<ConformanceVector>(include_str!($path))
            .expect(concat!("failed to parse conformance vector: ", $path))
    };
}

/// Parse a view kind string from a fixture into a `ReadableViewKind`.
fn parse_view_kind(s: &str) -> ReadableViewKind {
    match s {
        "outline" => ReadableViewKind::Outline,
        "read" => ReadableViewKind::Read,
        "find" => ReadableViewKind::Find,
        "info" => ReadableViewKind::Info,
        "error" => ReadableViewKind::Error,
        "warning" => ReadableViewKind::Warning,
        other => panic!("unknown view_kind in conformance vector: {other}"),
    }
}

// ── Block extraction from output ─────────────────────────────────────────

/// A parsed block section extracted from readable-view output.
#[derive(Debug)]
struct ParsedBlock {
    pointer: String,
    byte_length: u64,
    payload: String,
}

/// Parse all `[block ...]...[endblock ...]` sections from output.
fn parse_blocks(output: &str) -> Vec<ParsedBlock> {
    let mut blocks = Vec::new();
    let bytes = output.as_bytes();
    let Some((_header_end, mut cursor)) = first_block_boundary(bytes) else {
        return blocks;
    };

    while let Some(start_rel) = find_bytes(&bytes[cursor..], b"[block ") {
        let start = cursor + start_rel;

        // Find end of start marker line: "[block <pointer> bytes=<n>]\n"
        let start_line_end = find_byte(&bytes[start..], b'\n')
            .map(|pos| start + pos)
            .expect("block start marker missing LF");
        let start_line = std::str::from_utf8(&bytes[start..start_line_end])
            .expect("block start marker should be UTF-8");

        // Parse pointer and byte_length from the start marker.
        // Format: [block <pointer> bytes=<n>]
        let inner = start_line
            .strip_prefix("[block ")
            .and_then(|s| s.strip_suffix(']'))
            .expect("malformed block start marker");

        let (pointer, bytes_part) = inner
            .rsplit_once(" bytes=")
            .expect("block start marker missing 'bytes='");

        let byte_length: u64 = bytes_part.parse().expect("block bytes value not a number");
        let byte_length_usize =
            usize::try_from(byte_length).expect("block bytes value exceeds usize");

        // Consume exactly the declared UTF-8 payload bytes. Marker-looking text
        // inside the payload is ordinary content and must not affect parsing.
        let payload_start = start_line_end + 1; // after the start line LF
        let payload_end = payload_start
            .checked_add(byte_length_usize)
            .expect("block payload byte range overflow");
        assert!(
            payload_end <= bytes.len(),
            "block {pointer:?} declares {byte_length} bytes, beyond output length"
        );
        let payload = std::str::from_utf8(&bytes[payload_start..payload_end])
            .expect("block payload should be valid UTF-8")
            .to_owned();

        // If the payload did not end with LF, the renderer inserts one framing
        // LF before the end marker. That LF is not part of the payload.
        let mut marker_start = payload_end;
        if !bytes[payload_start..payload_end].ends_with(b"\n") {
            assert!(
                bytes.get(marker_start) == Some(&b'\n'),
                "block {pointer:?} missing framing LF before end marker"
            );
            marker_start += 1;
        }

        let end_marker = format!("[endblock {pointer}]\n");
        let end_marker_bytes = end_marker.as_bytes();
        assert!(
            bytes[marker_start..].starts_with(end_marker_bytes),
            "block {pointer:?} missing end marker after declared payload bytes"
        );

        blocks.push(ParsedBlock {
            pointer: pointer.to_owned(),
            byte_length,
            payload,
        });

        // Advance past the end marker line.
        cursor = marker_start + end_marker_bytes.len();
    }

    blocks
}

fn find_byte(bytes: &[u8], needle: u8) -> Option<usize> {
    bytes.iter().position(|byte| *byte == needle)
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }

    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn first_block_boundary(bytes: &[u8]) -> Option<(usize, usize)> {
    find_bytes(bytes, b"\n\n[block ").map(|separator| (separator, separator + 2))
}

/// Extract the header JSON portion from output (everything before the first
/// block boundary, trimmed).
fn extract_header(output: &str) -> &str {
    let header_end = first_block_boundary(output.as_bytes())
        .map(|(header_end, _block_start)| header_end)
        .unwrap_or(output.len());
    output[..header_end].trim_end()
}

fn assert_restores_source(
    vector: &ConformanceVector,
    header_value: &Value,
    blocks: &[ParsedBlock],
) {
    let mut restored = header_value.clone();

    for block in blocks {
        let header_ref = header_value.pointer(&block.pointer).unwrap_or_else(|| {
            panic!(
                "header block reference {pointer:?} not found.\nVector: {desc}",
                pointer = block.pointer,
                desc = vector.description,
            )
        });
        assert_eq!(
            header_ref.pointer("/$block").and_then(Value::as_str),
            Some(block.pointer.as_str()),
            "header block reference {pointer:?} has wrong $block value.\nVector: {desc}",
            pointer = block.pointer,
            desc = vector.description,
        );
        assert_eq!(
            header_ref.pointer("/bytes").and_then(Value::as_u64),
            Some(block.byte_length),
            "header block reference {pointer:?} has wrong byte length.\nVector: {desc}",
            pointer = block.pointer,
            desc = vector.description,
        );

        let target = restored.pointer_mut(&block.pointer).unwrap_or_else(|| {
            panic!(
                "restored header target {pointer:?} not found.\nVector: {desc}",
                pointer = block.pointer,
                desc = vector.description,
            )
        });
        *target = Value::String(block.payload.clone());
    }

    assert_eq!(
        &restored,
        &vector.input,
        "readable-view header plus parsed blocks does not restore source payload.\n\
         Vector: {desc}",
        desc = vector.description,
    );
}

// ── Assertion runner ─────────────────────────────────────────────────────

/// Run all assertions for a vector against the renderer output.
fn check_assertions(vector: &ConformanceVector, output: &str, is_failure: bool) {
    // For failure vectors, assertions are typically empty — the expected_failure
    // block carries the verification.  We skip header/block parsing since the
    // output is an error message string, not readable-view output.
    if is_failure {
        // Only check assertions that make sense against an error message.
        for assertion in &vector.assertions {
            match assertion {
                Assertion::OutputContains { text } => {
                    assert!(
                        output.contains(text.as_str()),
                        "error message does not contain expected text.\n\
                         Expected: {text:?}\n\
                         Message: {output}\n\
                         Vector: {desc}",
                        text = text,
                        output = output,
                        desc = vector.description,
                    );
                }
                _ => {
                    panic!(
                        "assertion type {:?} is not valid for failure vectors.\n\
                         Vector: {desc}",
                        std::mem::discriminant(assertion),
                        desc = vector.description,
                    );
                }
            }
        }
        return;
    }

    let header_str = extract_header(output);
    let header_value: Value =
        serde_json::from_str(header_str).expect("header JSON should be valid");
    let blocks = parse_blocks(output);
    assert_restores_source(vector, &header_value, &blocks);

    for assertion in &vector.assertions {
        match assertion {
            Assertion::Block {
                pointer,
                byte_length,
                payload,
                payload_contains,
            } => {
                let matched = blocks.iter().find(|b| &b.pointer == pointer);
                assert!(
                    matched.is_some(),
                    "expected block with pointer {pointer:?}, but not found in output.\n\
                     Found blocks: {blocks:?}\n\
                     Vector: {desc}",
                    pointer = pointer,
                    blocks = blocks.iter().map(|b| &b.pointer).collect::<Vec<_>>(),
                    desc = vector.description,
                );
                let block = matched.unwrap();

                if let Some(expected_len) = byte_length {
                    assert_eq!(
                        block.byte_length, *expected_len,
                        "block {pointer:?} byte_length mismatch: expected {expected_len}, got {actual}.\n\
                         Vector: {desc}",
                        pointer = pointer,
                        expected_len = expected_len,
                        actual = block.byte_length,
                        desc = vector.description,
                    );
                }

                if let Some(expected_payload) = payload {
                    assert_eq!(
                        &block.payload,
                        expected_payload,
                        "block {pointer:?} payload mismatch.\n\
                         Expected: {expected_payload:?}\n\
                         Actual:   {actual:?}\n\
                         Vector: {desc}",
                        pointer = pointer,
                        expected_payload = expected_payload,
                        actual = block.payload,
                        desc = vector.description,
                    );
                }

                if let Some(expected_substr) = payload_contains {
                    assert!(
                        block.payload.contains(expected_substr.as_str()),
                        "block {pointer:?} payload does not contain expected text.\n\
                         Expected substring: {expected_substr:?}\n\
                         Actual payload: {payload:?}\n\
                         Vector: {desc}",
                        pointer = pointer,
                        expected_substr = expected_substr,
                        payload = block.payload,
                        desc = vector.description,
                    );
                }
            }

            Assertion::NoBlocks => {
                assert!(
                    blocks.is_empty(),
                    "expected no blocks, but found {} block(s): {found:?}.\n\
                     Vector: {desc}",
                    blocks.len(),
                    found = blocks.iter().map(|b| &b.pointer).collect::<Vec<_>>(),
                    desc = vector.description,
                );
                assert!(
                    !output.contains("[block"),
                    "output should not contain '[block' marker.\nVector: {desc}",
                    desc = vector.description,
                );
                assert!(
                    !output.contains("[endblock"),
                    "output should not contain '[endblock' marker.\nVector: {desc}",
                    desc = vector.description,
                );
            }

            Assertion::HeaderField { pointer, value } => {
                let actual = header_value.pointer(pointer).unwrap_or_else(|| {
                    panic!(
                        "header field {pointer:?} not found in header JSON.\n\
                         Header: {header_str}\n\
                         Vector: {desc}",
                        pointer = pointer,
                        header_str = header_str,
                        desc = vector.description,
                    )
                });

                assert_eq!(
                    actual,
                    value,
                    "header field {pointer:?} value mismatch.\n\
                     Expected: {expected}\n\
                     Actual:   {actual}\n\
                     Vector: {desc}",
                    pointer = pointer,
                    expected = value,
                    actual = actual,
                    desc = vector.description,
                );
            }

            Assertion::HeaderContains { text } => {
                assert!(
                    header_str.contains(text.as_str()),
                    "header JSON does not contain expected text.\n\
                     Expected: {text:?}\n\
                     Header: {header_str}\n\
                     Vector: {desc}",
                    text = text,
                    header_str = header_str,
                    desc = vector.description,
                );
            }

            Assertion::OutputContains { text } => {
                assert!(
                    output.contains(text.as_str()),
                    "output does not contain expected text.\n\
                     Expected: {text:?}\n\
                     Output: {output}\n\
                     Vector: {desc}",
                    text = text,
                    output = output,
                    desc = vector.description,
                );
            }

            Assertion::OutputNotContains { text } => {
                assert!(
                    !output.contains(text.as_str()),
                    "output should not contain: {text:?}\n\
                     Output: {output}\n\
                     Vector: {desc}",
                    text = text,
                    output = output,
                    desc = vector.description,
                );
            }

            Assertion::NoCrInFraming => {
                let cr_count = output.bytes().filter(|&b| b == b'\r').count();
                assert_eq!(
                    cr_count,
                    0,
                    "readable-view output contains {cr_count} CR (\\r) bytes; \
                     framing must use LF (0x0A) only.\n\
                     Vector: {desc}",
                    cr_count = cr_count,
                    desc = vector.description,
                );
            }
        }
    }
}

/// Run a single conformance vector.
fn run_vector(vector: &ConformanceVector) {
    let kind = parse_view_kind(&vector.view_kind);

    // Build renderer config with optional override.
    let config = if let Some(override_cfg) = &vector.config_override {
        let mut cfg = RendererConfig::default_config();
        cfg.views.insert(
            kind,
            ViewBlockConfig {
                blocks: override_cfg.blocks.clone(),
            },
        );
        cfg.validate().unwrap();
        cfg
    } else {
        let cfg = RendererConfig::default_config();
        cfg.validate().unwrap();
        cfg
    };

    match &vector.expected_failure {
        Some(expected) => {
            // Renderer should fail.
            let result = render_readable_view(&vector.input, kind, &config);
            assert!(
                result.is_err(),
                "expected renderer failure but got success.\n\
                 Vector: {desc}",
                desc = vector.description,
            );
            let err = result.unwrap_err();
            assert_eq!(
                err.id,
                expected.error_id,
                "error id mismatch.\nVector: {desc}",
                desc = vector.description,
            );
            if let Some(substr) = &expected.message_contains {
                assert!(
                    err.message.contains(substr.as_str()),
                    "error message does not contain expected substring.\n\
                     Expected: {substr:?}\n\
                     Actual:   {msg}\n\
                     Vector: {desc}",
                    substr = substr,
                    msg = err.message,
                    desc = vector.description,
                );
            }
            // For failure vectors, check assertions with the error message as output.
            check_assertions(vector, &err.message, true);
        }
        None => {
            // Renderer should succeed.
            let output = render_readable_view(&vector.input, kind, &config).unwrap_or_else(|e| {
                panic!(
                    "renderer unexpectedly failed.\n\
                         Error: {e}\n\
                         Vector: {desc}",
                    e = e,
                    desc = vector.description,
                )
            });
            check_assertions(vector, &output, false);
        }
    }
}

// ── Individual tests — one per committed fixture ─────────────────────────
//
// Each test loads its fixture at compile time via `include_str!` so the
// vector file is a committed, auditable acceptance artifact, not an ad-hoc
// in-test construction.

#[test]
fn conformance_01_no_block_outline() {
    run_vector(&load_vector!(
        "fixtures/conformance/01_no_block_outline.json"
    ));
}

#[test]
fn conformance_04_single_block() {
    run_vector(&load_vector!("fixtures/conformance/04_single_block.json"));
}

#[test]
fn conformance_07_chinese() {
    run_vector(&load_vector!("fixtures/conformance/07_chinese.json"));
}

#[test]
fn conformance_10_crlf_payload() {
    run_vector(&load_vector!("fixtures/conformance/10_crlf_payload.json"));
}

#[test]
fn conformance_11_no_trailing_newline() {
    run_vector(&load_vector!(
        "fixtures/conformance/11_no_trailing_newline.json"
    ));
}

#[test]
fn conformance_12_block_marker_in_body() {
    run_vector(&load_vector!(
        "fixtures/conformance/12_block_marker_in_body.json"
    ));
}

#[test]
fn conformance_13_warning() {
    run_vector(&load_vector!("fixtures/conformance/13_warning.json"));
}

#[test]
fn conformance_14_readable_error() {
    run_vector(&load_vector!("fixtures/conformance/14_readable_error.json"));
}

#[test]
fn conformance_15_error_guidance_array() {
    run_vector(&load_vector!(
        "fixtures/conformance/15_error_guidance_array.json"
    ));
}

#[test]
fn conformance_16_undeclared_extension_fields() {
    run_vector(&load_vector!(
        "fixtures/conformance/16_undeclared_extension_fields.json"
    ));
}

#[test]
fn conformance_17_order_independent_assertions() {
    run_vector(&load_vector!(
        "fixtures/conformance/17_order_independent_assertions.json"
    ));
}

#[test]
fn conformance_18_renderer_failure_missing_pointer() {
    run_vector(&load_vector!(
        "fixtures/conformance/18_renderer_failure_missing_pointer.json"
    ));
}

#[test]
fn conformance_19_renderer_failure_non_string() {
    run_vector(&load_vector!(
        "fixtures/conformance/19_renderer_failure_non_string.json"
    ));
}
