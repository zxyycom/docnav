use docnav_adapter_contracts::StandardInputBinding;
use docnav_navigation::DocumentParameterBinding;
use docnav_protocol::{Operation, PagedOperation};
use docnav_typed_fields::{
    DefaultMetadata, FieldBound, FieldNumericRange, FieldRange, MergeStrategy, ProcessingId,
    ProcessingLocator, ValueKind,
};

use super::{
    document_parameter_catalog, AUTO_READ_IDENTITY, LIMIT_IDENTITY, MAX_HEADING_LEVEL_IDENTITY,
    OUTPUT_IDENTITY, PAGE_IDENTITY, PAGINATION_ENABLED_IDENTITY,
};

#[test]
fn core_catalog_contains_the_auto_read_orchestration_parameter() {
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
            AUTO_READ_IDENTITY,
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
        AUTO_READ_IDENTITY,
        None,
        &[
            DocumentParameterBinding::AutoReadMode(Operation::Outline),
            DocumentParameterBinding::AutoReadMode(Operation::Find),
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
        ExpectedField {
            value_kind: ValueKind::Integer,
            default: DefaultMetadata::Static(1.into()),
            cli_flag: "--page",
            config_path: None,
            numeric_range: integer_range(1, i64::from(u32::MAX)),
        },
    );
    assert_field(
        &catalog,
        LIMIT_IDENTITY,
        ExpectedField {
            value_kind: ValueKind::Integer,
            default: DefaultMetadata::Static(6000.into()),
            cli_flag: "--limit",
            config_path: Some(&["defaults", "pagination", "limit"]),
            numeric_range: integer_range(1, i64::from(u32::MAX)),
        },
    );
    assert_field(
        &catalog,
        PAGINATION_ENABLED_IDENTITY,
        ExpectedField {
            value_kind: ValueKind::Boolean,
            default: DefaultMetadata::Static(true.into()),
            cli_flag: "--pagination",
            config_path: Some(&["defaults", "pagination", "enabled"]),
            numeric_range: FieldNumericRange::None,
        },
    );
    assert_field(
        &catalog,
        OUTPUT_IDENTITY,
        ExpectedField {
            value_kind: ValueKind::String,
            default: DefaultMetadata::Static("readable-view".into()),
            cli_flag: "--output",
            config_path: Some(&["defaults", "output"]),
            numeric_range: FieldNumericRange::None,
        },
    );
    assert_field(
        &catalog,
        AUTO_READ_IDENTITY,
        ExpectedField {
            value_kind: ValueKind::String,
            default: DefaultMetadata::Static("unique-ref".into()),
            cli_flag: "--auto-read",
            config_path: Some(&["defaults", "auto_read"]),
            numeric_range: FieldNumericRange::None,
        },
    );
    let auto_read = catalog
        .fields()
        .field(&docnav_typed_fields::FieldIdentity::new(AUTO_READ_IDENTITY).unwrap())
        .expect("auto-read field");
    let accepted_values = [
        serde_json::Value::String("disabled".to_owned()),
        serde_json::Value::String("unique-ref".to_owned()),
    ];
    assert_eq!(
        auto_read.constraints().enum_values.as_deref(),
        Some(accepted_values.as_slice())
    );
    assert_field(
        &catalog,
        MAX_HEADING_LEVEL_IDENTITY,
        ExpectedField {
            value_kind: ValueKind::Integer,
            default: DefaultMetadata::Static(3.into()),
            cli_flag: "--max-heading-level",
            config_path: Some(&["options", "docnav-markdown", "max_heading_level"]),
            numeric_range: integer_range(1, 6),
        },
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
                AUTO_READ_IDENTITY,
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
                AUTO_READ_IDENTITY,
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

struct ExpectedField<'a> {
    value_kind: ValueKind,
    default: DefaultMetadata,
    cli_flag: &'a str,
    config_path: Option<&'a [&'a str]>,
    numeric_range: FieldNumericRange,
}

fn assert_field(
    catalog: &docnav_navigation::DocumentParameterCatalog,
    identity: &str,
    expected: ExpectedField<'_>,
) {
    let identity = docnav_typed_fields::FieldIdentity::new(identity).expect("valid identity");
    let field = catalog.fields().field(&identity).expect("catalog field");
    let metadata = field.schema_metadata();
    assert_eq!(metadata.value_kind(), expected.value_kind);
    assert_eq!(metadata.default(), &expected.default);
    assert_eq!(metadata.merge_strategy(), MergeStrategy::Replace);
    assert_eq!(
        &metadata.constraints().numeric_range,
        &expected.numeric_range
    );

    let cli = field
        .processing_metadata(&ProcessingId::new("cli").expect("valid processing id"))
        .expect("CLI processing metadata");
    assert_eq!(
        cli.locator,
        ProcessingLocator::CliFlag(expected.cli_flag.to_owned())
    );

    let config = field
        .processing_metadata(&ProcessingId::new("config").expect("valid processing id"))
        .and_then(|metadata| metadata.locator.config_path().cloned());
    assert_eq!(
        config.as_ref().map(|path| path.segments()),
        expected.config_path.map(|segments| segments.to_vec())
    );
}

fn integer_range(min: i64, max: i64) -> FieldNumericRange {
    FieldNumericRange::Integer(FieldRange::between(
        FieldBound::closed(min),
        FieldBound::closed(max),
    ))
}
