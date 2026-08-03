use crate::{
    typed_codes, DiagnosticEffect, DiagnosticRecordDraft, DiagnosticRecordError,
    DiagnosticSeverity, DiagnosticSource, FieldReasonDetails, FormatUnknownDetails,
    ProtocolDiagnosticCode,
};

#[test]
fn diagnostic_record_finalization_enforces_summary_and_code_defaults() {
    let source = DiagnosticSource::with_stage("docnav", "argv");
    let record = DiagnosticRecordDraft::new::<typed_codes::protocol::InvalidRequest>(
        "invalid input",
        FieldReasonDetails::new("argv", "unknown argument --future"),
        source.clone(),
    )
    .into_record()
    .unwrap();

    assert_eq!(record.code(), ProtocolDiagnosticCode::InvalidRequest.into());
    assert_eq!(record.severity(), DiagnosticSeverity::Error);
    assert_eq!(record.effect(), DiagnosticEffect::InputRejected);
    assert_eq!(record.source(), &source);
    assert!(record.guidance().is_none());
    assert!(!record.recoverable());

    let invalid = DiagnosticRecordDraft::new::<typed_codes::protocol::InvalidRequest>(
        "",
        FieldReasonDetails::new("argv", "unknown argument --future"),
        DiagnosticSource::with_stage("docnav", "argv"),
    );

    assert_eq!(
        invalid.into_record(),
        Err(DiagnosticRecordError::EmptySummary)
    );
}

#[test]
fn format_unknown_record_has_exact_routing_details() {
    let record = DiagnosticRecordDraft::new::<typed_codes::protocol::FormatUnknown>(
        "format unknown",
        FormatUnknownDetails::new("docs/unknown.data"),
        DiagnosticSource::with_stage("docnav-navigation", "routing"),
    )
    .into_record()
    .unwrap();

    assert_eq!(
        record.details().to_value(),
        serde_json::json!({
            "path": "docs/unknown.data",
            "reason": "FORMAT_NOT_RECOGNIZED",
            "candidates": []
        })
    );
}
