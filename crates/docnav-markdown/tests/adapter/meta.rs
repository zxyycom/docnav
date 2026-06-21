use super::*;

// @case WB-MD-META-001
#[test]
fn manifest_declares_markdown_v0_capabilities() {
    let manifest = MarkdownAdapter.manifest();

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
    assert_eq!(
        manifest.capabilities,
        vec![
            Operation::Outline,
            Operation::Read,
            Operation::Find,
            Operation::Info
        ]
    );

    let value = serde_json::to_value(&manifest).expect("manifest JSON");
    assert!(value.get("protocol").is_none());
    assert!(value.get("recommended_parameters").is_none());
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
fn info_returns_markdown_summary_and_capabilities() {
    let path = write_doc("info.md", "# A\nBody\n");
    let arguments = InfoArguments { options: None };
    let request = make_request(
        &path,
        Operation::Info,
        OperationArguments::Info(arguments.clone()),
    );

    let info = MarkdownAdapter.info(&request, &arguments).expect("info");

    assert!(info.display.contains("text/markdown"));
    assert_eq!(
        info.capabilities,
        vec![
            Operation::Outline,
            Operation::Read,
            Operation::Find,
            Operation::Info
        ]
    );
}
