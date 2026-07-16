use cli_config_resolution::{
    resolve, CandidateInvalidReason, DiagnosticReason, ExpectedFieldShape, FieldDef, FieldDefSet,
    FieldValidation, MergeStrategy, ProcessStrategy, SourceId, SourceLocator, TypedValue,
};
use serde_json::json;

use crate::support::{candidate, custom_field_set, identity, invalid_candidate, source};

#[test]
fn overridden_invalid_candidate_is_trace_only() {
    let replace_fields = custom_field_set("mode", false);
    let invalid_low = source(
        "invalid-low",
        10,
        [invalid_candidate("mode", json!("bad"), "cannot decode")],
    );
    let valid_high = source("valid-high", 20, [candidate("mode", json!("ok"))]);
    let result = resolve(&replace_fields, &[invalid_low, valid_high]).expect("valid input");
    assert_eq!(
        result
            .materialize()
            .expect("overridden invalid is non-blocking")[&identity("mode")],
        TypedValue::String("ok".to_owned())
    );
    assert!(result.diagnostics().is_empty());
    assert!(matches!(
        result.trace(&identity("mode")).expect("trace").overridden[0].invalid_reason,
        Some(CandidateInvalidReason::Decode(ref reason)) if reason == "cannot decode"
    ));
}

#[test]
fn selected_invalid_candidate_blocks_materialization() {
    let replace_fields = custom_field_set("mode", false);
    let valid_low = source("valid-low", 10, [candidate("mode", json!("fallback"))]);
    let invalid_high = source(
        "invalid-high",
        20,
        [invalid_candidate("mode", json!("bad"), "cannot decode")],
    );
    let result = resolve(&replace_fields, &[valid_low, invalid_high]).expect("valid input");
    assert!(result.materialize().is_err());
    assert_eq!(
        result
            .trace(&identity("mode"))
            .expect("trace")
            .selected
            .as_ref()
            .expect("selected invalid")
            .source_id
            .as_str(),
        "invalid-high"
    );
}

#[test]
fn invalid_append_contributor_blocks_with_observable_provenance() {
    let append_fields = FieldDefSet::builder()
        .field(
            FieldDef::builder("items")
                .process("custom", ProcessStrategy::rust_field())
                .validation(FieldValidation::array())
                .merge(MergeStrategy::Append),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("field set");
    let valid = source("valid", 10, [candidate("items", json!([1]))]);
    let invalid = source(
        "invalid",
        20,
        [invalid_candidate("items", json!("bad"), "not an array")],
    );
    let result = resolve(&append_fields, &[valid, invalid]).expect("valid input");
    assert!(result.materialize().is_err());
    let trace = result.trace(&identity("items")).expect("trace");
    let invalid_contributor = trace
        .contributors
        .iter()
        .find(|contributor| contributor.source_id.as_str() == "invalid")
        .expect("invalid merge contributor");
    assert_eq!(
        invalid_contributor.locator,
        SourceLocator::Custom("items".to_owned())
    );
    assert_eq!(invalid_contributor.raw, json!("bad"));
    assert!(matches!(
        invalid_contributor.invalid_reason,
        Some(CandidateInvalidReason::Decode(ref reason)) if reason == "not an array"
    ));
    assert!(result.diagnostics().iter().any(|diagnostic| {
        diagnostic.source_id.as_ref().map(SourceId::as_str) == Some("invalid")
            && diagnostic.locator == Some(SourceLocator::Custom("items".to_owned()))
            && diagnostic.raw == Some(json!("bad"))
            && matches!(
                diagnostic.reason,
                DiagnosticReason::InvalidCandidate(CandidateInvalidReason::Decode(ref reason))
                    if reason == "not an array"
            )
    }));
}
