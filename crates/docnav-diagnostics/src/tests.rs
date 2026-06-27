// @case WB-DIAG-WARN-001
use super::*;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

#[test]
fn warning_ids_serialize_as_stable_strings() {
    assert_eq!(CLI_ARGV_IGNORED.as_str(), "cli_argv_ignored");
    assert_eq!(
        ADAPTER_CANDIDATE_FAILURE.as_str(),
        "adapter_candidate_failure"
    );
    assert_eq!(
        ADAPTER_CONFIG_SOURCE_SKIPPED.as_str(),
        "adapter_config_source_skipped"
    );
    assert_eq!(
        serde_json::to_value(WarningId::new("adapter_owned").unwrap()).unwrap(),
        json!("adapter_owned")
    );
    assert!(WarningId::new("AdapterOwned").is_err());
}

#[test]
fn argv_warning_constructors_keep_existing_shape() {
    let warning = Warning::unused_operation_flag("--page", Some("nope"), "info");
    assert_eq!(warning.id, CLI_ARGV_IGNORED);
    assert_eq!(warning.effect, WarningEffect::OperationContinued);
    assert_eq!(warning.reason, "flag is not used by info command");
    assert_eq!(
        serde_json::to_value(warning.details).unwrap(),
        json!({"tokens": ["--page", "nope"]})
    );
}

#[test]
fn adapter_candidate_warning_keeps_existing_shape() {
    let warning =
        Warning::adapter_candidate_failure("markdown", "probe", "UNSUPPORTED", "no match", true);
    assert_eq!(warning.id, ADAPTER_CANDIDATE_FAILURE);
    assert_eq!(warning.effect, WarningEffect::CandidateSkipped);
    assert_eq!(
        serde_json::to_value(warning.details).unwrap(),
        json!({
            "adapter_id": "markdown",
            "stage": "probe",
            "code": "UNSUPPORTED",
            "preselected": true
        })
    );
}

#[test]
fn adapter_config_source_warning_keeps_stable_shape() {
    let warning = Warning::adapter_config_source_skipped(
        "project",
        "override",
        "D:\\project\\.docnav\\docnav-markdown.json",
        "invalid_json",
    );

    assert_eq!(warning.id, ADAPTER_CONFIG_SOURCE_SKIPPED);
    assert_eq!(warning.effect, WarningEffect::OperationContinued);
    assert_eq!(
        serde_json::to_value(warning.details).unwrap(),
        json!({
            "source_level": "project",
            "path_origin": "override",
            "path": "D:\\project\\.docnav\\docnav-markdown.json",
            "reason_code": "invalid_json"
        })
    );
}

#[test]
fn known_warnings_roundtrip_through_diagnostic_records() {
    let warnings = vec![
        Warning::cli_argv_ignored(vec!["--future".to_owned()], "unknown CLI flag ignored"),
        Warning::adapter_candidate_failure("markdown", "probe", "UNSUPPORTED", "no match", true),
        Warning::adapter_config_source_skipped(
            "project",
            "override",
            "missing.json",
            "missing_override",
        ),
    ];

    for warning in warnings {
        let draft = warning
            .to_record_draft(DiagnosticSource::with_stage("docnav", "test"))
            .unwrap();
        let mut stack = DiagnosticStack::new();
        let id = stack.push(draft).unwrap();
        let record = stack.get(id).unwrap();

        assert_eq!(Warning::from_record(record), Some(warning));
    }
}

#[test]
fn warning_text_line_matches_stderr_contract() {
    let cases = [
        (
            Warning::cli_argv_ignored(vec!["--future".to_owned()], "unknown\nCLI flag ignored"),
            "warning: id=cli_argv_ignored, effect=operation_continued, reason=unknown CLI flag ignored, details={\"tokens\":[\"--future\"]}",
        ),
        (
            Warning::adapter_config_source_skipped(
                "project",
                "override",
                "missing.json",
                "missing_override",
            ),
            "warning: id=adapter_config_source_skipped, effect=operation_continued, reason=adapter config source skipped, details={\"source_level\":\"project\",\"path_origin\":\"override\",\"path\":\"missing.json\",\"reason_code\":\"missing_override\"}",
        ),
    ];

    for (warning, expected) in cases {
        assert_eq!(warning_text_line(&warning).unwrap(), expected);
    }
}

