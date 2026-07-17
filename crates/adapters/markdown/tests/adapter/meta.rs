use super::*;

// @case WB-MD-META-001
#[test]
fn manifest_declares_markdown_v0_identity_and_formats() {
    let definition = markdown_adapter_definition();
    let manifest = definition.manifest();

    manifest.validate_semantics().expect("manifest semantics");
    assert_eq!(manifest.adapter.id, "docnav-markdown");
    assert_eq!(manifest.formats[0].id, "markdown");
    assert!(manifest.formats[0].extensions.contains(&".md".to_owned()));
    assert!(manifest.formats[0]
        .extensions
        .contains(&".markdown".to_owned()));
    assert!(manifest.formats[0]
        .content_types
        .contains(&"text/markdown".to_owned()));
    let value = serde_json::to_value(manifest).expect("manifest JSON");
    assert!(value.get("protocol").is_none());
    assert!(value.get("recommended_parameters").is_none());
}

#[test]
fn definition_declares_manifest_and_full_read_capabilities() {
    let definition = markdown_adapter_definition();

    assert_eq!(definition.id(), "docnav-markdown");
    assert_eq!(definition.manifest().adapter.id, "docnav-markdown");
    let full_read = definition
        .unstructured_full_read_capabilities()
        .expect("full-read capabilities");
    assert!(full_read.content_hook);
    assert!(full_read.has_cost_measurement_unit("lines"));
    assert!(full_read.has_cost_measurement_unit("bytes"));
    assert!(full_read.has_cost_measurement_unit("tokens"));
}

#[test]
fn probe_returns_format_evidence_without_navigation_payload() {
    let path = write_doc("sample.md", "# Title\n");
    let probe = MarkdownAdapter.probe(&path_string(&path));
    let value = serde_json::to_value(&probe).expect("probe JSON");

    assert!(probe.supported);
    assert_eq!(probe.format.as_deref(), Some("markdown"));
    assert!(probe
        .reasons
        .iter()
        .any(|reason| reason.detail.contains("Markdown")));
    assert!(value.get("entries").is_none());
    assert!(value.get("content").is_none());
}

#[test]
fn info_returns_markdown_summary() {
    let path = write_doc("info.md", "# A\nBody\n");
    let input = InfoInput {
        document_path: path_string(&path),
    };

    let info = MarkdownAdapter.info(&input).expect("info");

    assert_eq!(
        info.document
            .as_ref()
            .and_then(|document| document.content_type.as_deref()),
        Some("text/markdown")
    );
    assert_eq!(
        info.adapter
            .as_ref()
            .and_then(|adapter| adapter.format.as_deref()),
        Some("markdown")
    );
}
