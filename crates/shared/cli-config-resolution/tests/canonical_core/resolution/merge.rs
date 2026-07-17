use cli_config_resolution::{
    resolve, DiagnosticReason, ExpectedFieldShape, FieldBound, FieldDef, FieldDefSet, FieldLength,
    FieldValidation, MergeStrategy, ProcessStrategy, Source, SourceCandidate, SourceId, SourceKind,
    TypedValue,
};
use serde_json::json;

use crate::support::{candidate, direct_locator, identity, merge_field_set, source};

#[test]
fn append_merge_preserves_source_order_and_provenance() {
    let fields = merge_field_set("items", FieldValidation::array(), MergeStrategy::Append);
    let low = source("low", 10, [candidate("items", json!(["low"]))]);
    let same_priority_earlier =
        source("same-earlier", 20, [candidate("items", json!(["earlier"]))]);
    let same_priority_later = source("same-later", 20, [candidate("items", json!(["later"]))]);

    let result =
        resolve(&fields, &[low, same_priority_earlier, same_priority_later]).expect("valid input");
    let values = result.materialize().expect("valid merge");
    assert_eq!(
        values[&identity("items")],
        TypedValue::Array(vec![json!("low"), json!("earlier"), json!("later")])
    );
    assert_eq!(
        result
            .trace(&identity("items"))
            .expect("append trace")
            .contributors
            .iter()
            .map(|candidate| candidate.source_id.as_str())
            .collect::<Vec<_>>(),
        vec!["low", "same-earlier", "same-later"]
    );
}

#[test]
fn append_applies_canonical_constraints_only_after_merging_contributors() {
    let fields = FieldDefSet::builder()
        .field(
            FieldDef::builder("items")
                .process("custom", ProcessStrategy::rust_field())
                .validation(
                    FieldValidation::array().length(FieldLength::min(FieldBound::closed(2))),
                )
                .merge(MergeStrategy::Append),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("field set");
    let low = source("low", 10, [candidate("items", json!(["low"]))]);
    let high = source("high", 20, [candidate("items", json!(["high"]))]);

    let result = resolve(&fields, &[low, high]).expect("valid input");

    assert_eq!(
        result
            .materialize()
            .expect("merged array satisfies minimum length")[&identity("items")],
        TypedValue::Array(vec![json!("low"), json!("high")])
    );
}

#[test]
fn merged_value_is_revalidated() {
    let fields = merge_field_set(
        "items",
        FieldValidation::array().unique_items(),
        MergeStrategy::Append,
    );
    let low = source("low", 10, [candidate("items", json!(["same"]))]);
    let high = source("high", 20, [candidate("items", json!(["same"]))]);

    let result = resolve(&fields, &[low, high]).expect("valid input");
    assert!(result.materialize().is_err());
    assert!(result.diagnostics().iter().any(|diagnostic| {
        diagnostic.field.as_str() == "items"
            && matches!(diagnostic.reason, DiagnosticReason::FinalValidation(_))
    }));
}

#[test]
fn deny_conflict_reports_all_source_locators() {
    let fields = merge_field_set(
        "mode",
        FieldValidation::string(),
        MergeStrategy::DenyConflict,
    );
    let low = Source::new(
        SourceId::new("low").expect("source id"),
        SourceKind::Direct,
        10,
        vec![SourceCandidate::value(
            identity("mode"),
            direct_locator("low-mode"),
            json!("a"),
        )],
    )
    .expect("low source");
    let high = Source::new(
        SourceId::new("high").expect("source id"),
        SourceKind::Direct,
        20,
        vec![SourceCandidate::value(
            identity("mode"),
            direct_locator("high-mode"),
            json!("b"),
        )],
    )
    .expect("high source");

    let result = resolve(&fields, &[low, high]).expect("valid input");
    assert!(result.materialize().is_err());
    let conflict = result
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.field.as_str() == "mode")
        .expect("deny-conflict diagnostic");
    let DiagnosticReason::MergeConflict(locators) = &conflict.reason else {
        panic!("expected deny-conflict reason");
    };
    assert_eq!(
        locators,
        &vec![direct_locator("low-mode"), direct_locator("high-mode"),]
    );
}

#[test]
fn map_merge_preserves_source_order_and_provenance() {
    let fields = merge_field_set("map", FieldValidation::object(), MergeStrategy::MapMerge);
    let low = source(
        "low",
        10,
        [candidate("map", json!({"same": "low", "low": true}))],
    );
    let high = source(
        "high",
        20,
        [candidate("map", json!({"same": "high", "high": true}))],
    );

    let result = resolve(&fields, &[low, high]).expect("valid input");
    let values = result.materialize().expect("valid map merge");
    assert_eq!(
        values[&identity("map")],
        TypedValue::Object(
            serde_json::from_value(json!({"same": "high", "low": true, "high": true}))
                .expect("object")
        )
    );
    assert_eq!(
        result
            .trace(&identity("map"))
            .expect("map trace")
            .contributors
            .iter()
            .map(|candidate| candidate.source_id.as_str())
            .collect::<Vec<_>>(),
        vec!["low", "high"]
    );
}

#[test]
fn deny_conflict_accepts_equal_values() {
    let fields = merge_field_set(
        "mode",
        FieldValidation::string(),
        MergeStrategy::DenyConflict,
    );
    let low = source("low", 10, [candidate("mode", json!("same"))]);
    let high = source("high", 20, [candidate("mode", json!("same"))]);

    let values = resolve(&fields, &[low, high])
        .expect("valid input")
        .materialize()
        .expect("equal values do not conflict");
    assert_eq!(
        values[&identity("mode")],
        TypedValue::String("same".to_owned())
    );
}