#[test]
fn attach_warnings_keeps_json_payload_shape() {
    let warning = Warning::cli_argv_ignored(vec!["--future".to_owned()], "test warning");

    let object = attach_warnings_to_value(json!({"ok": true}), std::slice::from_ref(&warning));
    assert_eq!(object["ok"], json!(true));
    assert_eq!(object["warnings"][0]["id"], "cli_argv_ignored");

    let scalar = attach_warnings_to_value(json!("ok"), &[warning]);
    assert_eq!(scalar["value"], json!("ok"));
    assert_eq!(scalar["warnings"][0]["id"], "cli_argv_ignored");
}

#[test]
fn diagnostic_code_rules_cover_each_variant() {
    let codes: Vec<_> = DiagnosticCode::all().collect();
    assert!(codes.contains(&ProtocolDiagnosticCode::InvalidRequest.into()));
    assert!(codes.contains(&ReadableWarningDiagnosticCode::CliArgvIgnored.into()));
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
        assert!(
            projection.stderr
                || projection.protocol_code.is_some()
                || projection.readable_warning_id.is_some()
        );
        let _ = code.category();
        let _ = code.default_severity();
        let _ = code.default_effect();
    }
}

#[test]
fn detail_rules_reject_missing_wrong_and_extra_fields_for_each_code() {
    for code in DiagnosticCode::all() {
        let rule = code.details_rule();
        let required = rule
            .fields()
            .iter()
            .find(|field| field.required)
            .expect("each diagnostic code has at least one required details field");
        let valid = valid_details_for(rule);
        assert!(
            rule.validate_value(&Value::Object(valid.clone())).is_ok(),
            "{code:?}"
        );

        let mut missing = valid.clone();
        missing.remove(required.name);
        assert!(
            matches!(
                rule.validate_value(&Value::Object(missing)),
                Err(DiagnosticDetailsError::MissingField { field }) if field == required.name
            ),
            "{code:?}"
        );

        if required.kind != DetailFieldType::Any {
            let mut wrong = valid.clone();
            wrong.insert(required.name.to_owned(), wrong_value_for(required.kind));
            assert!(
                matches!(
                    rule.validate_value(&Value::Object(wrong)),
                    Err(DiagnosticDetailsError::WrongType { field, expected })
                        if field == required.name && expected == required.kind
                ),
                "{code:?}"
            );
        }

        let mut extra = valid;
        extra.insert("extra".to_owned(), json!(true));
        assert!(
            matches!(
                rule.validate_value(&Value::Object(extra)),
                Err(DiagnosticDetailsError::ExtraField { field }) if field == "extra"
            ),
            "{code:?}"
        );
    }
}

#[test]
fn invalid_request_details_accept_known_optional_context_fields() {
    let rule = DiagnosticCode::from(ProtocolDiagnosticCode::InvalidRequest).details_rule();
    let valid = json!({
        "field": "defaults.output",
        "reason": "invalid output mode",
        "path": ".docnav/docnav.json",
        "received": "text",
        "accepted": ["readable-view", "readable-json", "protocol-json"]
    });

    assert!(rule.validate_value(&valid).is_ok());

    let wrong_accepted = json!({
        "field": "defaults.output",
        "reason": "invalid output mode",
        "accepted": "readable-view"
    });
    assert!(matches!(
        rule.validate_value(&wrong_accepted),
        Err(DiagnosticDetailsError::WrongType { field, expected })
            if field == "accepted" && expected == DetailFieldType::StringArray
    ));
}

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

fn valid_details_for(rule: DiagnosticDetailsRule) -> Map<String, Value> {
    rule.fields()
        .iter()
        .filter(|field| field.required)
        .map(|field| (field.name.to_owned(), value_for(field.kind)))
        .collect()
}

fn value_for(kind: DetailFieldType) -> Value {
    match kind {
        DetailFieldType::String => json!("value"),
        DetailFieldType::StringArray => json!(["value"]),
        DetailFieldType::Boolean => json!(true),
        DetailFieldType::U32 => json!(1),
        DetailFieldType::I32 => json!(-1),
        DetailFieldType::Object => json!({}),
        DetailFieldType::Any => json!({"any": true}),
    }
}

fn wrong_value_for(kind: DetailFieldType) -> Value {
    match kind {
        DetailFieldType::String => json!(1),
        DetailFieldType::StringArray => json!("value"),
        DetailFieldType::Boolean => json!("true"),
        DetailFieldType::U32 => json!(-1),
        DetailFieldType::I32 => json!("1"),
        DetailFieldType::Object => json!("object"),
        DetailFieldType::Any => json!(null),
    }
}
