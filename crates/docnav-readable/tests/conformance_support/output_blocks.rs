/// A parsed block section extracted from readable-view output.
#[derive(Debug)]
pub(crate) struct ParsedBlock {
    pub(crate) pointer: String,
    pub(crate) byte_length: u64,
    pub(crate) payload: String,
}

/// Parse all `[block ...]...[endblock ...]` sections from output.
pub(crate) fn parse_blocks(output: &str) -> Vec<ParsedBlock> {
    let mut blocks = Vec::new();
    let bytes = output.as_bytes();
    let Some((_header_end, mut cursor)) = first_block_boundary(bytes) else {
        return blocks;
    };

    while let Some(start_rel) = find_bytes(&bytes[cursor..], b"[block ") {
        let start = cursor + start_rel;
        let (block, next_cursor) = parse_block_at(bytes, start);
        blocks.push(block);
        cursor = next_cursor;
    }

    blocks
}

/// Extract the header JSON portion from output (everything before the first
/// block boundary, trimmed).
pub(crate) fn extract_header(output: &str) -> &str {
    let header_end = first_block_boundary(output.as_bytes())
        .map(|(header_end, _block_start)| header_end)
        .unwrap_or(output.len());
    output[..header_end].trim_end()
}

fn parse_block_at(bytes: &[u8], start: usize) -> (ParsedBlock, usize) {
    let start_line_end = find_byte(&bytes[start..], b'\n')
        .map(|pos| start + pos)
        .expect("block start marker missing LF");
    let start_line = std::str::from_utf8(&bytes[start..start_line_end])
        .expect("block start marker should be UTF-8");
    let (pointer, byte_length) = parse_start_marker(start_line);

    let payload = payload_slice(bytes, start_line_end + 1, &pointer, byte_length);
    let payload_start = start_line_end + 1;
    let payload_end = payload_start + payload.len();
    let marker_start = end_marker_start(bytes, payload_start, payload_end, &pointer);
    let next_cursor = assert_end_marker(bytes, marker_start, &pointer);

    (
        ParsedBlock {
            pointer,
            byte_length,
            payload: std::str::from_utf8(payload)
                .expect("block payload should be valid UTF-8")
                .to_owned(),
        },
        next_cursor,
    )
}

fn parse_start_marker(start_line: &str) -> (String, u64) {
    let inner = start_line
        .strip_prefix("[block ")
        .and_then(|s| s.strip_suffix(']'))
        .expect("malformed block start marker");
    let (pointer, bytes_part) = inner
        .rsplit_once(" bytes=")
        .expect("block start marker missing 'bytes='");
    let byte_length = bytes_part.parse().expect("block bytes value not a number");

    (pointer.to_owned(), byte_length)
}

fn payload_slice<'a>(
    bytes: &'a [u8],
    payload_start: usize,
    pointer: &str,
    byte_length: u64,
) -> &'a [u8] {
    let byte_length_usize = usize::try_from(byte_length).expect("block bytes value exceeds usize");
    let payload_end = payload_start
        .checked_add(byte_length_usize)
        .expect("block payload byte range overflow");
    assert!(
        payload_end <= bytes.len(),
        "block {pointer:?} declares {byte_length} bytes, beyond output length"
    );

    &bytes[payload_start..payload_end]
}

fn end_marker_start(
    bytes: &[u8],
    payload_start: usize,
    payload_end: usize,
    pointer: &str,
) -> usize {
    let mut marker_start = payload_end;

    if !bytes[payload_start..payload_end].ends_with(b"\n") {
        assert!(
            bytes.get(marker_start) == Some(&b'\n'),
            "block {pointer:?} missing framing LF before end marker"
        );
        marker_start += 1;
    }

    marker_start
}

fn assert_end_marker(bytes: &[u8], marker_start: usize, pointer: &str) -> usize {
    let end_marker = format!("[endblock {pointer}]\n");
    let end_marker_bytes = end_marker.as_bytes();
    assert!(
        bytes[marker_start..].starts_with(end_marker_bytes),
        "block {pointer:?} missing end marker after declared payload bytes"
    );

    marker_start + end_marker_bytes.len()
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
