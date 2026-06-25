use super::*;
use serde_json::json;

#[test]
fn derived_field_defs_extract_json_into_typed_params_object() {
    fn consume_params(params: DocnavParams) -> (Option<i64>, OutputMode) {
        (params.defaults.limit_chars, params.defaults.output)
    }
    fn consume_defaults(params: DocnavParamsDefaultValues) -> (Option<i64>, Option<OutputMode>) {
        (params.defaults.limit_chars, params.defaults.output)
    }

    let fields = DocnavParams::field_defs().expect("definition set builds");

    assert_eq!(
        fields.value_kinds().get("docnav.defaults.limit_chars"),
        Some(&ValueKind::Integer)
    );
    assert_eq!(
        consume_defaults(fields.default_values()),
        (Some(20_000), Some(OutputMode::ReadableView))
    );
    assert_eq!(fields.schema_metadata().len(), 2);
    let output_schema = fields
        .schema_metadata()
        .into_iter()
        .find(|metadata| metadata.identity.as_str() == "docnav.defaults.output")
        .expect("output metadata exists");
    assert_eq!(
        output_schema.constraints.enum_values,
        Some(vec![
            json!("readable-view"),
            json!("readable-json"),
            json!("protocol-json")
        ])
    );

    let input = json!({"a": {"b": 4000}, "defaults": {"output": "readable-json"}});
    let params = fields
        .extract_without_default(&input)
        .expect("valid input extracts params");
    assert_eq!(
        consume_params(params),
        (Some(4000), OutputMode::ReadableJson)
    );

    let input = json!({"defaults": {"output": "protocol-json"}});
    let params = fields
        .extract_without_default(&input)
        .expect("missing optional field extracts");
    assert_eq!(params.defaults.limit_chars, None);
    assert_eq!(params.defaults.output, OutputMode::ProtocolJson);

    let input = json!({"defaults": {"output": "protocol-json"}});
    let params = fields
        .extract_with_static_defaults(&input)
        .expect("missing value falls back to static default");
    assert_eq!(
        consume_params(params),
        (Some(20_000), OutputMode::ProtocolJson)
    );

    fields
        .validate_with_static_defaults(&json!({}))
        .expect("static defaults satisfy missing required fields");
}

#[test]
fn built_field_defs_can_return_typed_builder_for_static_reuse() {
    let fields = DocnavParams::field_defs().expect("definition set builds");

    let mut fields2 = fields.to_builder();
    fields2.defaults.limit_chars = fields2
        .defaults
        .limit_chars
        .path(["defaults", "limit_chars"]);
    let fields2 = fields2.build().expect("updated definition set builds");

    let original = fields
        .extract_without_default(&json!({
            "a": {"b": 111},
            "defaults": {"limit_chars": 222, "output": "readable-view"}
        }))
        .expect("original fields still use original path");
    assert_eq!(original.defaults.limit_chars, Some(111));

    let reused = fields2
        .extract_without_default(&json!({
            "a": {"b": 111},
            "defaults": {"limit_chars": 222, "output": "readable-view"}
        }))
        .expect("rebuilt fields use updated path");
    assert_eq!(reused.defaults.limit_chars, Some(222));
}

#[test]
fn single_derive_extracts_typed_params_object_without_default() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: Defaults,
    }

    #[derive(Debug, FieldDefs)]
    struct Defaults {
        #[field(
            FieldDef::builder("docnav.defaults.limit_chars")
                .path(["defaults", "limit_chars"])
                .validation(
                    FieldValidation::int()
                        .between(FieldBound::closed(1), FieldBound::closed(100_000)),
                )
                .default_static(20_000)
        )]
        limit_chars: Option<i64>,
    }

    let fields = Params::field_defs().expect("definition set builds");

    assert_eq!(fields.default_values().defaults.limit_chars, Some(20_000));
    let params = fields
        .extract_without_default(&json!({"defaults": {"limit_chars": 4000}}))
        .expect("valid input extracts params");
    assert_eq!(params.defaults.limit_chars, Some(4000));

    let params = fields
        .extract_with_static_defaults(&json!({"defaults": {}}))
        .expect("missing input uses static default");
    assert_eq!(params.defaults.limit_chars, Some(20_000));
}

#[test]
fn extract_without_default_and_validate_without_default_share_validation_errors() {
    let fields = DocnavParams::field_defs().expect("definition set builds");
    let input = json!({"a": {"b": "4000"}, "defaults": {"output": "xml"}});

    let extract_error = fields
        .extract_without_default(&input)
        .expect_err("extract_without_default fails");
    let validate_error = fields
        .validate_without_default(&input)
        .expect_err("validate_without_default fails");

    assert_eq!(extract_error, validate_error);
    assert_eq!(extract_error.failures().len(), 2);
    assert!(extract_error.failures().iter().any(|failure| {
        matches!(
            failure.reason,
            ValidationReason::WrongType {
                expected: ValueKind::Integer,
                actual: ActualValueKind::String
            }
        )
    }));
    assert!(extract_error
        .failures()
        .iter()
        .any(|failure| { matches!(failure.reason, ValidationReason::DisallowedEnumValue { .. }) }));
}

#[test]
fn raw_identifier_declaration_uses_json_field_name_for_defaults() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: Defaults,
    }

    #[derive(Debug, FieldDefs)]
    struct Defaults {
        #[field(
            FieldDef::builder("docnav.defaults.type")
                .path(["defaults", "type"])
                .validation(FieldValidation::string())
                .default_static("markdown")
        )]
        r#type: Option<String>,
    }

    let fields = Params::field_defs().expect("definition set builds");

    assert_eq!(
        fields.default_values().defaults.r#type,
        Some("markdown".to_string())
    );

    let params = fields
        .extract_without_default(&json!({"defaults": {"type": "plain"}}))
        .expect("raw identifier field extracts");
    assert_eq!(params.defaults.r#type, Some("plain".to_string()));
}

#[test]
fn typed_default_values_use_none_for_fields_without_static_default() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: Defaults,
    }

    #[derive(Debug, FieldDefs)]
    struct Defaults {
        #[field(
            FieldDef::builder("docnav.defaults.limit_chars")
                .path(["defaults", "limit_chars"])
                .validation(FieldValidation::int())
        )]
        limit_chars: Option<i64>,
    }

    let fields = Params::field_defs().expect("definition set builds");

    assert_eq!(fields.default_values().defaults.limit_chars, None);
}

#[test]
fn extract_with_static_defaults_keeps_required_failures_without_default() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: Defaults,
    }

    #[derive(Debug, FieldDefs)]
    struct Defaults {
        #[field(
            FieldDef::builder("docnav.defaults.output")
                .path(["defaults", "output"])
                .validation(FieldValidation::string())
        )]
        output: String,
    }

    let fields = Params::field_defs().expect("definition set builds");

    let error = fields
        .extract_with_static_defaults(&json!({"defaults": {}}))
        .expect_err("missing required value without default still fails");

    assert_eq!(
        error.failures()[0].reason,
        ValidationReason::MissingRequired
    );
}
