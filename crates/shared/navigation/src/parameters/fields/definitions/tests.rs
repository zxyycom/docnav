use docnav_typed_fields::{
    ExpectedFieldShape, FieldDefBuilder, FieldDefSet, ProcessingId, ProcessingLocator,
};

use super::adapter_id_field;

const CLI_PROCESSING: &str = "cli";
const CONFIG_PROCESSING: &str = "config";
const DIRECT_PROCESSING: &str = "direct";

#[test]
fn common_named_fields_author_cli_processing_metadata() {
    let fields = fields(adapter_id_field(DIRECT_PROCESSING, CONFIG_PROCESSING));
    let metadata = fields
        .processing_metadata(&ProcessingId::new(CLI_PROCESSING).expect("CLI processing id"))
        .into_iter()
        .next()
        .expect("common field CLI metadata");
    assert_eq!(
        metadata.locator,
        ProcessingLocator::CliFlag("--adapter".to_owned())
    );
    let cli = metadata.cli.expect("adapter field CLI presentation");
    assert_eq!(
        cli.help.as_deref(),
        Some("Select the adapter for this document")
    );
    assert_eq!(cli.value_name.as_deref(), Some("adapter-id"));
    assert_eq!(cli.boolean_encoding, None);
}

fn fields<T: 'static>(builder: FieldDefBuilder<T>) -> FieldDefSet {
    FieldDefSet::builder()
        .field(builder, ExpectedFieldShape::optional())
        .build()
        .expect("common field declaration")
}
