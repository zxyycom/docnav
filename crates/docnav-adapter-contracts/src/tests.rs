use super::*;
use docnav_protocol::Operation;

#[test]
fn adapter_options_keep_same_target_key_with_distinct_identity_and_owner() {
    let integer_variant = AdapterOptionSpec::builder("docnav.adapters.integer.options.shared")
        .owner("integer-adapter")
        .operations(&[Operation::Outline])
        .path(["options", "shared"])
        .process(
            "config",
            AdapterOptionProcessStrategy::json_path(["options", "shared"]),
        )
        .validation(FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(3)))
        .build();
    let string_variant = AdapterOptionSpec::builder("docnav.adapters.string.options.shared")
        .owner("string-adapter")
        .operations(&[Operation::Outline])
        .path(["options", "shared"])
        .process(
            "config",
            AdapterOptionProcessStrategy::json_path(["options", "shared"]),
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

// @case WB-CONTRACTS-NATIVE-001
#[test]
fn adapter_option_builder_wraps_typed_field_declaration_and_bindings() {
    let outline_only =
        AdapterOptionSpec::builder("docnav.adapters.markdown.options.max_heading_level")
            .owner("docnav-markdown")
            .operations(&[Operation::Outline])
            .path(["options", "max_heading_level"])
            .process(
                "cli",
                AdapterOptionProcessStrategy::cli_flag("--max-heading-level"),
            )
            .process(
                "config",
                AdapterOptionProcessStrategy::json_path(["options", "max_heading_level"]),
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
            AdapterOptionProcessStrategy::json_path(["options", "read_mode"]),
        )
        .validation(FieldValidation::string())
        .build();

    let mut builder = FieldDefSet::builder();
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

    let explicit = fields.processing_metadata(&ProcessingId::from("cli"));

    assert_eq!(explicit.len(), 1);
    assert_eq!(
        explicit[0].identity.as_str(),
        "docnav.adapters.markdown.options.max_heading_level"
    );
    assert_eq!(
        explicit[0].path.segments(),
        vec!["options", "max_heading_level"]
    );
    assert_eq!(explicit[0].value_kind, ValueKind::Integer);
    assert_eq!(explicit[0].default, DefaultMetadata::Static(3.into()));
    assert_eq!(outline_only.cli_arg_id(), Some("max-heading-level"));
    assert_eq!(
        outline_only.cli_input_path().unwrap(),
        vec!["options", "max_heading_level"]
    );
    assert_eq!(
        outline_only.processing_path("config").unwrap().unwrap(),
        vec!["options".to_owned(), "max_heading_level".to_owned()]
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
            AdapterOptionProcessStrategy::json_path(["options", "max_heading_level"]),
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
