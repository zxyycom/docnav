use serde_json::json;

use super::*;

#[test]
fn repeated_find_refs_dispatch_one_read_on_later_pages_with_continuation() {
    let read_result = ReadResult {
        page: positive(3),
        ..read_result("opaque:find")
    };
    let base_result = FindResult::new(
        vec![
            entry("opaque:find", "First match"),
            entry("opaque:find", "Second match"),
        ],
        positive(8),
    );
    let adapter = RecordingAdapter::new(
        OutlineResult::structured(Vec::new(), None),
        Some(read_result.clone()),
    )
    .with_find_result(base_result.clone());
    let mut command = command(
        docnav_protocol::Operation::Find,
        vec![
            cli_value_candidate("docnav.document.page", "--page", json!(4)),
            cli_value_candidate("docnav.defaults.pagination.limit", "--limit", json!(222)),
        ],
    );
    command.document_path = "docs/find.stub".to_owned();

    let outcome = execute(&adapter, command);

    assert_eq!(
        adapter.read_inputs(),
        vec![ReadInput {
            document_path: "docs/find.stub".to_owned(),
            ref_id: "opaque:find".to_owned(),
            page: positive(1).unwrap(),
            limit: positive(222).unwrap(),
        }]
    );
    let ProtocolResponse::Success(success) = outcome.response else {
        panic!("expected success response");
    };
    assert_eq!(success.operation, docnav_protocol::Operation::Find);
    let OperationResult::Find(result) = success.result else {
        panic!("expected find result");
    };
    assert_eq!(result.matches, base_result.matches);
    assert_eq!(result.page, positive(8));
    let auto_read = result.auto_read.expect("successful auto-read");
    assert_eq!(auto_read.reason, AutoReadReason::UniqueRef);
    assert_eq!(auto_read.read, read_result);
}

#[test]
fn find_eligibility_keeps_empty_or_multiple_ref_base_results() {
    let cases = [
        FindResult::new(Vec::new(), None),
        FindResult::new(
            vec![entry("opaque:a", "A"), entry("opaque:b", "B")],
            positive(5),
        ),
    ];

    for base_result in cases {
        let adapter = RecordingAdapter::new(
            OutlineResult::structured(Vec::new(), None),
            Some(read_result("unused")),
        )
        .with_find_result(base_result.clone());
        let outcome = execute(
            &adapter,
            command(docnav_protocol::Operation::Find, Vec::new()),
        );

        assert_eq!(adapter.read_inputs(), Vec::new());
        assert_eq!(find_result(outcome.response), base_result);
    }
}
