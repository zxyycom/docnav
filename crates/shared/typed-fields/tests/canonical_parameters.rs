use docnav_typed_fields::{
    ActualValueKind, BuildError, CliBooleanEncoding, CliProcessingMetadata, DefaultMetadata,
    ExpectedFieldShape, FieldDef, FieldDefBuildFailure, FieldDefBuilder, FieldDefDeclaration,
    FieldDefSet, FieldDefSetBuildError, FieldDefs, FieldIdentity, FieldValidation, FieldValueMap,
    MergeStrategy, ProcessStrategy, ProcessingId, ProcessingInputKind, ProcessingLocator,
    TypedValue, ValidationReason, ValueKind,
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
fn cli_metadata_survives_clone_type_erasure_build_and_aggregation() {
    let cli = ProcessingId::new("cli").expect("valid processing id");
    let mapped_builder = FieldDef::builder("pagination")
        .process(
            "cli",
            ProcessStrategy::cli_flag("--pagination").cli_metadata(
                CliProcessingMetadata::new()
                    .help("Enable or disable pagination")
                    .value_name("STATE")
                    .boolean_encoding(CliBooleanEncoding::explicit("enabled", "disabled")),
            ),
        )
        .validation(FieldValidation::boolean())
        .default_static(true);
    let declaration =
        FieldDefDeclaration::new(mapped_builder.clone(), ExpectedFieldShape::optional());

    let erased = declaration
        .processing_metadata(&cli)
        .expect("type-erased declaration builds")
        .expect("CLI processing exists");
    let fields = FieldDefSet::builder()
        .field_declaration(declaration)
        .field(
            FieldDef::builder("verbose")
                .process(
                    "cli",
                    ProcessStrategy::cli_flag("--verbose").cli_metadata(
                        CliProcessingMetadata::new()
                            .help("Enable verbose output")
                            .boolean_encoding(CliBooleanEncoding::PresenceMeansTrue),
                    ),
                )
                .validation(FieldValidation::boolean()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("CLI metadata declarations build");
    let aggregated = fields.processing_metadata(&cli);

    assert_eq!(aggregated.len(), 2);
    assert_eq!(aggregated[0], erased);
    assert_eq!(aggregated[0].value_kind, ValueKind::Boolean);
    assert_eq!(aggregated[0].default, DefaultMetadata::Static(json!(true)));
    assert_eq!(aggregated[0].merge_strategy, MergeStrategy::Replace);
    assert_eq!(
        aggregated[0].cli,
        Some(
            CliProcessingMetadata::new()
                .help("Enable or disable pagination")
                .value_name("STATE")
                .boolean_encoding(CliBooleanEncoding::explicit("enabled", "disabled"))
        )
    );
    assert_eq!(
        aggregated[1].cli,
        Some(
            CliProcessingMetadata::new()
                .help("Enable verbose output")
                .boolean_encoding(CliBooleanEncoding::PresenceMeansTrue)
        )
    );
}

#[test]
fn config_only_field_builds_without_cli_metadata() {
    let fields = FieldDefSet::builder()
        .field(
            FieldDef::builder("theme")
                .process("config", ProcessStrategy::config_path(["theme"]))
                .validation(FieldValidation::string()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("config-only field builds");

    assert!(fields
        .processing_metadata(&ProcessingId::new("cli").expect("valid processing id"))
        .is_empty());
    assert_eq!(
        fields.processing_metadata(&ProcessingId::new("config").expect("valid processing id"))[0]
            .cli,
        None
    );
}

#[test]
fn field_build_rejects_invalid_cli_metadata_declarations() {
    let invalid_attachment = FieldDefSet::builder()
        .field_with_declaration_path(
            ["parameters", "theme"],
            FieldDef::builder("theme")
                .process(
                    "config",
                    ProcessStrategy::config_path(["theme"])
                        .cli_metadata(CliProcessingMetadata::new().help("Theme")),
                )
                .validation(FieldValidation::string()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect_err("CLI metadata on config processing fails");
    assert_eq!(
        invalid_attachment,
        FieldDefSetBuildError::Field(FieldDefBuildFailure {
            declaration_path: Some(vec!["parameters".to_owned(), "theme".to_owned()]),
            error: BuildError::CliMetadataRequiresCliFlag,
        })
    );

    let duplicate = canonical_build_error(
        FieldDef::builder("theme")
            .process(
                "cli",
                ProcessStrategy::cli_flag("--theme")
                    .cli_metadata(CliProcessingMetadata::new().help("Theme"))
                    .cli_metadata(CliProcessingMetadata::new().value_name("THEME")),
            )
            .validation(FieldValidation::string()),
    );
    assert_eq!(duplicate, BuildError::DuplicateCliMetadata);

    let incompatible = canonical_build_error(
        FieldDef::builder("theme")
            .process(
                "cli",
                ProcessStrategy::cli_flag("--theme").cli_metadata(
                    CliProcessingMetadata::new()
                        .boolean_encoding(CliBooleanEncoding::PresenceMeansTrue),
                ),
            )
            .validation(FieldValidation::string()),
    );
    assert_eq!(
        incompatible,
        BuildError::IncompatibleCliBooleanEncoding {
            value_kind: ValueKind::String,
        }
    );

    let incomplete = canonical_build_error(
        FieldDef::builder("pagination")
            .process(
                "cli",
                ProcessStrategy::cli_flag("--pagination").cli_metadata(
                    CliProcessingMetadata::new().boolean_encoding(CliBooleanEncoding::Explicit {
                        true_token: Some("enabled".to_owned()),
                        false_token: None,
                    }),
                ),
            )
            .validation(FieldValidation::boolean()),
    );
    assert_eq!(incomplete, BuildError::IncompleteCliBooleanMapping);

    let ambiguous = canonical_build_error(
        FieldDef::builder("pagination")
            .process(
                "cli",
                ProcessStrategy::cli_flag("--pagination").cli_metadata(
                    CliProcessingMetadata::new().boolean_encoding(CliBooleanEncoding::Explicit {
                        true_token: Some("enabled".to_owned()),
                        false_token: Some("enabled".to_owned()),
                    }),
                ),
            )
            .validation(FieldValidation::boolean()),
    );
    assert_eq!(
        ambiguous,
        BuildError::AmbiguousCliBooleanMapping {
            token: "enabled".to_owned(),
        }
    );
}

fn canonical_build_error<T: 'static>(builder: FieldDefBuilder<T>) -> BuildError {
    match FieldDefSet::builder()
        .field(builder, ExpectedFieldShape::optional())
        .build()
        .expect_err("invalid canonical field declaration fails")
    {
        FieldDefSetBuildError::Field(FieldDefBuildFailure { error, .. }) => error,
        error => panic!("expected canonical field build failure, got {error:?}"),
    }
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
