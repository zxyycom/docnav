use crate::{
    typed_codes, DiagnosticDetails, DiagnosticEffect, DiagnosticRecordDraft, DiagnosticRecordError,
    DiagnosticSeverity, DiagnosticSource, FieldReasonDetails, FormatCandidateDetails,
    FormatUnknownDetails, ProtocolDiagnosticCode,
};

// @case WB-DIAG-RECORD-001
#[test]
fn diagnostic_record_validates_details_and_uses_code_defaults() {
    let source = DiagnosticSource::with_stage("docnav", "argv");
    let record = DiagnosticRecordDraft::new::<typed_codes::protocol::InvalidRequest>(
        "invalid input",
        FieldReasonDetails::new("argv", "unknown argument --future"),
        source.clone(),
    )
    .into_record()
    .unwrap();

    assert_eq!(record.id().get(), 0);
    assert_eq!(record.code(), ProtocolDiagnosticCode::InvalidRequest.into());
    assert_eq!(record.severity(), DiagnosticSeverity::Error);
    assert_eq!(record.effect(), DiagnosticEffect::InputRejected);
    assert_eq!(record.source(), &source);
    assert!(record.guidance().is_none());
    assert!(!record.recoverable());

    let invalid = DiagnosticRecordDraft::from_erased_for_test(
        ProtocolDiagnosticCode::InvalidRequest,
        "invalid request",
        DiagnosticDetails::Path {
            path: "wrong-shape".to_owned(),
        },
        DiagnosticSource::new("protocol"),
    );
    assert!(matches!(
        invalid.into_record(),
        Err(DiagnosticRecordError::InvalidDetails(_))
    ));
}

#[test]
fn diagnostic_record_rejects_empty_summary() {
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
fn format_record_accepts_candidate_failures() {
    let candidate = FormatCandidateDetails::new("docnav-markdown", "probe", "PROBE_UNSUPPORTED");
    let record = DiagnosticRecordDraft::new::<typed_codes::protocol::FormatUnknown>(
        "format unknown",
        FormatUnknownDetails::new(
            "docs/unknown.data",
            "NO_SUPPORTED_ADAPTER",
            vec![candidate.clone()],
        )
        .with_candidate_failures(vec![candidate]),
        DiagnosticSource::with_stage("docnav-navigation", "routing"),
    )
    .into_record()
    .unwrap();

    assert_eq!(
        record.details().to_value()["candidate_failures"][0]["reason"],
        "PROBE_UNSUPPORTED"
    );
}
