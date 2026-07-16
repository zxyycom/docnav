use docnav_typed_fields::{
    ActualValueKind, ExpectedFieldShape, FieldDef, FieldDefSet, FieldDefs, FieldIdentity,
    FieldValidation, FieldValueMap, MergeStrategy, ProcessStrategy, ProcessingId,
    ProcessingInputKind, ProcessingLocator, TypedValue, ValidationReason,
};
use serde_json::json;

fn canonical_fields() -> FieldDefSet {
    FieldDefSet::builder()
        .field(
            FieldDef::builder("limit")
                .process("cli", ProcessStrategy::cli_flag("--limit"))
                .process("env", ProcessStrategy::env_var("APP_LIMIT"))
                .process("config", ProcessStrategy::config_path(["read", "limit"]))
                .validation(FieldValidation::int()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("canonical field set builds")
}

#[test]
// @case WB-PARAM-FIELD-CONTRACT-001
fn canonical_processing_metadata_exposes_source_locators() {
    let fields = canonical_fields();

    let cli = fields.processing_metadata(&ProcessingId::new("cli").expect("valid processing id"));
    assert_eq!(cli.len(), 1);
    assert_eq!(cli[0].input_kind, ProcessingInputKind::CliArguments);
    assert_eq!(
        cli[0].locator,
        ProcessingLocator::CliFlag("--limit".to_owned())
    );

    let env = fields.processing_metadata(&ProcessingId::new("env").expect("valid processing id"));
    assert_eq!(env.len(), 1);
    assert_eq!(env[0].input_kind, ProcessingInputKind::EnvironmentVariables);
    assert_eq!(
        env[0].locator,
        ProcessingLocator::EnvVar("APP_LIMIT".to_owned())
    );

    let config =
        fields.processing_metadata(&ProcessingId::new("config").expect("valid processing id"));
    assert_eq!(config.len(), 1);
    assert_eq!(config[0].input_kind, ProcessingInputKind::JsonValue);
    assert_eq!(
        config[0].locator,
        ProcessingLocator::ConfigPath(
            docnav_typed_fields::FieldPath::new(["read", "limit"]).expect("path")
        )
    );
}

#[test]
fn set_build_rejects_duplicate_and_invalid_source_locators() {
    let duplicate = FieldDefSet::builder()
        .field(
            FieldDef::builder("limit")
                .process("cli", ProcessStrategy::cli_flag("--value"))
                .validation(FieldValidation::int()),
            ExpectedFieldShape::optional(),
        )
        .field(
            FieldDef::builder("mode")
                .process("cli", ProcessStrategy::cli_flag("--value"))
                .validation(FieldValidation::string()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect_err("duplicate CLI locator fails deterministically");
    assert!(matches!(
        duplicate,
        docnav_typed_fields::FieldDefSetBuildError::DuplicateProcessingLocator(_)
    ));

    let invalid = FieldDefSet::builder()
        .field(
            FieldDef::builder("limit")
                .process("cli", ProcessStrategy::cli_flag(" "))
                .validation(FieldValidation::int()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect_err("blank CLI locator fails during set build");
    assert!(matches!(
        invalid,
        docnav_typed_fields::FieldDefSetBuildError::Field(
            docnav_typed_fields::FieldDefBuildFailure {
                error: docnav_typed_fields::BuildError::InvalidCliFlag,
                ..
            }
        )
    ));

    let invalid_env = FieldDefSet::builder()
        .field(
            FieldDef::builder("limit")
                .process("env", ProcessStrategy::env_var("APP=LIMIT"))
                .validation(FieldValidation::int()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect_err("invalid environment locator fails during set build");
    assert!(matches!(
        invalid_env,
        docnav_typed_fields::FieldDefSetBuildError::Field(
            docnav_typed_fields::FieldDefBuildFailure {
                error: docnav_typed_fields::BuildError::InvalidEnvVar,
                ..
            }
        )
    ));

    let invalid_identity = FieldDefSet::builder()
        .field(
            FieldDef::builder("a..b")
                .process("config", ProcessStrategy::config_path(["a", "b"]))
                .process("cli", ProcessStrategy::cli_flag("--value"))
                .process("env", ProcessStrategy::env_var("APP_VALUE"))
                .validation(FieldValidation::string()),
            ExpectedFieldShape::optional(),
        )
        .build();
    assert!(matches!(
        invalid_identity.expect_err("an identity with an empty path segment must fail"),
        docnav_typed_fields::FieldDefSetBuildError::Field(
            docnav_typed_fields::FieldDefBuildFailure {
                error: docnav_typed_fields::BuildError::EmptyPathSegment,
                ..
            }
        )
    ));
}

#[test]
fn merge_strategy_is_canonical_field_metadata() {
    let fields = FieldDefSet::builder()
        .field(
            FieldDef::builder("scalar")
                .process("config", ProcessStrategy::config_path(["scalar"]))
                .validation(FieldValidation::string()),
            ExpectedFieldShape::optional(),
        )
        .field(
            FieldDef::builder("items")
                .process("config", ProcessStrategy::config_path(["items"]))
                .validation(FieldValidation::array())
                .merge(MergeStrategy::Append),
            ExpectedFieldShape::optional(),
        )
        .field(
            FieldDef::builder("labels")
                .process("config", ProcessStrategy::config_path(["labels"]))
                .validation(FieldValidation::object())
                .merge(MergeStrategy::MapMerge),
            ExpectedFieldShape::optional(),
        )
        .field(
            FieldDef::builder("mode")
                .process("config", ProcessStrategy::config_path(["mode"]))
                .validation(FieldValidation::string())
                .merge(MergeStrategy::DenyConflict),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("merge metadata builds");

    let metadata = fields.schema_metadata();
    assert_eq!(metadata[0].merge_strategy, MergeStrategy::Replace);
    assert_eq!(metadata[1].merge_strategy, MergeStrategy::Append);
    assert_eq!(metadata[2].merge_strategy, MergeStrategy::MapMerge);
    assert_eq!(metadata[3].merge_strategy, MergeStrategy::DenyConflict);

    let incompatible = FieldDefSet::builder()
        .field(
            FieldDef::builder("scalar")
                .process("config", ProcessStrategy::config_path(["scalar"]))
                .validation(FieldValidation::string())
                .merge(MergeStrategy::Append),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect_err("append is only valid for canonical array fields");
    assert!(matches!(
        incompatible,
        docnav_typed_fields::FieldDefSetBuildError::Field(
            docnav_typed_fields::FieldDefBuildFailure {
                error: docnav_typed_fields::BuildError::IncompatibleMergeStrategy {
                    value_kind: docnav_typed_fields::ValueKind::String,
                    merge_strategy: MergeStrategy::Append,
                },
                ..
            }
        )
    ));
}

#[test]
fn field_lookup_uses_canonical_final_value_validation() {
    let fields = canonical_fields();
    let identity = FieldIdentity::new("limit").expect("identity");
    let field = fields.field(&identity).expect("canonical field lookup");

    assert_eq!(
        field.validate_value(&json!(12)).expect("integer validates"),
        TypedValue::Integer(12)
    );
    let failure = field
        .validate_value(&json!("12"))
        .expect_err("wrong final value kind fails");
    assert_eq!(
        failure.reason,
        ValidationReason::WrongType {
            expected: docnav_typed_fields::ValueKind::Integer,
            actual: ActualValueKind::String,
        }
    );
}

#[derive(Debug, PartialEq, FieldDefs)]
struct MaterializedParameters {
    #[field(
        FieldDef::builder("z_limit")
            .process("config", ProcessStrategy::config_path(["limit"]))
            .validation(FieldValidation::int())
    )]
    limit: i64,

    #[field(
        FieldDef::builder("a_mode")
            .process("config", ProcessStrategy::config_path(["mode"]))
            .validation(FieldValidation::string())
    )]
    mode: Option<String>,
}

#[test]
fn derived_field_set_materializes_canonical_typed_values() {
    let fields = MaterializedParameters::field_defs().expect("field definitions build");
    let values = FieldValueMap::from([
        (
            FieldIdentity::new("z_limit").expect("identity"),
            TypedValue::Integer(25),
        ),
        (
            FieldIdentity::new("a_mode").expect("identity"),
            TypedValue::String("fast".to_owned()),
        ),
    ]);
    assert_eq!(
        fields
            .materialize(&values)
            .expect("canonical typed values materialize"),
        MaterializedParameters {
            limit: 25,
            mode: Some("fast".to_owned()),
        }
    );

    let invalid = FieldValueMap::from([(
        FieldIdentity::new("z_limit").expect("identity"),
        TypedValue::String("25".to_owned()),
    )]);
    let failures = fields
        .materialize(&invalid)
        .expect_err("materialization revalidates final values");
    assert!(failures.failures().iter().any(|failure| {
        failure.field.as_str() == "z_limit"
            && matches!(
                failure.reason,
                ValidationReason::WrongType {
                    expected: docnav_typed_fields::ValueKind::Integer,
                    actual: ActualValueKind::String
                }
            )
    }));

    let missing = FieldValueMap::new();
    let failures = fields
        .materialize(&missing)
        .expect_err("missing required canonical value fails materialization");
    assert!(failures.failures().iter().any(|failure| {
        failure.field.as_str() == "z_limit"
            && matches!(failure.reason, ValidationReason::MissingRequired)
    }));
}

#[derive(Debug, PartialEq, FieldDefs)]
struct MaterializedNumbers {
    #[field(
        FieldDef::builder("required_number")
            .process("config", ProcessStrategy::config_path(["required_number"]))
            .validation(FieldValidation::num())
    )]
    required: f64,

    #[field(
        FieldDef::builder("optional_number")
            .process("config", ProcessStrategy::config_path(["optional_number"]))
            .validation(FieldValidation::num())
    )]
    optional: Option<f64>,
}

#[test]
fn materialization_rejects_non_finite_numbers_for_optional_and_required_fields() {
    let fields = MaterializedNumbers::field_defs().expect("field definitions build");
    let non_finite = f64::NAN;

    let optional_values = FieldValueMap::from([
        (
            FieldIdentity::new("required_number").expect("identity"),
            TypedValue::Number(1.0),
        ),
        (
            FieldIdentity::new("optional_number").expect("identity"),
            TypedValue::Number(non_finite),
        ),
    ]);
    let failures = fields
        .materialize(&optional_values)
        .expect_err("non-finite optional number must not become an absent value");
    assert!(failures
        .failures()
        .iter()
        .any(|failure| failure.field.as_str() == "optional_number"));

    let required_values = FieldValueMap::from([(
        FieldIdentity::new("required_number").expect("identity"),
        TypedValue::Number(non_finite),
    )]);
    let failures = fields
        .materialize(&required_values)
        .expect_err("non-finite required number must fail canonical validation");
    assert!(failures
        .failures()
        .iter()
        .any(|failure| failure.field.as_str() == "required_number"));
}

#[derive(Debug, PartialEq, FieldDefs)]
struct NestedMaterializedParameters {
    #[field(group)]
    nested: NestedMaterializedValues,
}

#[derive(Debug, PartialEq, FieldDefs)]
struct NestedMaterializedValues {
    #[field(
        FieldDef::builder("nested.value")
            .process("config", ProcessStrategy::config_path(["nested", "value"]))
            .validation(FieldValidation::int())
    )]
    value: i64,
}

#[test]
fn derived_field_set_materializes_nested_groups() {
    let fields = NestedMaterializedParameters::field_defs().expect("field definitions build");
    let values = FieldValueMap::from([(
        FieldIdentity::new("nested.value").expect("identity"),
        TypedValue::Integer(7),
    )]);

    assert_eq!(
        fields
            .materialize(&values)
            .expect("nested group materializes"),
        NestedMaterializedParameters {
            nested: NestedMaterializedValues { value: 7 },
        }
    );
}
