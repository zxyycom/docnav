mod document;
mod options;
mod parse;
mod refs;
mod text;

pub use document::{MarkdownDocument, ResolvedRef};
pub use options::max_heading_level;
pub use text::cost_for;

#[cfg(test)]
mod tests;
