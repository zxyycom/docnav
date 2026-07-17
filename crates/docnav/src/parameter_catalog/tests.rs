use docnav_adapter_contracts::StandardInputBinding;
use docnav_navigation::DocumentParameterBinding;
use docnav_protocol::{Operation, PagedOperation};
use docnav_typed_fields::{
    DefaultMetadata, FieldBound, FieldNumericRange, FieldRange, MergeStrategy, ProcessingId,
    ProcessingLocator, ValueKind,
};

use super::{
    document_parameter_catalog, LIMIT_IDENTITY, MAX_HEADING_LEVEL_IDENTITY, OUTPUT_IDENTITY,
    PAGE_IDENTITY, PAGINATION_ENABLED_IDENTITY,
};

#[test]
fn core_catalog_contains_exactly_the_five_closed_scalar_parameters() {
    let catalog = document_parameter_catalog().expect("core parameter catalog is valid");

    assert_eq!(
        catalog
            .entries()
            .iter()
            .map(|entry| entry.identity().as_str())
            .collect::<Vec<_>>(),
        vec![
            PAGE_IDENTITY,
            LIMIT_IDENTITY,
            PAGINATION_ENABLED_IDENTITY,
            OUTPUT_IDENTITY,
            MAX_HEADING_LEVEL_IDENTITY,
        ]
    );
    assert_entry(
        &catalog,
        PAGE_IDENTITY,
        None,
        &[
            DocumentParameterBinding::StandardInput(StandardInputBinding::OutlinePage),
            DocumentParameterBinding::StandardInput(StandardInputBinding::ReadPage),
            DocumentParameterBinding::StandardInput(StandardInputBinding::FindPage),
        ],
    );
    assert_entry(
        &catalog,
        LIMIT_IDENTITY,
        None,
        &[
            DocumentParameterBinding::StandardInput(StandardInputBinding::OutlineLimit),
            DocumentParameterBinding::StandardInput(StandardInputBinding::ReadLimit),
            DocumentParameterBinding::StandardInput(StandardInputBinding::FindLimit),
        ],
    );
    assert_entry(
        &catalog,
        PAGINATION_ENABLED_IDENTITY,
        None,
        &[
            DocumentParameterBinding::PaginationEnabled(PagedOperation::Outline),
            DocumentParameterBinding::PaginationEnabled(PagedOperation::Read),
            DocumentParameterBinding::PaginationEnabled(PagedOperation::Find),
        ],
    );
    assert_entry(
        &catalog,
        OUTPUT_IDENTITY,
        None,
        &[
            DocumentParameterBinding::OutputMode(Operation::Outline),
            DocumentParameterBinding::OutputMode(Operation::Read),
            DocumentParameterBinding::OutputMode(Operation::Find),
            DocumentParameterBinding::OutputMode(Operation::Info),
        ],
    );
    assert_entry(
        &catalog,
        MAX_HEADING_LEVEL_IDENTITY,
        Some("docnav-markdown"),
        &[
            DocumentParameterBinding::StandardInput(StandardInputBinding::OutlineMaxHeadingLevel),
            DocumentParameterBinding::StandardInput(StandardInputBinding::FindMaxHeadingLevel),
        ],
    );
}

#[test]
fn catalog_fields_preserve_current_locator_type_default_merge_and_range_facts() {
    let catalog = document_parameter_catalog().expect("core parameter catalog is valid");

    assert_field(
        &catalog,
        PAGE_IDENTITY,
        ValueKind::Integer,
        DefaultMetadata::Static(1.into()),
        "--page",
        None,
        integer_range(1, i64::from(u32::MAX)),
    );
    assert_field(
        &catalog,
        LIMIT_IDENTITY,
        ValueKind::Integer,
        DefaultMetadata::Static(6000.into()),
        "--limit",
        Some(&["defaults", "pagination", "limit"]),
        integer_range(1, i64::from(u32::MAX)),
    );
    assert_field(
        &catalog,
        PAGINATION_ENABLED_IDENTITY,
        ValueKind::Boolean,
        DefaultMetadata::Static(true.into()),
        "--pagination",
        Some(&["defaults", "pagination", "enabled"]),
        FieldNumericRange::None,
    );
    assert_field(
        &catalog,
        OUTPUT_IDENTITY,
        ValueKind::String,
        DefaultMetadata::Static("readable-view".into()),
        "--output",
        Some(&["defaults", "output"]),
        FieldNumericRange::None,
    );
    assert_field(
        &catalog,
        MAX_HEADING_LEVEL_IDENTITY,
        ValueKind::Integer,
        DefaultMetadata::Static(3.into()),
        "--max-heading-level",
        Some(&["options", "docnav-markdown", "max_heading_level"]),
        integer_range(1, 6),
    );
}

