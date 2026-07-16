use cli_config_resolution::{
    DiagnosticReason, ExpectedFieldShape, FieldBound, FieldDef, FieldDefSet, FieldLength,
    FieldValidation, MergeStrategy, ProcessStrategy, Resolver, Source, SourceCandidate, SourceId,
    SourceKind, SourceLocator, TypedValue,
};
use serde_json::json;

use crate::support::{candidate, identity, merge_field_set, source};

#[test]
fn append_merge_preserves_source_order_and_provenance() {
    let fields = merge_field_set("items", FieldValidation::array(), MergeStrategy::Append);
    let low = source("low", 10, [candidate("items", json!(["low"]))]);
    let same_priority_earlier =
        source("same-earlier", 20, [candidate("items", json!(["earlier"]))]);
    let same_priority_later = source("same-later", 20, [candidate("items", json!(["later"]))]);

    let result = Resolver::resolve(&fields, &[low, same_priority_earlier, same_priority_later])
        .expect("valid input");
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

    let result = Resolver::resolve(&fields, &[low, high]).expect("valid input");

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

    let result = Resolver::resolve(&fields, &[low, high]).expect("valid input");
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
        SourceKind::Custom("test".to_owned()),
        10,
        vec![SourceCandidate::value(
            identity("mode"),
            SourceLocator::Custom("low-mode".to_owned()),
            json!("a"),
        )],
    )
    .expect("low source");
    let high = Source::new(
        SourceId::new("high").expect("source id"),
        SourceKind::Custom("test".to_owned()),
        20,
        vec![SourceCandidate::value(
            identity("mode"),
            SourceLocator::Custom("high-mode".to_owned()),
            json!("b"),
        )],
    )
    .expect("high source");

    let result = Resolver::resolve(&fields, &[low, high]).expect("valid input");
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
        &vec![
            SourceLocator::Custom("low-mode".to_owned()),
            SourceLocator::Custom("high-mode".to_owned()),
        ]
    );
}

#[test]
fn map_merge_preserves_source_order() {
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

    let values = Resolver::resolve(&fields, &[low, high])
        .expect("valid input")
        .materialize()
        .expect("valid map merge");
    assert_eq!(
        values[&identity("map")],
        TypedValue::Object(
            serde_json::from_value(json!({"same": "high", "low": true, "high": true}))
                .expect("object")
        )
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

    let values = Resolver::resolve(&fields, &[low, high])
        .expect("valid input")
        .materialize()
        .expect("equal values do not conflict");
    assert_eq!(
        values[&identity("mode")],
        TypedValue::String("same".to_owned())
    );
}
