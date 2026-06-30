use crate::{
    typed_codes, BoundaryDetails, DiagnosticDetails, DiagnosticEffect, DiagnosticRecordDraft,
    DiagnosticRecordError, DiagnosticSeverity, DiagnosticSource, DiagnosticStack,
    FieldReasonDetails, ProtocolDiagnosticCode,
};

#[test]
fn diagnostic_record_validates_details_and_uses_code_defaults() {
    let draft = DiagnosticRecordDraft::new::<typed_codes::protocol::InvalidRequest>(
        "invalid input",
        FieldReasonDetails::new("argv", "unknown argument --future"),
        DiagnosticSource::with_stage("docnav", "argv"),
    );
    let mut stack = DiagnosticStack::new();
    let id = stack.push(draft).unwrap();
    let record = stack.get(id).unwrap();

    assert_eq!(record.code(), ProtocolDiagnosticCode::InvalidRequest.into());
    assert_eq!(record.severity(), DiagnosticSeverity::Error);
    assert_eq!(record.effect(), DiagnosticEffect::InputRejected);
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
        stack.push(invalid),
        Err(DiagnosticRecordError::InvalidDetails(_))
    ));
}

#[test]
fn diagnostic_stack_id_mark_drain_and_snapshot_are_lifo() {
    let mut stack = DiagnosticStack::new();
    let first = stack
        .push(invalid_request_draft("invalid argv", "--future"))
        .unwrap();
    let mark = stack.mark();
    let second = stack.push(boundary_draft("config failed")).unwrap();
    let third = stack.push(boundary_draft("candidate failed")).unwrap();

    assert_eq!(stack.peek_recent().unwrap().id(), third);
    assert_eq!(stack.snapshot()[0].id(), third);
    assert_eq!(stack.snapshot()[1].id(), second);
    assert_eq!(stack.snapshot()[2].id(), first);

    let drained = stack.drain_after(mark);
    assert_eq!(
        drained.iter().map(|record| record.id()).collect::<Vec<_>>(),
        vec![third, second]
    );
    assert_eq!(stack.len(), 1);
    assert!(stack.get(first).is_some());
}

#[test]
fn diagnostic_stack_drains_after_event_with_optional_anchor() {
    let mut stack = DiagnosticStack::new();
    let first = stack.push(invalid_request_draft("first", "--one")).unwrap();
    let anchor = stack
        .push(invalid_request_draft("anchor", "--two"))
        .unwrap();
    let after = stack
        .push(invalid_request_draft("after", "--three"))
        .unwrap();

    let drained = stack.drain_after_event(anchor, false);
    assert_eq!(
        drained.iter().map(|record| record.id()).collect::<Vec<_>>(),
        vec![after]
    );
    assert!(stack.get(first).is_some());
    assert!(stack.get(anchor).is_some());

    let later = stack
        .push(invalid_request_draft("later", "--four"))
        .unwrap();
    let drained = stack.drain_after_event(anchor, true);
    assert_eq!(
        drained.iter().map(|record| record.id()).collect::<Vec<_>>(),
        vec![later, anchor]
    );
    assert!(stack.get(first).is_some());
    assert!(stack.get(anchor).is_none());
}

fn invalid_request_draft(summary: &str, token: &str) -> DiagnosticRecordDraft {
    DiagnosticRecordDraft::new::<typed_codes::protocol::InvalidRequest>(
        summary,
        FieldReasonDetails::new("argv", format!("unexpected token {token}")),
        DiagnosticSource::new("test"),
    )
}

fn boundary_draft(summary: &str) -> DiagnosticRecordDraft {
    DiagnosticRecordDraft::new::<typed_codes::boundary::FailedToWriteJson>(
        summary,
        BoundaryDetails::new("stdout closed"),
        DiagnosticSource::new("test"),
    )
}
