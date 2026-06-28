use super::*;
use serde_json::json;

#[derive(Debug, FieldDefs)]
struct DocnavParams {
    #[field(group)]
    defaults: DefaultsParams,
}

#[derive(Debug, FieldDefs)]
struct DefaultsParams {
    #[field(
        FieldDef::builder("docnav.defaults.limit_chars")
            .process(CONFIG_PROCESSING, config_json_path(["a", "b"]))
            .validation(limit_chars_validation())
            .default_static(20_000)
    )]
    limit_chars: Option<i64>,

    #[field(
        FieldDef::builder("docnav.defaults.output")
            .process(CONFIG_PROCESSING, config_json_path(["defaults", "output"]))
            .validation(output_mode_validation())
            .default_static(OutputMode::ReadableView)
    )]
    output: OutputMode,
}

// @case WB-TYPED-FIELDS-PROJECTION-001
fn limit_chars_validation() -> FieldValidation<i64> {
    FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(100_000))
}

fn output_mode_validation() -> FieldValidation<OutputMode> {
    FieldValidation::string_enum::<OutputMode>()
}

fn docnav_fields() -> <DocnavParams as FieldDefs>::DefinitionSet {
    DocnavParams::field_defs().expect("definition set builds")
}

fn native_processing() -> ProcessingBuild<'static, JsonValue, JsonValue> {
    ProcessingBuild::new(CONFIG_PROCESSING, |raw: JsonValue| {
        raw.get("native").cloned().unwrap_or_else(|| json!({}))
    })
    .expect("processing id is valid")
}

fn valid_input_with_native() -> JsonValue {
    json!({
        "a": {"b": 4000},
        "defaults": {"output": "readable-json"},
        "native": {"theme": "dark"}
    })
}

fn consume_params(params: DocnavParams) -> (Option<i64>, OutputMode) {
    (params.defaults.limit_chars, params.defaults.output)
}

fn consume_defaults(params: DocnavParamsDefaultValues) -> (Option<i64>, Option<OutputMode>) {
    (params.defaults.limit_chars, params.defaults.output)
}

fn assert_valid_params_and_processing_result(
    processed: &ProcessedExtraction<Result<DocnavParams, FieldExtractionError>, JsonValue>,
    expected_processing: &JsonValue,
) {
    let params = processed
        .extraction()
        .as_ref()
        .expect("valid field extraction succeeds");
    assert_eq!(params.defaults.limit_chars, Some(4000));
    assert_eq!(params.defaults.output, OutputMode::ReadableJson);
    assert_eq!(
        processed.processing().processing_id().as_str(),
        CONFIG_PROCESSING
    );
    assert_eq!(processed.processing().value(), expected_processing);
}

#[test]
fn derived_field_defs_project_metadata_and_defaults() {
    let fields = docnav_fields();

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
}

#[test]
fn derived_field_defs_extract_valid_json_into_typed_params_object() {
    let fields = docnav_fields();

    let input = json!({"a": {"b": 4000}, "defaults": {"output": "readable-json"}});
    let params = fields
        .extract(CONFIG_PROCESSING, &input)
        .expect("valid input extracts params");
    assert_eq!(
        consume_params(params),
        (Some(4000), OutputMode::ReadableJson)
    );
}

#[test]
fn derived_field_defs_extract_missing_optional_without_static_default() {
    let fields = docnav_fields();

    let input = json!({"defaults": {"output": "protocol-json"}});
    let params = fields
        .extract(CONFIG_PROCESSING, &input)
        .expect("missing optional field extracts");
    assert_eq!(params.defaults.limit_chars, None);
    assert_eq!(params.defaults.output, OutputMode::ProtocolJson);
}

#[test]
fn derived_field_defs_static_defaults_fill_missing_inputs() {
    let fields = docnav_fields();

    let input = json!({"defaults": {"output": "protocol-json"}});
    let params = fields
        .extract_with_static_defaults(CONFIG_PROCESSING, &input)
        .expect("missing value falls back to static default");
    assert_eq!(
        consume_params(params),
        (Some(20_000), OutputMode::ProtocolJson)
    );

    fields
        .validate_with_static_defaults(CONFIG_PROCESSING, &json!({}))
        .expect("static defaults satisfy missing required fields");
}

#[test]
fn derived_field_defs_process_returns_extraction_and_processing_result() {
    let fields = docnav_fields();
    let processing = native_processing();

    let processed = fields.process(&processing, &valid_input_with_native());

    assert_valid_params_and_processing_result(&processed, &json!({"theme": "dark"}));
}

#[test]
fn derived_field_defs_process_keeps_processing_result_when_extraction_fails() {
    let fields = docnav_fields();
    let processing = native_processing();

    let processed = fields.process(
        &processing,
        &json!({
            "a": {"b": "invalid"},
            "defaults": {"output": "readable-json"},
            "native": {"theme": "dark"}
        }),
    );

    assert!(processed.extraction().is_err());
    assert_eq!(processed.processing().value(), &json!({"theme": "dark"}));
}

#[test]
fn derived_field_defs_extract_with_passthrough_keeps_original_json_by_default() {
    let fields = docnav_fields();
    let input = valid_input_with_native();

    let processed = fields.extract_with_passthrough(CONFIG_PROCESSING, &input, None);

    assert_valid_params_and_processing_result(&processed, &input);
}

#[test]
fn derived_field_defs_extract_with_passthrough_returns_processing_result() {
    let fields = docnav_fields();
    let processing = native_processing();

    let processed = fields.extract_with_passthrough(
        CONFIG_PROCESSING,
        &valid_input_with_native(),
        Some(&processing),
    );

    assert_valid_params_and_processing_result(&processed, &json!({"theme": "dark"}));
}

