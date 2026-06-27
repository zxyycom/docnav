use crate::{
    typed_codes, AdapterCandidateDetails, AdapterConfigSourceDetails, CliArgvDetails,
    DiagnosticDetails, DiagnosticEffect, DiagnosticRecordDraft, DiagnosticRecordError,
    DiagnosticSeverity, DiagnosticSource, DiagnosticStack, ProtocolDiagnosticCode,
    ReadableWarningDiagnosticCode,
};

#[test]
fn diagnostic_record_validates_details_and_uses_code_defaults() {
    let draft = DiagnosticRecordDraft::new::<typed_codes::readable_warning::CliArgvIgnored>(
        "ignored unused argv",
        CliArgvDetails::new(vec!["--future".to_owned()]),
        DiagnosticSource::with_stage("docnav", "argv"),
    );
    let mut stack = DiagnosticStack::new();
    let id = stack.push(draft).unwrap();
    let record = stack.get(id).unwrap();

    assert_eq!(
        record.code(),
        ReadableWarningDiagnosticCode::CliArgvIgnored.into()
    );
    assert_eq!(record.severity(), DiagnosticSeverity::Warning);
    assert_eq!(record.effect(), DiagnosticEffect::OperationContinued);
    assert!(record.guidance().is_none());
    assert!(record.recoverable());

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
        .push(cli_argv_draft("ignored argv", "--future"))
        .unwrap();
    let mark = stack.mark();
    let second = stack
        .push(adapter_config_source_draft("config skipped"))
        .unwrap();
    let third = stack
        .push(adapter_candidate_draft("candidate skipped"))
        .unwrap();

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
    let first = stack.push(cli_argv_draft("first", "--one")).unwrap();
    let anchor = stack.push(cli_argv_draft("anchor", "--two")).unwrap();
    let after = stack.push(cli_argv_draft("after", "--three")).unwrap();

    let drained = stack.drain_after_event(anchor, false);
    assert_eq!(
        drained.iter().map(|record| record.id()).collect::<Vec<_>>(),
        vec![after]
    );
    assert!(stack.get(first).is_some());
    assert!(stack.get(anchor).is_some());

    let later = stack.push(cli_argv_draft("later", "--four")).unwrap();
    let drained = stack.drain_after_event(anchor, true);
    assert_eq!(
        drained.iter().map(|record| record.id()).collect::<Vec<_>>(),
        vec![later, anchor]
    );
    assert!(stack.get(first).is_some());
    assert!(stack.get(anchor).is_none());
}

fn cli_argv_draft(summary: &str, token: &str) -> DiagnosticRecordDraft {
    DiagnosticRecordDraft::new::<typed_codes::readable_warning::CliArgvIgnored>(
        summary,
        CliArgvDetails::new(vec![token.to_owned()]),
        DiagnosticSource::new("test"),
    )
}

fn adapter_config_source_draft(summary: &str) -> DiagnosticRecordDraft {
    DiagnosticRecordDraft::new::<typed_codes::readable_warning::AdapterConfigSourceSkipped>(
        summary,
        AdapterConfigSourceDetails::new("project", "override", "missing.json", "missing_override"),
        DiagnosticSource::new("test"),
    )
}

fn adapter_candidate_draft(summary: &str) -> DiagnosticRecordDraft {
    DiagnosticRecordDraft::new::<typed_codes::readable_warning::AdapterCandidateFailure>(
        summary,
        AdapterCandidateDetails::new("markdown", "probe", "UNSUPPORTED", None),
        DiagnosticSource::new("test"),
    )
}