#[test]
fn catalog_fields_do_not_enable_an_env_source() {
    let catalog = document_parameter_catalog().expect("core parameter catalog is valid");
    let env = ProcessingId::new("env").expect("valid processing id");

    for entry in catalog.entries() {
        let field = catalog
            .fields()
            .field(entry.identity())
            .expect("catalog entry has a field");
        assert!(
            field.processing_metadata(&env).is_none(),
            "{} unexpectedly enables env",
            entry.identity().as_str()
        );
    }
}

#[test]
fn operation_projection_filters_only_by_closed_bindings() {
    let catalog = document_parameter_catalog().expect("core parameter catalog is valid");

    for (operation, expected) in [
        (
            Operation::Outline,
            vec![
                PAGE_IDENTITY,
                LIMIT_IDENTITY,
                PAGINATION_ENABLED_IDENTITY,
                OUTPUT_IDENTITY,
                MAX_HEADING_LEVEL_IDENTITY,
            ],
        ),
        (
            Operation::Read,
            vec![
                PAGE_IDENTITY,
                LIMIT_IDENTITY,
                PAGINATION_ENABLED_IDENTITY,
                OUTPUT_IDENTITY,
            ],
        ),
        (
            Operation::Find,
            vec![
                PAGE_IDENTITY,
                LIMIT_IDENTITY,
                PAGINATION_ENABLED_IDENTITY,
                OUTPUT_IDENTITY,
                MAX_HEADING_LEVEL_IDENTITY,
            ],
        ),
        (Operation::Info, vec![OUTPUT_IDENTITY]),
    ] {
        let fields = catalog.operation_fields(operation);
        assert_eq!(
            fields
                .map(|field| field.schema_metadata())
                .map(|metadata| metadata.identity().as_str().to_owned())
                .collect::<Vec<_>>(),
            expected
        );
    }
}

fn assert_entry(
    catalog: &docnav_navigation::DocumentParameterCatalog,
    identity: &str,
    adapter_id: Option<&str>,
    bindings: &[DocumentParameterBinding],
) {
    let identity = docnav_typed_fields::FieldIdentity::new(identity).expect("valid identity");
    let entry = catalog.entry(&identity).expect("catalog entry");
    assert_eq!(entry.adapter_id(), adapter_id);
    assert_eq!(entry.bindings(), bindings);
}

fn assert_field(
    catalog: &docnav_navigation::DocumentParameterCatalog,
    identity: &str,
    value_kind: ValueKind,
    default: DefaultMetadata,
    cli_flag: &str,
    config_path: Option<&[&str]>,
    numeric_range: FieldNumericRange,
) {
    let identity = docnav_typed_fields::FieldIdentity::new(identity).expect("valid identity");
    let field = catalog.fields().field(&identity).expect("catalog field");
    let metadata = field.schema_metadata();
    assert_eq!(metadata.value_kind(), value_kind);
    assert_eq!(metadata.default(), &default);
    assert_eq!(metadata.merge_strategy(), MergeStrategy::Replace);
    assert_eq!(&metadata.constraints().numeric_range, &numeric_range);

    let cli = field
        .processing_metadata(&ProcessingId::new("cli").expect("valid processing id"))
        .expect("CLI processing metadata");
    assert_eq!(cli.locator, ProcessingLocator::CliFlag(cli_flag.to_owned()));

    let config = field
        .processing_metadata(&ProcessingId::new("config").expect("valid processing id"))
        .and_then(|metadata| metadata.locator.config_path().cloned());
    assert_eq!(
        config.as_ref().map(|path| path.segments()),
        config_path.map(|segments| segments.to_vec())
    );
}

fn integer_range(min: i64, max: i64) -> FieldNumericRange {
    FieldNumericRange::Integer(FieldRange::between(
        FieldBound::closed(min),
        FieldBound::closed(max),
    ))
}
