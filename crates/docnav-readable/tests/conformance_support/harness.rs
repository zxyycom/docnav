use docnav_readable::conformance::ConformanceVector;
use docnav_readable::renderer::render_readable_view;
use docnav_readable::view_kind::ReadableViewKind;
use docnav_readable::{RendererConfig, ViewBlockConfig};

mod assertions;
mod block_assertions;
mod output_blocks;

use assertions::check_assertions;

/// Run a single conformance vector.
pub fn run_vector(vector: &ConformanceVector) {
    let kind = parse_view_kind(&vector.view_kind);
    let config = renderer_config_for(vector, kind);

    match &vector.expected_failure {
        Some(expected) => {
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

            check_assertions(vector, &err.message, true);
        }
        None => {
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

fn renderer_config_for(vector: &ConformanceVector, kind: ReadableViewKind) -> RendererConfig {
    let mut config = RendererConfig::default_config();

    if let Some(override_cfg) = &vector.config_override {
        config.views.insert(
            kind,
            ViewBlockConfig {
                blocks: override_cfg.blocks.clone(),
            },
        );
    }

    config.validate().unwrap();
    config
}
