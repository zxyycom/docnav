use docnav_typed_fields::{
    CliBooleanEncoding, DefaultMetadata, ExpectedFieldShape, FieldDefBuilder, FieldDefSet,
    ProcessingId, ProcessingLocator, ProcessingMetadataView,
};
use serde_json::json;

use crate::NavigationOutputMode;

use super::{
    adapter_id_field, configurable_limit_field, configurable_output_field,
    pagination_enabled_field, standard_page_field,
};

const CLI_PROCESSING: &str = "cli";
const CONFIG_PROCESSING: &str = "config";
const DIRECT_PROCESSING: &str = "direct";

#[test]
fn common_named_fields_author_cli_processing_metadata() {
    let cases = [
        (
            metadata(adapter_id_field(DIRECT_PROCESSING, CONFIG_PROCESSING)),
            "--adapter",
            "Select the adapter for this document",
            "adapter-id",
            None,
        ),
        (
            metadata(standard_page_field(DIRECT_PROCESSING)),
            "--page",
            "Select the result page",
            "positive integer",
            None,
        ),
        (
            metadata(configurable_limit_field(
                DIRECT_PROCESSING,
                CONFIG_PROCESSING,
            )),
            "--limit",
            "Set the result page size",
            "positive integer",
            None,
        ),
        (
            metadata(pagination_enabled_field(
                DIRECT_PROCESSING,
                CONFIG_PROCESSING,
            )),
            "--pagination",
            "Enable or disable pagination",
            "enabled|disabled",
            Some(CliBooleanEncoding::explicit("enabled", "disabled")),
        ),
    ];

    for (metadata, flag, help, value_name, boolean_encoding) in cases {
        assert_eq!(
            metadata.locator,
            ProcessingLocator::CliFlag(flag.to_owned())
        );
        let cli = metadata.cli.expect("common field CLI presentation");
        assert_eq!(cli.help.as_deref(), Some(help));
        assert_eq!(cli.value_name.as_deref(), Some(value_name));
        assert_eq!(cli.boolean_encoding, boolean_encoding);
    }
}

#[test]
fn output_cli_metadata_uses_canonical_accepted_and_default_facts() {
    let metadata = metadata(
        configurable_output_field::<NavigationOutputMode>(DIRECT_PROCESSING, CONFIG_PROCESSING)
            .default_static(NavigationOutputMode::ReadableView),
    );
    let cli = metadata.cli.expect("output CLI metadata");

    assert_eq!(
        metadata.locator,
        ProcessingLocator::CliFlag("--output".to_owned())
    );
    assert_eq!(cli.help.as_deref(), Some("Select the document output mode"));
    assert_eq!(cli.value_name.as_deref(), Some("mode"));
    assert_eq!(cli.boolean_encoding, None);
    assert_eq!(
        metadata.constraints.enum_values,
        Some(vec![
            json!("readable-view"),
            json!("readable-json"),
            json!("protocol-json"),
        ])
    );
    assert_eq!(
        metadata.default,
        DefaultMetadata::Static(json!("readable-view"))
    );
    for duplicated_fact in ["readable-view", "readable-json", "protocol-json"] {
        assert!(!cli
            .help
            .as_deref()
            .is_some_and(|help| help.contains(duplicated_fact)));
        assert!(!cli
            .value_name
            .as_deref()
            .is_some_and(|value_name| value_name.contains(duplicated_fact)));
    }
}

fn metadata<T: 'static>(builder: FieldDefBuilder<T>) -> ProcessingMetadataView {
    let fields = FieldDefSet::builder()
        .field(builder, ExpectedFieldShape::optional())
        .build()
        .expect("common field declaration");
    fields
        .processing_metadata(&ProcessingId::new(CLI_PROCESSING).expect("CLI processing id"))
        .into_iter()
        .next()
        .expect("common field CLI metadata")
}
