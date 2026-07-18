use super::*;
use serde_json::json;
use serde_json::Value;

// @case WB-TYPED-FIELDS-001
fn limit_field() -> FieldDef {
    FieldDef::builder("docnav.defaults.limit")
        .process(CONFIG_PROCESSING, config_json_path(["defaults", "limit"]))
        .validation(
            FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(100_000)),
        )
        .default_static(20_000)
        .build()
        .expect("limit field builds")
}

#[test]
fn builder_exposes_schema_metadata_and_validates_values() {
    let field = limit_field();

    let schema = field.schema_metadata();
    assert_eq!(schema.identity().as_str(), "docnav.defaults.limit");
    assert_eq!(schema.path.segments(), ["defaults", "limit"]);
    assert_eq!(schema.value_kind(), ValueKind::Integer);
    assert_eq!(schema.default(), &DefaultMetadata::Static(json!(20_000)));
    assert_eq!(
        schema.constraints().numeric_range,
        FieldNumericRange::Integer(FieldRange::between(
            FieldBound::closed(1),
            FieldBound::closed(100_000),
        ))
    );

    let value = schema
        .validate_optional_value(Some(&json!(4000)))
        .expect("valid value decodes");
    assert_eq!(value, Some(TypedValue::Integer(4000)));
}

#[test]
fn json_validation_accepts_any_json_value_including_null() {
    let field = FieldDef::builder("docnav.adapters.example.options.payload")
        .process(CONFIG_PROCESSING, config_json_path(["options", "payload"]))
        .validation(FieldValidation::json())
        .build()
        .expect("json field builds");
    let schema = field.schema_metadata();

    assert_eq!(schema.value_kind(), ValueKind::Json);
    assert_eq!(
        schema
            .validate_optional_value(Some(&json!({"mode": "wide"})))
            .expect("object JSON passes"),
        Some(TypedValue::Json(json!({"mode": "wide"})))
    );
    assert_eq!(
        schema
            .validate_optional_value(Some(&Value::Null))
            .expect("null JSON passes"),
        Some(TypedValue::Json(Value::Null))
    );
    assert_eq!(
        schema
            .validate_optional_value(None)
            .expect("absent optional JSON passes"),
        None
    );
}

#[test]
fn validation_failures_keep_field_attribution() {
    let field = limit_field();

    let schema = field.schema_metadata();
    let missing = schema.validate_optional_value(None).unwrap();
    assert_eq!(missing, None);

    let error = schema
        .validate_optional_value(Some(&json!("4000")))
        .expect_err("wrong type fails");
    assert_eq!(error.field.as_str(), "docnav.defaults.limit");
    assert_eq!(error.path.segments(), ["defaults", "limit"]);
    assert_eq!(
        error.reason,
        ValidationReason::WrongType {
            expected: ValueKind::Integer,
            actual: ActualValueKind::String
        }
    );

    let error = schema
        .validate_optional_value(Some(&json!(0)))
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
    let fields = FieldDefSet::builder()
        .field_with_declaration_path(
            ["defaults", "mode"],
            FieldDef::builder("docnav.defaults.mode")
                .process(CONFIG_PROCESSING, config_json_path(["defaults", "mode"]))
                .validation(FieldValidation::string_enum::<ExampleMode>()),
            ExpectedFieldShape::required(),
        )
        .build()
        .expect("required enum field builds");
    let json_fields = JsonFieldSet::new(&fields);

    let error = json_fields
        .validate(CONFIG_PROCESSING, &json!({"defaults": {}}))
        .expect_err("missing required field fails");
    assert_eq!(
        validation_failures(&error)[0].reason,
        ValidationReason::MissingRequired
    );

    json_fields
        .validate(CONFIG_PROCESSING, &json!({"defaults": {"mode": "compact"}}))
        .expect("allowed enum value passes");

    let error = json_fields
        .validate(
            CONFIG_PROCESSING,
            &json!({"defaults": {"mode": "unsupported"}}),
        )
        .expect_err("disallowed enum value fails");
    assert!(matches!(
        validation_failures(&error)[0].reason,
        ValidationReason::DisallowedEnumValue { .. }
    ));
}
