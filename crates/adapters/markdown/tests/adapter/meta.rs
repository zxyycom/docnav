use super::*;

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
    assert!(manifest.formats[0].filenames.is_empty());
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
fn info_returns_markdown_summary() {
    let path = write_doc("info.md", "# A\nBody\n");
    let input = InfoInput {
        document_path: path_string(&path),
    };

    let info = adapter_document(&input.document_path)
        .info(&input)
        .expect("info");

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

#[test]
fn cost_full_read_and_structured_outline_share_prepared_view() {
    let initial = "# Stable\nold body\n";
    let replacement = "# Changed\nnew body\n";
    let path = write_doc("full-read-reuse.md", initial);
    let definition = markdown_adapter_definition();
    let mut document = definition.create_document(path_string(&path));
    let request = docnav_protocol::RequestEnvelope {
        protocol_version: docnav_protocol::PROTOCOL_VERSION.to_owned(),
        request_id: "markdown-full-read-reuse".to_owned(),
        operation: docnav_protocol::Operation::Outline,
        document: docnav_protocol::Document {
            path: path_string(&path),
        },
        arguments: docnav_protocol::OperationArguments::Outline(
            docnav_protocol::OutlineArguments {
                limit: positive(6000),
                page: positive(1),
                options: None,
            },
        ),
    };

    let measured = document
        .measure_unstructured_full_read_cost(&request, &["bytes".to_owned()])
        .expect("cost should prepare the Markdown view");
    assert_eq!(measured.measurements[0].value, initial.len() as u64);

    fs::write(&path, replacement).expect("mutate document after cost preparation");
    let full_read = document
        .unstructured_full_read(&request)
        .expect("full read should reuse the cost view");
    assert_eq!(full_read.content, initial);
    let outline = document
        .outline(&outline_input(&path, 6000, 1, Some(3)))
        .expect("structured fallback should reuse the cost view")
        .into_structured()
        .expect("structured outline result");
    assert_eq!(outline.entries[0].label, "Stable");

    let mut fresh_document = definition.create_document(path_string(&path));
    let fresh = fresh_document
        .unstructured_full_read(&request)
        .expect("fresh document should observe the replacement");
    assert_eq!(fresh.content, replacement);
}
