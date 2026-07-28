use super::*;

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
