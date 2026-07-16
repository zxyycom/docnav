use super::*;

// @case WB-MD-META-001
#[test]
fn manifest_declares_markdown_v0_identity_and_formats() {
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
    let value = serde_json::to_value(&manifest).expect("manifest JSON");
    assert!(value.get("protocol").is_none());
    assert!(value.get("recommended_parameters").is_none());
}

#[test]
fn definition_declares_metadata_handlers_native_option_and_full_read_group() {
    let definition = markdown_adapter_definition();

    assert_eq!(definition.id(), "docnav-markdown");
    assert_eq!(definition.manifest().adapter.id, "docnav-markdown");
    assert!(definition
        .operation_handlers()
        .operations()
        .contains(&Operation::Outline));
    let option = definition
        .native_options()
        .iter()
        .find(|option| {
            option.key() == "max_heading_level"
                && option.owner == "docnav-markdown"
                && option.applies_to(Operation::Outline)
                && option.applies_to(Operation::Find)
        })
        .expect("max heading level declaration");
    let cli = option
        .field_declaration()
        .expect("valid max heading level declaration")
        .processing_metadata(
            &docnav_adapter_contracts::ProcessingId::new("cli").expect("valid CLI processing id"),
        )
        .expect("valid max heading level field")
        .expect("max heading level CLI processing metadata");
    assert_eq!(cli.locator.cli_flag(), Some("--max-heading-level"));
    let cli = cli.cli.expect("max heading level CLI presentation");
    assert_eq!(
        cli.help.as_deref(),
        Some("Set the maximum Markdown heading level")
    );
    assert_eq!(cli.value_name.as_deref(), Some("value"));
    assert_eq!(cli.boolean_encoding, None);

    let full_read = definition
        .full_read_capability_group()
        .expect("full-read capability group");
    assert!(full_read.capabilities().content_hook);
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
    let arguments = InfoArguments { options: None };
    let request = make_request(
        &path,
        Operation::Info,
        OperationArguments::Info(arguments.clone()),
    );

    let info = MarkdownAdapter.info(&request, &arguments).expect("info");

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
