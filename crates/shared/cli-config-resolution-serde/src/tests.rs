use cli_config_resolution::{
    CandidateInput, ExpectedFieldShape, FieldDef, FieldDefSet, FieldPath, FieldValidation,
    ProcessStrategy, ProcessingId, ProcessingLocator, SourceId, SourceKind, SourceLocator,
};
use serde_json::json;

use super::{extract_config, ConfigExtractionError};

fn source_id(value: &str) -> SourceId {
    SourceId::new(value).expect("valid source id")
}

#[test]
fn extracts_only_declared_nested_config_path_with_source_facts() {
    let fields = FieldDefSet::builder()
        .field(
            FieldDef::builder("profile")
                .process("config", ProcessStrategy::config_path(["tool", "profile"]))
                .validation(FieldValidation::json()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("field set");
    let processing_id = ProcessingId::new("config").expect("processing id");

    let source = extract_config(
        &json!({
            "tool": {
                "profile": { "mode": "fast", "include": ["docs"] },
                "undeclared": true
            },
            "unknown_root": 10
        }),
        &fields,
        &processing_id,
        source_id("project-config"),
        20,
    )
    .expect("declared config path extracts");

    assert_eq!(source.id().as_str(), "project-config");
    assert_eq!(source.kind(), &SourceKind::Config);
    assert_eq!(source.priority(), 20);
    assert_eq!(source.candidates().len(), 1);
    let candidate = &source.candidates()[0];
    assert_eq!(candidate.field().as_str(), "profile");
    assert_eq!(
        candidate.locator(),
        &SourceLocator::ConfigPath(FieldPath::new(["tool", "profile"]).expect("path"))
    );
    assert_eq!(
        candidate.input(),
        &CandidateInput::Value(json!({ "mode": "fast", "include": ["docs"] }))
    );
}

#[test]
fn present_null_false_and_empty_containers_produce_candidates() {
    let fields = FieldDefSet::builder()
        .field(
            FieldDef::builder("value")
                .process("config", ProcessStrategy::config_path(["value"]))
                .validation(FieldValidation::json()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("field set");
    let processing_id = ProcessingId::new("config").expect("processing id");

    for (label, expected) in [
        ("null", json!(null)),
        ("false", json!(false)),
        ("empty array", json!([])),
        ("empty object", json!({})),
    ] {
        let source = extract_config(
            &json!({ "value": expected.clone() }),
            &fields,
            &processing_id,
            source_id("config"),
            20,
        )
        .expect("present leaf extracts");

        assert_eq!(source.candidates().len(), 1, "{label}");
        assert_eq!(
            source.candidates()[0].input(),
            &CandidateInput::Value(expected),
            "{label}"
        );
    }
}

#[test]
fn missing_path_or_non_object_intermediate_produces_no_candidate() {
    let fields = FieldDefSet::builder()
        .field(
            FieldDef::builder("missing")
                .process("config", ProcessStrategy::config_path(["read", "missing"]))
                .validation(FieldValidation::string()),
            ExpectedFieldShape::optional(),
        )
        .field(
            FieldDef::builder("blocked")
                .process("config", ProcessStrategy::config_path(["scalar", "child"]))
                .validation(FieldValidation::string()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("field set");

    let source = extract_config(
        &json!({ "read": { "other": "value" }, "scalar": 5 }),
        &fields,
        &ProcessingId::new("config").expect("processing id"),
        source_id("config"),
        20,
    )
    .expect("missing paths are absence");

    assert!(source.candidates().is_empty());
}

#[test]
fn non_config_locator_returns_a_public_error() {
    let fields = FieldDefSet::builder()
        .field(
            FieldDef::builder("limit")
                .process("cli", ProcessStrategy::cli_flag("--limit"))
                .validation(FieldValidation::int()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("field set");
    let processing_id = ProcessingId::new("cli").expect("processing id");

    let error = extract_config(
        &json!({ "limit": 10 }),
        &fields,
        &processing_id,
        source_id("config"),
        20,
    )
    .expect_err("a CLI locator is not a config path");

    assert_eq!(
        error,
        ConfigExtractionError::UnsupportedLocator {
            processing_id,
            locator: ProcessingLocator::CliFlag("--limit".to_owned()),
        }
    );
}
