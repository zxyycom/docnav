mod document;
mod format;
mod options;
mod parse;
mod refs;
mod text;

pub use document::{MarkdownDocument, ResolvedRef};
pub use format::{is_markdown_extension, is_utf8_markdown_candidate};
pub use options::max_heading_level;
pub use text::cost_for;

#[cfg(test)]
mod tests;
