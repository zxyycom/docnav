use cli_config_resolution::{
    extract_env, CandidateInput, CandidateInvalidReason, DiagnosticReason, ExpectedFieldShape,
    FieldDef, FieldDefSet, FieldValidation, JsonValue, ProcessStrategy, ProcessingId, Resolver,
    SourceId, SourceKind, SourceLocator,
};
use serde_json::json;

#[test]
fn env_extractor_reads_declared_values_only_and_omits_missing_values() {
    let fields = FieldDefSet::builder()
        .field(
            FieldDef::builder("limit")
                .process("env", ProcessStrategy::env_var("APP_LIMIT"))
                .validation(FieldValidation::int()),
            ExpectedFieldShape::optional(),
        )
        .field(
            FieldDef::builder("missing")
                .process("env", ProcessStrategy::env_var("APP_MISSING"))
                .validation(FieldValidation::string()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("field set");

    let env = extract_env(
        &fields,
        &ProcessingId::new("env").expect("valid processing id"),
        SourceId::new("env").expect("source id"),
        20,
        [
            ("APP_LIMIT".to_owned(), "7".to_owned()),
            ("UNKNOWN".to_owned(), "ignored".to_owned()),
        ],
    )
    .expect("env extraction");

    assert_eq!(env.id().as_str(), "env");
    assert_eq!(env.kind(), &SourceKind::Env);
    assert_eq!(env.priority(), 20);
    assert_eq!(env.candidates().len(), 1);
    let limit = &env.candidates()[0];
    assert_eq!(limit.field().as_str(), "limit");
    assert_eq!(
        limit.locator(),
        &SourceLocator::EnvVar("APP_LIMIT".to_owned())
    );
    assert!(matches!(
        limit.input(),
        CandidateInput::Value(JsonValue::Number(number)) if number.as_i64() == Some(7)
    ));
}

#[test]
fn selected_invalid_env_value_preserves_diagnostic_facts() {
    let fields = FieldDefSet::builder()
        .field(
            FieldDef::builder("enabled")
                .process("env", ProcessStrategy::env_var("APP_ENABLED"))
                .validation(FieldValidation::boolean()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("field set");
    let env = extract_env(
        &fields,
        &ProcessingId::new("env").expect("valid processing id"),
        SourceId::new("env").expect("source id"),
        20,
        [("APP_ENABLED", "not-a-bool")],
    )
    .expect("env extraction");

    let result = Resolver::resolve(&fields, &[env]).expect("valid resolver input");
    let error = result
        .materialize()
        .expect_err("selected invalid env blocks");
    assert!(error.diagnostics().iter().any(|diagnostic| {
        diagnostic.field.as_str() == "enabled"
            && diagnostic.source_id.as_ref().map(SourceId::as_str) == Some("env")
            && matches!(diagnostic.locator, Some(SourceLocator::EnvVar(ref name)) if name == "APP_ENABLED")
            && diagnostic.raw == Some(json!("not-a-bool"))
            && matches!(diagnostic.reason, DiagnosticReason::InvalidCandidate(
                CandidateInvalidReason::Decode(ref reason)
            ) if reason.contains("boolean"))
    }));
}
