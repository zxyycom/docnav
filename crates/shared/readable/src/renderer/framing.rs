use super::RenderedBlock;

/// Platform-independent LF byte used in all readable-view framing.
pub(super) const LF: u8 = 0x0A;

const BLOCK_START_PREFIX: &str = "[block ";
const BLOCK_BYTES_PREFIX: &str = " bytes=";
const BLOCK_END_PREFIX: &str = "[endblock ";
const BLOCK_MARKER_SUFFIX: &str = "]\n";
const MAX_FRAMING_LF_LEN: usize = 1;

pub(super) fn rendered_output_capacity(header_json: &str, blocks: &[RenderedBlock]) -> usize {
    let separator_capacity = usize::from(!blocks.is_empty());
    header_json.len() + separator_capacity + block_sections_capacity(blocks)
}

pub(super) fn append_block_sections(output: &mut String, blocks: &[RenderedBlock]) {
    if !blocks.is_empty() {
        output.push(char::from(LF));
        for block in blocks {
            emit_block_section(output, block);
        }
    }
}

/// Append a block section to `output`.
///
/// Format:
/// ```text
/// [block <pointer> bytes=<n>]\n
/// <payload>[framing LF if payload doesn't end with LF]
/// [endblock <pointer>]\n
/// ```
fn emit_block_section(output: &mut String, block: &RenderedBlock) {
    output.push_str(BLOCK_START_PREFIX);
    output.push_str(&block.pointer);
    output.push_str(BLOCK_BYTES_PREFIX);
    output.push_str(&block.byte_length.to_string());
    output.push_str(BLOCK_MARKER_SUFFIX);

    output.push_str(&block.payload);

    // Framing LF: if payload does not already end with LF, add one
    // so the end marker always starts on its own line.
    if !block.payload.ends_with('\n') {
        output.push(char::from(LF));
    }

    output.push_str(BLOCK_END_PREFIX);
    output.push_str(&block.pointer);
    output.push_str(BLOCK_MARKER_SUFFIX);
}

fn block_sections_capacity(blocks: &[RenderedBlock]) -> usize {
    blocks
        .iter()
        .map(|b| {
            BLOCK_START_PREFIX.len()
                + b.pointer.len()
                + BLOCK_BYTES_PREFIX.len()
                + b.byte_length.to_string().len()
                + BLOCK_MARKER_SUFFIX.len()
                + b.payload.len()
                + MAX_FRAMING_LF_LEN
                + BLOCK_END_PREFIX.len()
                + b.pointer.len()
                + BLOCK_MARKER_SUFFIX.len()
        })
        .sum()
}
