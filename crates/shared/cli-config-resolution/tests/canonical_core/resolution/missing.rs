use cli_config_resolution::{resolve, DiagnosticReason};

use crate::support::{custom_field_set, identity};

#[test]
fn missing_required_value_returns_no_partial_values() {
    let fields = custom_field_set("required", true);
    let result = resolve(&fields, &[]).expect("valid resolver input");
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
