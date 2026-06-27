use crate::{
    DiagnosticCode, DiagnosticDetails, DiagnosticEffect, DiagnosticRecordDraft,
    DiagnosticRecordError, DiagnosticSeverity, DiagnosticSource, DiagnosticStack,
    ProtocolDiagnosticCode, ReadableWarningDiagnosticCode,
};

#[test]
fn diagnostic_record_validates_details_and_uses_code_defaults() {
    let draft = DiagnosticRecordDraft::new(
        ReadableWarningDiagnosticCode::CliArgvIgnored,
        "ignored unused argv",
        DiagnosticDetails::CliArgv {
            tokens: vec!["--future".to_owned()],
        },
        DiagnosticSource::with_stage("docnav", "argv"),
    );
    let mut stack = DiagnosticStack::new();
    let id = stack.push(draft).unwrap();
    let record = stack.get(id).unwrap();

    assert_eq!(
        record.code,
        ReadableWarningDiagnosticCode::CliArgvIgnored.into()
    );
    assert_eq!(record.severity, DiagnosticSeverity::Warning);
    assert_eq!(record.effect, DiagnosticEffect::OperationContinued);
    assert!(record.guidance.is_none());
    assert!(record.recoverable);

    let invalid = DiagnosticRecordDraft::new(
        ProtocolDiagnosticCode::InvalidRequest,
        "invalid request",
        DiagnosticDetails::Path {
            path: "wrong-shape".to_owned(),
        },
        DiagnosticSource::new("protocol"),
    );
    assert!(matches!(
        stack.push(invalid),
        Err(DiagnosticRecordError::InvalidDetails(_))
    ));
}

#[test]
fn diagnostic_stack_id_mark_drain_and_snapshot_are_lifo() {
    let mut stack = DiagnosticStack::new();
    let first = stack
        .push(draft(
            ReadableWarningDiagnosticCode::CliArgvIgnored,
            "ignored argv",
            DiagnosticDetails::CliArgv {
                tokens: vec!["--future".to_owned()],
            },
        ))
        .unwrap();
    let mark = stack.mark();
    let second = stack
        .push(draft(
            ReadableWarningDiagnosticCode::AdapterConfigSourceSkipped,
            "config skipped",
            DiagnosticDetails::AdapterConfigSource {
                source_level: "project".to_owned(),
                path_origin: "override".to_owned(),
                path: "missing.json".to_owned(),
                reason_code: "missing_override".to_owned(),
            },
        ))
        .unwrap();
    let third = stack
        .push(draft(
            ReadableWarningDiagnosticCode::AdapterCandidateFailure,
            "candidate skipped",
            DiagnosticDetails::AdapterCandidate {
                adapter_id: "markdown".to_owned(),
                stage: "probe".to_owned(),
                code: "UNSUPPORTED".to_owned(),
                preselected: None,
            },
        ))
        .unwrap();

    assert_eq!(stack.peek_recent().unwrap().id, third);
    assert_eq!(stack.snapshot()[0].id, third);
    assert_eq!(stack.snapshot()[1].id, second);
    assert_eq!(stack.snapshot()[2].id, first);

    let drained = stack.drain_after(mark);
    assert_eq!(
        drained.iter().map(|record| record.id).collect::<Vec<_>>(),
        vec![third, second]
    );
    assert_eq!(stack.len(), 1);
    assert!(stack.get(first).is_some());
}

#[test]
fn diagnostic_stack_drains_after_event_with_optional_anchor() {
    let mut stack = DiagnosticStack::new();
    let first = stack
        .push(draft(
            ReadableWarningDiagnosticCode::CliArgvIgnored,
            "first",
            DiagnosticDetails::CliArgv {
                tokens: vec!["--one".to_owned()],
            },
        ))
        .unwrap();
    let anchor = stack
        .push(draft(
            ReadableWarningDiagnosticCode::CliArgvIgnored,
            "anchor",
            DiagnosticDetails::CliArgv {
                tokens: vec!["--two".to_owned()],
            },
        ))
        .unwrap();
    let after = stack
        .push(draft(
            ReadableWarningDiagnosticCode::CliArgvIgnored,
            "after",
            DiagnosticDetails::CliArgv {
                tokens: vec!["--three".to_owned()],
            },
        ))
        .unwrap();

    let drained = stack.drain_after_event(anchor, false);
    assert_eq!(
        drained.iter().map(|record| record.id).collect::<Vec<_>>(),
        vec![after]
    );
    assert!(stack.get(first).is_some());
    assert!(stack.get(anchor).is_some());

    let later = stack
        .push(draft(
            ReadableWarningDiagnosticCode::CliArgvIgnored,
            "later",
            DiagnosticDetails::CliArgv {
                tokens: vec!["--four".to_owned()],
            },
        ))
        .unwrap();
    let drained = stack.drain_after_event(anchor, true);
    assert_eq!(
        drained.iter().map(|record| record.id).collect::<Vec<_>>(),
        vec![later, anchor]
    );
    assert!(stack.get(first).is_some());
    assert!(stack.get(anchor).is_none());
}

fn draft(
    code: impl Into<DiagnosticCode>,
    summary: &str,
    details: DiagnosticDetails,
) -> DiagnosticRecordDraft {
    DiagnosticRecordDraft::new(code, summary, details, DiagnosticSource::new("test"))
}
