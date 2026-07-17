use docnav_typed_fields::{
    ActualValueKind, BuildError, CliBooleanEncoding, CliProcessingMetadata, ExpectedFieldShape,
    FieldDef, FieldDefBuildFailure, FieldDefBuilder, FieldDefSet, FieldDefSetBuildError,
    FieldIdentity, FieldValidation, MergeStrategy, ProcessStrategy, ProcessingId,
    ProcessingInputKind, ProcessingLocator, TypedValue, ValidationReason, ValueKind,
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
    assert_eq!(metadata[0].merge_strategy(), MergeStrategy::Replace);
    assert_eq!(metadata[1].merge_strategy(), MergeStrategy::Append);
    assert_eq!(metadata[2].merge_strategy(), MergeStrategy::MapMerge);
    assert_eq!(metadata[3].merge_strategy(), MergeStrategy::DenyConflict);

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
