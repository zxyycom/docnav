use super::*;
use serde_json::json;

// @case WB-TYPED-FIELDS-001
fn limit_chars_field() -> FieldDef {
    FieldDef::builder("docnav.defaults.limit_chars")
        .path(["defaults", "limit_chars"])
        .validation(
            FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(100_000)),
        )
        .default_static(20_000)
        .build()
        .expect("limit_chars field builds")
}

#[test]
fn builder_exposes_schema_metadata_and_validates_values() {
    let field = limit_chars_field();

    let schema = field.schema_metadata();
    assert_eq!(schema.identity.as_str(), "docnav.defaults.limit_chars");
    assert_eq!(schema.path.segments(), ["defaults", "limit_chars"]);
    assert_eq!(schema.value_kind, ValueKind::Integer);
    assert_eq!(schema.default, DefaultMetadata::Static(json!(20_000)));
    assert_eq!(
        schema.constraints.numeric_range,
        FieldNumericRange::Integer(FieldRange::between(
            FieldBound::closed(1),
            FieldBound::closed(100_000),
        ))
    );

    let value = field
        .decode_without_default(&json!({"defaults": {"limit_chars": 4000}}))
        .expect("valid value decodes");
    assert_eq!(value, Some(TypedValue::Integer(4000)));
}

#[test]
fn validation_failures_keep_field_attribution() {
    let field = limit_chars_field();

    let missing = field
        .decode_without_default(&json!({"defaults": {}}))
        .unwrap();
    assert_eq!(missing, None);

    let error = field
        .decode_without_default(&json!({"defaults": {"limit_chars": "4000"}}))
        .expect_err("wrong type fails");
    assert_eq!(error.field.as_str(), "docnav.defaults.limit_chars");
    assert_eq!(error.path.segments(), ["defaults", "limit_chars"]);
    assert_eq!(
        error.reason,
        ValidationReason::WrongType {
            expected: ValueKind::Integer,
            actual: ActualValueKind::String
        }
    );

    let error = field
        .decode_without_default(&json!({"defaults": {"limit_chars": 0}}))
        .expect_err("range violation fails");
    assert_eq!(
        error.reason,
        ValidationReason::BelowMinimum {
            minimum: FieldNumericBound::Integer(FieldBound::closed(1))
        }
    );
}

#[test]
fn required_and_enum_constraints_are_driven_by_field_declarations() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: RequiredEnumDefaults,
    }

    #[derive(Debug, FieldDefs)]
    struct RequiredEnumDefaults {
        #[field(
            FieldDef::builder("docnav.defaults.output")
                .path(["defaults", "output"])
                .validation(FieldValidation::string_enum::<OutputMode>())
        )]
        output: OutputMode,
    }

    let fields = Params::field_defs().expect("required enum field builds");

    let error = fields
        .extract_without_default(&json!({"defaults": {}}))
        .expect_err("missing required field fails");
    assert_eq!(
        error.failures()[0].reason,
        ValidationReason::MissingRequired
    );

    let params = fields
        .extract_without_default(&json!({"defaults": {"output": "readable-json"}}))
        .expect("allowed enum value passes");
    assert_eq!(params.defaults.output, OutputMode::ReadableJson);

    let error = fields
        .extract_without_default(&json!({"defaults": {"output": "xml"}}))
        .expect_err("disallowed enum value fails");
    assert!(matches!(
        error.failures()[0].reason,
        ValidationReason::DisallowedEnumValue { .. }
    ));
}
