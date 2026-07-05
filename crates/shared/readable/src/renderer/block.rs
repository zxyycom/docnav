use serde_json::Value;

use crate::error::RenderError;
use crate::renderer_config::ViewBlockConfig;

/// Rendered output for a single block field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedBlock {
    /// The JSON Pointer (e.g. `"/content"`).
    pub pointer: String,
    /// Number of UTF-8 bytes in the payload.
    pub byte_length: u64,
    /// The raw payload text.
    pub payload: String,
}

/// Extract block payloads from the JSON value and replace them with
/// `{"$block", "bytes"}` references.
pub(super) fn extract_blocks(
    value: &Value,
    view_config: &ViewBlockConfig,
) -> Result<(Value, Vec<RenderedBlock>), RenderError> {
    let mut header = value.clone();
    let mut blocks: Vec<RenderedBlock> = Vec::with_capacity(view_config.blocks.len());

    for pointer_str in &view_config.blocks {
        let block_content = read_block_field(&header, pointer_str)?;
        let byte_length = block_content.len() as u64;

        replace_with_block_ref(&mut header, pointer_str, byte_length)?;

        blocks.push(RenderedBlock {
            pointer: pointer_str.clone(),
            byte_length,
            payload: block_content,
        });
    }

    Ok((header, blocks))
}

/// Read a string field at `pointer` from `value`.
fn read_block_field(value: &Value, pointer: &str) -> Result<String, RenderError> {
    let field = value
        .pointer(pointer)
        .ok_or_else(|| RenderError::block_field_missing(pointer))?;

    field
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| RenderError::block_field_not_string(pointer))
}

/// Replace the value at `pointer` with a `{"$block": ..., "bytes": n}` object.
fn replace_with_block_ref(
    value: &mut Value,
    pointer: &str,
    byte_length: u64,
) -> Result<(), RenderError> {
    let target = value
        .pointer_mut(pointer)
        .ok_or_else(|| RenderError::block_field_missing(pointer))?;

    *target = Value::Object({
        let mut obj = serde_json::Map::with_capacity(2);
        obj.insert("$block".to_owned(), Value::String(pointer.to_owned()));
        obj.insert("bytes".to_owned(), Value::from(byte_length));
        obj
    });

    Ok(())
}
