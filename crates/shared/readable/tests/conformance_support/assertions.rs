use docnav_readable::conformance::{Assertion, ConformanceVector};
use serde_json::Value;

use super::block_assertions::{check_block_assertion, check_no_blocks_assertion, BlockAssertion};
use super::output_blocks::{extract_header, parse_blocks, ParsedBlock};

struct AssertionContext<'a> {
    vector: &'a ConformanceVector,
    output: &'a str,
    header_str: &'a str,
    header_value: &'a Value,
    blocks: &'a [ParsedBlock],
}

/// Run all assertions for a vector against the renderer output.
pub(crate) fn check_assertions(vector: &ConformanceVector, output: &str, is_failure: bool) {
    if is_failure {
        check_failure_assertions(vector, output);
        return;
    }

    check_readable_view_assertions(vector, output);
}

fn check_failure_assertions(vector: &ConformanceVector, output: &str) {
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
}

fn check_readable_view_assertions(vector: &ConformanceVector, output: &str) {
    let header_str = extract_header(output);
    let header_value: Value =
        serde_json::from_str(header_str).expect("header JSON should be valid");
    let blocks = parse_blocks(output);
    assert_restores_source(vector, &header_value, &blocks);

    let context = AssertionContext {
        vector,
        output,
        header_str,
        header_value: &header_value,
        blocks: &blocks,
    };

    for assertion in &vector.assertions {
        check_readable_view_assertion(&context, assertion);
    }
}

fn assert_restores_source(
    vector: &ConformanceVector,
    header_value: &Value,
    blocks: &[ParsedBlock],
) {
    let mut restored = header_value.clone();

    for block in blocks {
        assert_header_block_reference(vector, header_value, block);
        restore_block_payload(vector, &mut restored, block);
    }

    assert_eq!(
        &restored,
        &vector.input,
        "readable-view header plus parsed blocks does not restore source payload.\n\
         Vector: {desc}",
        desc = vector.description,
    );
}

fn assert_header_block_reference(
    vector: &ConformanceVector,
    header_value: &Value,
    block: &ParsedBlock,
) {
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
}

fn restore_block_payload(vector: &ConformanceVector, restored: &mut Value, block: &ParsedBlock) {
    let target = restored.pointer_mut(&block.pointer).unwrap_or_else(|| {
        panic!(
            "restored header target {pointer:?} not found.\nVector: {desc}",
            pointer = block.pointer,
            desc = vector.description,
        )
    });
    *target = Value::String(block.payload.clone());
}

fn check_readable_view_assertion(context: &AssertionContext<'_>, assertion: &Assertion) {
    match assertion {
        Assertion::Block {
            pointer,
            byte_length,
            payload,
            payload_contains,
        } => check_block_assertion(
            context.vector,
            context.blocks,
            BlockAssertion {
                byte_length,
                payload,
                payload_contains,
                pointer,
            },
        ),
        Assertion::NoBlocks => {
            check_no_blocks_assertion(context.vector, context.output, context.blocks);
        }
        Assertion::HeaderField { pointer, value } => {
            check_header_field_assertion(context, pointer, value);
        }
        Assertion::HeaderContains { text } => check_header_contains_assertion(context, text),
        Assertion::OutputContains { text } => check_output_contains_assertion(context, text),
        Assertion::OutputNotContains { text } => check_output_not_contains_assertion(context, text),
        Assertion::NoCrInFraming => check_no_cr_in_framing(context),
    }
}

fn check_header_field_assertion(context: &AssertionContext<'_>, pointer: &str, value: &Value) {
    let actual = context.header_value.pointer(pointer).unwrap_or_else(|| {
        panic!(
            "header field {pointer:?} not found in header JSON.\n\
             Header: {header_str}\n\
             Vector: {desc}",
            pointer = pointer,
            header_str = context.header_str,
            desc = context.vector.description,
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
        desc = context.vector.description,
    );
}

fn check_header_contains_assertion(context: &AssertionContext<'_>, text: &str) {
    assert!(
        context.header_str.contains(text),
        "header JSON does not contain expected text.\n\
         Expected: {text:?}\n\
         Header: {header_str}\n\
         Vector: {desc}",
        text = text,
        header_str = context.header_str,
        desc = context.vector.description,
    );
}

fn check_output_contains_assertion(context: &AssertionContext<'_>, text: &str) {
    assert!(
        context.output.contains(text),
        "output does not contain expected text.\n\
         Expected: {text:?}\n\
         Output: {output}\n\
         Vector: {desc}",
        text = text,
        output = context.output,
        desc = context.vector.description,
    );
}

fn check_output_not_contains_assertion(context: &AssertionContext<'_>, text: &str) {
    assert!(
        !context.output.contains(text),
        "output should not contain: {text:?}\n\
         Output: {output}\n\
         Vector: {desc}",
        text = text,
        output = context.output,
        desc = context.vector.description,
    );
}

fn check_no_cr_in_framing(context: &AssertionContext<'_>) {
    let cr_count = context.output.bytes().filter(|&byte| byte == b'\r').count();
    assert_eq!(
        cr_count,
        0,
        "readable-view output contains {cr_count} CR (\\r) bytes; \
         framing must use LF (0x0A) only.\n\
         Vector: {desc}",
        cr_count = cr_count,
        desc = context.vector.description,
    );
}