#[test]
fn json_field_set_unused_fields_reports_direct_unconsumed_keys() {
    let fields = docnav_fields();
    let input = json!({
        "a": {"b": 4000, "extra": true},
        "defaults": {"output": "readable-json", "theme": "dark"},
        "native": {"theme": "dark"}
    });
    let json_fields = JsonFieldSet::new(fields.as_ref());

    assert_eq!(
        json_fields
            .unused_fields(CONFIG_PROCESSING, &input, std::iter::empty::<&str>())
            .expect("unused root keys are computed"),
        json!({"native": {"theme": "dark"}})
    );
    assert_eq!(
        json_fields
            .unused_fields(CONFIG_PROCESSING, &input, ["a"])
            .expect("unused nested keys are computed"),
        json!({"extra": true})
    );
    assert_eq!(
        json_fields
            .unused_fields(CONFIG_PROCESSING, &input, ["defaults"])
            .expect("unused sibling keys are computed"),
        json!({"theme": "dark"})
    );
}

#[test]
fn built_field_defs_can_return_typed_builder_for_static_reuse() {
    let fields = docnav_fields();

    let mut fields2 = fields.to_builder();
    fields2.defaults.limit_chars = fields2.defaults.limit_chars.process(
        CONFIG_PROCESSING,
        config_json_path(["defaults", "limit_chars"]),
    );
    let fields2 = fields2.build().expect("updated definition set builds");

    let original = fields
        .extract(
            CONFIG_PROCESSING,
            &json!({
                "a": {"b": 111},
                "defaults": {"limit_chars": 222, "output": "readable-view"}
            }),
        )
        .expect("original fields still use original path");
    assert_eq!(original.defaults.limit_chars, Some(111));

    let reused = fields2
        .extract(
            CONFIG_PROCESSING,
            &json!({
                "a": {"b": 111},
                "defaults": {"limit_chars": 222, "output": "readable-view"}
            }),
        )
        .expect("rebuilt fields use updated path");
    assert_eq!(reused.defaults.limit_chars, Some(222));
}

#[test]
fn builder_process_json_path_drives_named_field_processing() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(
            FieldDef::builder("docnav.defaults.limit_chars")
                .process(
                    "config",
                    ProcessStrategy::json_path(["defaults", "limit_chars"])
                )
                .validation(limit_chars_validation())
        )]
        limit_chars: Option<i64>,
    }

    let fields = Params::field_defs().expect("definition set builds");

    let params = fields
        .extract("config", &json!({"defaults": {"limit_chars": 4096}}))
        .expect("field processing uses configured json path");

    assert_eq!(params.limit_chars, Some(4096));
}

#[test]
fn set_build_rejects_same_processing_id_with_different_input_kind() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(
            FieldDef::builder("docnav.defaults.limit_chars")
                .process(
                    "config",
                    ProcessStrategy::json_path(["defaults", "limit_chars"])
                )
                .validation(limit_chars_validation())
        )]
        limit_chars: Option<i64>,

        #[field(
            FieldDef::builder("docnav.defaults.output")
                .process("config", ProcessStrategy::rust_field())
                .validation(output_mode_validation())
        )]
        output: Option<OutputMode>,
    }

    let error =
        Params::field_defs().expect_err("processing input kind conflict fails at set build");

    assert!(matches!(
        error,
        FieldDefSetBuildError::ProcessingInputKindConflict { .. }
    ));
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
                .process(CONFIG_PROCESSING, config_json_path(["defaults", "limit_chars"]))
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
        .extract(
            CONFIG_PROCESSING,
            &json!({"defaults": {"limit_chars": 4000}}),
        )
        .expect("valid input extracts params");
    assert_eq!(params.defaults.limit_chars, Some(4000));

    let params = fields
        .extract_with_static_defaults(CONFIG_PROCESSING, &json!({"defaults": {}}))
        .expect("missing input uses static default");
    assert_eq!(params.defaults.limit_chars, Some(20_000));
}

#[test]
fn extract_and_validate_share_validation_errors() {
    let fields = docnav_fields();
    let input = json!({"a": {"b": "4000"}, "defaults": {"output": "xml"}});

    let extract_error = fields
        .extract(CONFIG_PROCESSING, &input)
        .expect_err("extract fails");
    let validate_error = fields
        .validate(CONFIG_PROCESSING, &input)
        .expect_err("validate fails");

    assert_eq!(extract_error, validate_error);
    assert_eq!(validation_failures(&extract_error).len(), 2);
    assert!(validation_failures(&extract_error).iter().any(|failure| {
        matches!(
            failure.reason,
            ValidationReason::WrongType {
                expected: ValueKind::Integer,
                actual: ActualValueKind::String
            }
        )
    }));
    assert!(validation_failures(&extract_error)
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
                .process(CONFIG_PROCESSING, config_json_path(["defaults", "type"]))
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
        .extract(CONFIG_PROCESSING, &json!({"defaults": {"type": "plain"}}))
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
                .process(CONFIG_PROCESSING, config_json_path(["defaults", "limit_chars"]))
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
                .process(CONFIG_PROCESSING, config_json_path(["defaults", "output"]))
                .validation(FieldValidation::string())
        )]
        output: String,
    }

    let fields = Params::field_defs().expect("definition set builds");

    let error = fields
        .extract_with_static_defaults(CONFIG_PROCESSING, &json!({"defaults": {}}))
        .expect_err("missing required value without default still fails");

    assert_eq!(
        validation_failures(&error)[0].reason,
        ValidationReason::MissingRequired
    );
}
