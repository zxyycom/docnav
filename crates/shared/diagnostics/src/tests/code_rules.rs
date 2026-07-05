use std::collections::BTreeSet;

use crate::{BoundaryDiagnosticCode, DiagnosticCode, ProtocolDiagnosticCode};

// @case WB-DIAG-RULES-001
#[test]
fn diagnostic_code_rules_cover_each_variant() {
    let codes: Vec<_> = DiagnosticCode::all().collect();
    assert!(codes.contains(&ProtocolDiagnosticCode::InvalidRequest.into()));
    assert!(codes.contains(&BoundaryDiagnosticCode::RequestSchemaValidationFailed.into()));

    let mut names = BTreeSet::new();
    for code in codes {
        assert!(!code.as_str().is_empty());
        assert!(
            names.insert(code.as_str()),
            "duplicate code {}",
            code.as_str()
        );
        assert!(!code.details_rule().fields().is_empty(), "{code:?}");
        let projection = code.projection_rule();
        assert!(projection.stderr || projection.protocol_code.is_some());
        let _ = code.category();
        let _ = code.default_severity();
        let _ = code.default_effect();
    }
}
