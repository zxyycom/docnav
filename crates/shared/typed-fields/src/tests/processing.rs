use super::*;

#[test]
fn processing_id_try_from_rejects_empty_value() {
    assert_eq!(ProcessingId::try_from(" "), Err(InvalidProcessingId));
}

#[test]
fn field_build_rejects_duplicate_processing_id() {
    let error = FieldDef::builder("docnav.defaults.limit")
        .process("config", config_json_path(["defaults", "limit"]))
        .process("config", config_json_path(["legacy", "limit"]))
        .validation(FieldValidation::int())
        .build()
        .expect_err("duplicate processing id must fail at field build");

    assert_eq!(
        error,
        BuildError::DuplicateProcessingId {
            processing_id: ProcessingId::new("config").expect("valid processing id"),
        }
    );
}

#[test]
fn set_build_rejects_missing_processing_strategy() {
    let error = FieldDefSet::builder()
        .field_with_declaration_path(
            ["defaults", "limit"],
            FieldDef::builder("docnav.defaults.limit").validation(FieldValidation::int()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect_err("missing processing definition fails at set build");

    assert_eq!(
        error,
        FieldDefSetBuildError::Field(FieldDefBuildFailure {
            declaration_path: Some(vec!["defaults".to_string(), "limit".to_string()]),
            error: BuildError::MissingProcessingStrategy,
        })
    );
}
