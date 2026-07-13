use crate::{
    AdapterOptionProcessStrategy, AdapterOptionSpec, DefaultMetadata, ExpectedFieldShape,
    FieldBound, FieldDefSet, FieldValidation, ProcessStrategy, ProcessingId, ValueKind,
};
use docnav_protocol::Operation;
use docnav_typed_fields::{FieldDef, ProcessingInputKind, ProcessingLocator};

#[test]
fn adapter_options_keep_same_target_key_with_distinct_identity_and_owner() {
    let integer_variant = AdapterOptionSpec::builder("docnav.adapters.integer.options.shared")
        .owner("integer-adapter")
        .operations(&[Operation::Outline])
        .path(["options", "shared"])
        .process(
            "config",
            AdapterOptionProcessStrategy::config_path(["options", "integer-adapter", "shared"]),
        )
        .validation(FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(3)))
        .build();
    let string_variant = AdapterOptionSpec::builder("docnav.adapters.string.options.shared")
        .owner("string-adapter")
        .operations(&[Operation::Outline])
        .path(["options", "shared"])
        .process(
            "config",
            AdapterOptionProcessStrategy::config_path(["options", "string-adapter", "shared"]),
        )
        .validation(FieldValidation::string())
        .build();

    assert_eq!(integer_variant.key(), string_variant.key());
    assert_ne!(integer_variant.identity, string_variant.identity);
    assert_ne!(integer_variant.owner, string_variant.owner);
    assert_eq!(integer_variant.value_kind(), ValueKind::Integer);
    assert_eq!(string_variant.value_kind(), ValueKind::String);
    assert_eq!(integer_variant.namespace(), string_variant.namespace());
    assert_ne!(integer_variant.value_kind(), string_variant.value_kind());
}

#[test]
fn adapter_option_builder_wraps_typed_field_declaration_and_bindings() {
    let outline_only =
        AdapterOptionSpec::builder("docnav.adapters.markdown.options.max_heading_level")
            .owner("docnav-markdown")
            .operations(&[Operation::Outline])
            .path(["options", "max_heading_level"])
            .process("cli", ProcessStrategy::cli_flag("--max-heading-level"))
            .process(
                "config",
                ProcessStrategy::config_path(["options", "docnav-markdown", "max_heading_level"]),
            )
            .validation(
                FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(6)),
            )
            .default_static(3)
            .build();
    let read_only = AdapterOptionSpec::builder("docnav.adapters.markdown.options.read_mode")
        .owner("docnav-markdown")
        .operations(&[Operation::Read])
        .path(["options", "read_mode"])
        .process(
            "config",
            AdapterOptionProcessStrategy::config_path(["options", "docnav-markdown", "read_mode"]),
        )
        .validation(FieldValidation::string())
        .build();

    let mut builder = FieldDefSet::builder().field(
        FieldDef::builder("docnav.defaults.pagination.limit")
            .process("direct", ProcessStrategy::json_path(["limit"]))
            .validation(FieldValidation::int()),
        ExpectedFieldShape::required(),
    );
    for option in [&outline_only, &read_only]
        .into_iter()
        .filter(|option| option.applies_to(Operation::Outline))
    {
        builder = builder.field_declaration(
            option
                .field_declaration()
                .expect("valid adapter option declaration"),
        );
    }
    let fields = builder.build().expect("adapter option field defs");

    let explicit =
        fields.processing_metadata(&ProcessingId::new("cli").expect("valid processing id"));
    let config =
        fields.processing_metadata(&ProcessingId::new("config").expect("valid processing id"));
    let direct =
        fields.processing_metadata(&ProcessingId::new("direct").expect("valid processing id"));

    assert_eq!(direct.len(), 1);
    assert_eq!(direct[0].input_kind, ProcessingInputKind::JsonValue);
    assert_eq!(explicit.len(), 1);
    assert_eq!(explicit[0].input_kind, ProcessingInputKind::CliArguments);
    assert_eq!(
        explicit[0].identity.as_str(),
        "docnav.adapters.markdown.options.max_heading_level"
    );
    assert_eq!(
        explicit[0].locator,
        ProcessingLocator::CliFlag("--max-heading-level".to_owned())
    );
    assert_eq!(explicit[0].value_kind, ValueKind::Integer);
    assert_eq!(explicit[0].default, DefaultMetadata::Static(3.into()));
    assert_eq!(config.len(), 1);
    assert!(matches!(
        &config[0].locator,
        ProcessingLocator::ConfigPath(path)
            if path.segments()
                == vec!["options", "docnav-markdown", "max_heading_level"]
    ));
    assert_eq!(
        outline_only.cli_arg_id(),
        Some("max-heading-level".to_owned())
    );
    assert_eq!(outline_only.processing_path("cli").unwrap(), None);
    assert_eq!(
        outline_only.cli_input_path().unwrap(),
        vec!["options", "max_heading_level"]
    );
    assert_eq!(
        outline_only.processing_path("config").unwrap().unwrap(),
        vec![
            "options".to_owned(),
            "docnav-markdown".to_owned(),
            "max_heading_level".to_owned(),
        ]
    );
    assert_eq!(
        outline_only.expected_value_description(),
        "integer in range 1..6"
    );
}

#[test]
fn adapter_option_field_declaration_rejects_invalid_path() {
    let invalid = AdapterOptionSpec::builder("docnav.adapters.markdown.options.max_heading_level")
        .owner("docnav-markdown")
        .operations(&[Operation::Outline])
        .path(["markdown", "max_heading_level"])
        .process(
            "config",
            AdapterOptionProcessStrategy::config_path([
                "options",
                "docnav-markdown",
                "max_heading_level",
            ]),
        )
        .validation(FieldValidation::int())
        .build();

    let error = invalid
        .field_declaration()
        .expect_err("invalid declaration path");

    assert_eq!(
        error.to_string(),
        "adapter option docnav.adapters.markdown.options.max_heading_level declaration path must be options.<key>, got markdown.max_heading_level"
    );
}
