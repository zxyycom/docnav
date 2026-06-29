use super::common::{positive, StubAdapter};
use crate::execute_operation;
use docnav_protocol::{
    Document, Operation, OperationArguments, OperationResult, OutlineArguments,
    ProtocolDiagnosticCode, RequestEnvelope, PROTOCOL_VERSION,
};

// @case WB-SDK-EXECUTE-001
#[test]
fn execute_operation_dispatches_typed_request() {
    let request = RequestEnvelope {
        protocol_version: PROTOCOL_VERSION.to_owned(),
        request_id: "req-1".to_owned(),
        operation: Operation::Outline,
        document: Document {
            path: "sample.stub".to_owned(),
        },
        arguments: OperationArguments::Outline(OutlineArguments {
            limit: positive(80),
            page: positive(1),
            options: None,
        }),
    };

    let result = execute_operation(&StubAdapter, &request).expect("execute outline");

    match result {
        OperationResult::Outline(result) => {
            assert_eq!(result.entries[0].ref_id, "L1:Stub");
            assert_eq!(result.page, None);
        }
        other => panic!("expected outline result, got {other:?}"),
    }
}

#[test]
fn execute_operation_rejects_mismatched_operation_arguments() {
    let request = RequestEnvelope {
        protocol_version: PROTOCOL_VERSION.to_owned(),
        request_id: "req-1".to_owned(),
        operation: Operation::Read,
        document: Document {
            path: "sample.stub".to_owned(),
        },
        arguments: OperationArguments::Outline(OutlineArguments {
            limit: positive(80),
            page: positive(1),
            options: None,
        }),
    };

    let error = execute_operation(&StubAdapter, &request).expect_err("mismatch fails");
    let stable = error.protocol_error();

    assert_eq!(stable.code(), ProtocolDiagnosticCode::InvalidRequest);
    assert_eq!(stable.details()["field"], "arguments");
    assert_eq!(
        stable.details()["reason"],
        "arguments do not match operation read"
    );
}
