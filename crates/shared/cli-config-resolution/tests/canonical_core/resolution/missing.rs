use cli_config_resolution::{
    resolve, DiagnosticReason, ExpectedFieldShape, FieldDef, FieldDefSet, FieldValidation,
    ProcessStrategy, TypedValue,
};
use serde_json::json;

use crate::support::{candidate, identity, source};

// Proves: one missing required field blocks materialization even when a peer
// field has already resolved successfully, so no partial FieldValueMap escapes.
#[test]
fn missing_required_value_returns_no_partial_values() {
    let fields = FieldDefSet::builder()
        .field(
            FieldDef::builder("required")
                .process("custom", ProcessStrategy::rust_field())
                .validation(FieldValidation::string()),
            ExpectedFieldShape::required(),
        )
        .field(
            FieldDef::builder("available")
                .process("custom", ProcessStrategy::rust_field())
                .validation(FieldValidation::string()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("field set");
    let available = source("available", 10, [candidate("available", json!("resolved"))]);
    let result = resolve(&fields, &[available]).expect("valid resolver input");
    assert_eq!(
        result.fields()[&identity("available")].value(),
        Some(&TypedValue::String("resolved".to_owned()))
    );

    let error = result.materialize().expect_err("required value is missing");
    assert!(error.diagnostics().iter().any(|diagnostic| {
        diagnostic.field.as_str() == "required"
            && matches!(diagnostic.reason, DiagnosticReason::MissingRequired(_))
    }));
    assert!(
        result
            .trace(&identity("required"))
            .expect("trace")
            .missing_required
    );
}
