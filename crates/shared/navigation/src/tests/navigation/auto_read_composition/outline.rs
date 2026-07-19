use serde_json::json;

use super::*;

#[test]
fn unique_outline_ref_composes_read_with_the_selected_document_context() {
    let read_result = ReadResult {
        ref_id: "opaque:section".to_owned(),
        content: "selected content".to_owned(),
        content_type: "text/markdown".to_owned(),
        cost: Cost {
            measurements: Vec::new(),
        },
        page: positive(2),
    };
    let adapter = RecordingAdapter::new(
        OutlineResult::structured(vec![entry("opaque:section", "Selected")], positive(7)),
        Some(read_result.clone()),
    );
    let mut command = command(
        docnav_protocol::Operation::Outline,
        vec![
            cli_value_candidate("docnav.document.page", "--page", json!(3)),
            cli_value_candidate("docnav.defaults.pagination.limit", "--limit", json!(321)),
        ],
    );
    command.document_path = "docs/normalized.stub".to_owned();

    let outcome = execute_loaded_navigation_command(
        command,
        config_sources(json!({}), json!({})),
        &document_parameter_catalog(),
        &SingleRegistry::new(&adapter),
    )
    .expect("navigation success");

    assert_eq!(
        adapter.read_inputs(),
        vec![ReadInput {
            document_path: "docs/normalized.stub".to_owned(),
            ref_id: "opaque:section".to_owned(),
            page: positive(1).unwrap(),
            limit: positive(321).unwrap(),
        }]
    );
    let ProtocolResponse::Success(success) = outcome.response else {
        panic!("expected success response");
    };
    assert_eq!(success.operation, docnav_protocol::Operation::Outline);
    let OperationResult::Outline(OutlineResult::Structured(result)) = success.result else {
        panic!("expected structured outline result");
    };
    assert_eq!(result.entries, vec![entry("opaque:section", "Selected")]);
    assert_eq!(result.page, positive(7));
    let auto_read = result.auto_read.expect("successful auto-read");
    assert_eq!(auto_read.reason, AutoReadReason::UniqueRef);
    assert_eq!(auto_read.read, read_result);
}

#[test]
fn outline_eligibility_keeps_non_unique_or_unstructured_base_results() {
    let cases = [
        OutlineResult::structured(Vec::new(), None),
        OutlineResult::structured(
            vec![entry("opaque:a", "A"), entry("opaque:b", "B")],
            positive(4),
        ),
        OutlineResult::unstructured(
            UnstructuredOutlineReason::PathRule,
            "full content",
            "text/plain",
            empty_cost(),
        ),
    ];

    for base_result in cases {
        let adapter = RecordingAdapter::new(base_result.clone(), Some(read_result("unused")));
        let outcome = execute(
            &adapter,
            command(docnav_protocol::Operation::Outline, Vec::new()),
        );

        assert_eq!(adapter.read_inputs(), Vec::new());
        assert_eq!(outline_result(outcome.response), base_result);
    }
}

#[test]
fn nested_read_diagnostic_silently_keeps_the_validated_base_result() {
    let base_result =
        OutlineResult::structured(vec![entry("opaque:missing", "Missing")], positive(6));
    let adapter = RecordingAdapter::new(base_result.clone(), None);

    let outcome = execute(
        &adapter,
        command(docnav_protocol::Operation::Outline, Vec::new()),
    );

    assert_eq!(adapter.read_inputs().len(), 1);
    assert_eq!(outline_result(outcome.response), base_result);
}

#[test]
fn disabled_mode_does_not_dispatch_nested_read() {
    let base_result = OutlineResult::structured(vec![entry("opaque:a", "A")], positive(2));
    let adapter = RecordingAdapter::new(base_result.clone(), Some(read_result("opaque:a")));
    let command = command(
        docnav_protocol::Operation::Outline,
        vec![cli_value_candidate(
            "docnav.defaults.auto_read",
            "--auto-read",
            json!("disabled"),
        )],
    );

    let outcome = execute(&adapter, command);

    assert_eq!(adapter.read_inputs(), Vec::new());
    assert_eq!(outline_result(outcome.response), base_result);
}

#[test]
fn adapter_base_result_with_auto_read_is_rejected_before_composition() {
    let mut base_result = OutlineResult::structured(vec![entry("opaque:a", "A")], positive(2));
    let OutlineResult::Structured(result) = &mut base_result else {
        panic!("expected structured outline result");
    };
    result.auto_read = Some(AutoReadResult::unique_ref(read_result("opaque:a")));
    let adapter = RecordingAdapter::new(base_result, Some(read_result("opaque:a")));
    let command = command(
        docnav_protocol::Operation::Outline,
        vec![cli_value_candidate(
            "docnav.defaults.auto_read",
            "--auto-read",
            json!("disabled"),
        )],
    );

    let error = execute_loaded_navigation_command(
        command,
        config_sources(json!({}), json!({})),
        &document_parameter_catalog(),
        &SingleRegistry::new(&adapter),
    )
    .expect_err("adapter-authored auto_read must violate result ownership");

    assert_eq!(
        error.failure_layer(),
        Some(NavigationFailureLayer::ResultValidation)
    );
    assert_eq!(adapter.read_inputs(), Vec::new());
}

#[test]
fn nested_read_reuses_the_effective_limit_when_pagination_is_disabled() {
    let adapter = RecordingAdapter::new(
        OutlineResult::structured(vec![entry("opaque:a", "A")], None),
        Some(read_result("opaque:a")),
    );
    let command = command(
        docnav_protocol::Operation::Outline,
        vec![
            cli_value_candidate("docnav.defaults.pagination.limit", "--limit", json!(123)),
            cli_value_candidate(
                "docnav.defaults.pagination.enabled",
                "--pagination",
                json!(false),
            ),
        ],
    );

    execute(&adapter, command);

    assert_eq!(adapter.read_inputs()[0].limit, positive(u32::MAX).unwrap(),);
}
