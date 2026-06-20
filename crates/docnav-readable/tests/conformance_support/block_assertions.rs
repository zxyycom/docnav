use docnav_readable::conformance::ConformanceVector;

use super::output_blocks::ParsedBlock;

pub(super) struct BlockAssertion<'a> {
    pub(super) byte_length: &'a Option<u64>,
    pub(super) payload: &'a Option<String>,
    pub(super) payload_contains: &'a Option<String>,
    pub(super) pointer: &'a str,
}

pub(super) fn check_block_assertion(
    vector: &ConformanceVector,
    blocks: &[ParsedBlock],
    assertion: BlockAssertion<'_>,
) {
    let block = expected_block(vector, blocks, assertion.pointer);
    check_block_byte_length(vector, block, assertion.pointer, assertion.byte_length);
    check_block_payload(vector, block, assertion.pointer, assertion.payload);
    check_block_payload_contains(vector, block, assertion.pointer, assertion.payload_contains);
}

pub(super) fn check_no_blocks_assertion(
    vector: &ConformanceVector,
    output: &str,
    blocks: &[ParsedBlock],
) {
    assert!(
        blocks.is_empty(),
        "expected no blocks, but found {} block(s): {found:?}.\n\
         Vector: {desc}",
        blocks.len(),
        found = block_pointers(blocks),
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

fn expected_block<'a>(
    vector: &ConformanceVector,
    blocks: &'a [ParsedBlock],
    pointer: &str,
) -> &'a ParsedBlock {
    blocks
        .iter()
        .find(|block| block.pointer == pointer)
        .unwrap_or_else(|| {
            panic!(
                "expected block with pointer {pointer:?}, but not found in output.\n\
                 Found blocks: {blocks:?}\n\
                 Vector: {desc}",
                pointer = pointer,
                blocks = block_pointers(blocks),
                desc = vector.description,
            )
        })
}

fn check_block_byte_length(
    vector: &ConformanceVector,
    block: &ParsedBlock,
    pointer: &str,
    byte_length: &Option<u64>,
) {
    if let Some(expected_len) = byte_length {
        assert_eq!(
            block.byte_length,
            *expected_len,
            "block {pointer:?} byte_length mismatch: expected {expected_len}, got {actual}.\n\
             Vector: {desc}",
            pointer = pointer,
            expected_len = expected_len,
            actual = block.byte_length,
            desc = vector.description,
        );
    }
}

fn check_block_payload(
    vector: &ConformanceVector,
    block: &ParsedBlock,
    pointer: &str,
    payload: &Option<String>,
) {
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
}

fn check_block_payload_contains(
    vector: &ConformanceVector,
    block: &ParsedBlock,
    pointer: &str,
    payload_contains: &Option<String>,
) {
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

pub(super) fn block_pointers(blocks: &[ParsedBlock]) -> Vec<&String> {
    blocks.iter().map(|block| &block.pointer).collect()
}
