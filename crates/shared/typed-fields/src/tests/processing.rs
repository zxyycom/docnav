use std::cell::Cell;

use super::*;

#[derive(Clone)]
struct NativeRawInput {
    text: String,
}

// @case WB-TYPED-FIELDS-PROCESSING-001
#[test]
fn processing_build_returns_caller_processed_value_for_typed_raw_input() {
    let calls = Cell::new(0);
    let processing = ProcessingBuild::new("native-input", |raw: NativeRawInput| {
        calls.set(calls.get() + 1);
        raw.text.len()
    })
    .expect("processing id is valid");

    let processed = processing.process(NativeRawInput {
        text: "docnav".to_owned(),
    });

    assert_eq!(processing.id().as_str(), "native-input");
    assert_eq!(processed.processing_id().as_str(), "native-input");
    assert_eq!(*processed.value(), 6);
    assert_eq!(processed.into_value(), 6);
    assert_eq!(calls.get(), 1);
}

#[test]
fn processing_build_rejects_empty_processing_id() {
    let error = ProcessingBuild::new(" ", |raw: NativeRawInput| raw.text).unwrap_err();

    assert_eq!(error, InvalidProcessingId);
}

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
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: MissingProcessingDefaults,
    }

    #[derive(Debug, FieldDefs)]
    struct MissingProcessingDefaults {
        #[field(FieldDef::builder("docnav.defaults.limit").validation(FieldValidation::int()))]
        limit: Option<i64>,
    }

    let error = Params::field_defs().expect_err("missing processing definition fails at set build");

    assert_eq!(
        error,
        FieldDefSetBuildError::Field(FieldDefBuildFailure {
            declaration_path: Some(vec!["defaults".to_string(), "limit".to_string()]),
            error: BuildError::MissingProcessingStrategy,
        })
    );
}
