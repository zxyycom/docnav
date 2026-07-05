use super::*;

// @case WB-READABLE-RENDERER-002
// ── 1.7.1 Config validation: pointer missing ──────────────────────

#[test]
fn pointer_missing_from_value_fails() {
    let value = json!({"not_content": "x"});

    let config = RendererConfig::default_config();
    config.validate().unwrap();

    let err = render_readable_view(&value, ReadableViewKind::Read, &config).unwrap_err();
    assert_eq!(err.id, RenderError::ERROR_ID);
    assert!(
        err.message.contains("/content"),
        "error should mention missing pointer"
    );
}

// ── 1.7.2 Config validation: non-string target ────────────────────

#[test]
fn non_string_target_fails() {
    let value = json!({"content": 42, "content_type": "text/plain"});

    let config = RendererConfig::default_config();
    config.validate().unwrap();

    let err = render_readable_view(&value, ReadableViewKind::Read, &config).unwrap_err();
    assert_eq!(err.id, RenderError::ERROR_ID);
    assert!(
        err.message.contains("not resolve to a string"),
        "error should mention non-string target"
    );
}

// ── 1.7.3 Config validation: duplicate pointer ────────────────────

#[test]
fn duplicate_pointer_in_config_fails() {
    let mut custom_config = RendererConfig::default_config();
    custom_config.views.insert(
        ReadableViewKind::Read,
        crate::renderer_config::ViewBlockConfig {
            blocks: vec!["/content".to_owned(), "/content".to_owned()],
        },
    );

    let err = custom_config.validate().unwrap_err();
    assert_eq!(err.id, RenderError::ERROR_ID);
    assert!(
        err.message.contains("duplicate block pointer"),
        "error should mention duplicate"
    );
}

// ── 1.7.4 Config validation: pointer syntax ───────────────────────

#[test]
fn pointer_without_leading_slash_fails_config_validation() {
    let mut custom_config = RendererConfig::default_config();
    custom_config.views.insert(
        ReadableViewKind::Read,
        crate::renderer_config::ViewBlockConfig {
            blocks: vec!["content".to_owned()], // missing leading /
        },
    );

    let err = custom_config.validate().unwrap_err();
    assert_eq!(err.id, RenderError::ERROR_ID);
    assert!(err.message.contains("must start with '/'"));
}

// ── 1.7.5 Renderer error id is stable ─────────────────────────────

#[test]
fn render_error_uses_stable_id() {
    let value = json!({"wrong": "shape"});
    let config = RendererConfig::default_config();
    config.validate().unwrap();

    let err = render_readable_view(&value, ReadableViewKind::Read, &config).unwrap_err();
    assert_eq!(err.id, "readable_view_render_failed");
}
