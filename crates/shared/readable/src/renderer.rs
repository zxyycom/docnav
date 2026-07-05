//! Readable-view renderer: JSON header + length-delimited block sections.

use serde_json::Value;

mod block;
mod framing;
#[cfg(test)]
mod tests;

use crate::error::RenderError;
use crate::renderer_config::RendererConfig;
use crate::view_kind::ReadableViewKind;

pub use block::RenderedBlock;

use block::extract_blocks;
use framing::{append_block_sections, rendered_output_capacity, LF};

/// Render a complete readable value into `readable-view` text.
///
/// The renderer clones the readable JSON value, replaces configured string
/// fields with block references in the header, and appends length-delimited
/// block sections. Callers that build custom configs should validate them
/// before rendering.
///
/// # Errors
///
/// Returns `RenderError` when:
/// - The config is missing an entry for `kind`.
/// - A configured block pointer is missing or targets a non-string field.
/// - Header JSON serialization fails.
pub fn render_readable_view(
    value: &Value,
    kind: ReadableViewKind,
    config: &RendererConfig,
) -> Result<String, RenderError> {
    let view_config = config.view_config(kind)?;
    let (header_value, blocks) = extract_blocks(value, view_config)?;
    let header_json = render_header_json(&header_value)?;

    let mut output = String::with_capacity(rendered_output_capacity(&header_json, &blocks));
    output.push_str(&header_json);
    append_block_sections(&mut output, &blocks);

    Ok(output)
}

fn render_header_json(header_value: &Value) -> Result<String, RenderError> {
    // serde_json currently emits no trailing LF; readable-view framing owns it.
    let mut header_json = serde_json::to_string_pretty(header_value)
        .map_err(RenderError::header_serialization_failed)?;
    if !header_json.ends_with('\n') {
        header_json.push(char::from(LF));
    }

    Ok(header_json)
}
