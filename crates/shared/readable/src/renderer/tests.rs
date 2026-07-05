// ── Tests ──────────────────────────────────────────────────────────────────

use super::*;
use crate::readable_value::to_readable_value;
use crate::renderer_config::RendererConfig;
use crate::test_payloads::{self, TestErrorPayload, TestReadPayload};
use serde_json::json;

// ── helpers ─────────────────────────────────────────────────────────

/// Render a test payload through the default config and return the output.
fn render_test<T: serde::Serialize>(
    payload: &T,
    kind: ReadableViewKind,
) -> Result<String, RenderError> {
    let value = to_readable_value(payload)?;
    let config = RendererConfig::default_config();
    config.validate()?;
    render_readable_view(&value, kind, &config)
}

fn assert_contains(output: &str, snippet: &str) {
    assert!(
        output.contains(snippet),
        "output missing expected snippet:\n--- output ---\n{output}\n--- expected ---\n{snippet}"
    );
}

fn assert_not_contains(output: &str, snippet: &str) {
    assert!(
        !output.contains(snippet),
        "output should not contain:\n{snippet}\n--- output ---\n{output}"
    );
}

mod errors;
mod success;
