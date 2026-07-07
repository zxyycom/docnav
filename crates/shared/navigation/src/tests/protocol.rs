use docnav_protocol::{
    positive_result, Operation, OperationArguments, OperationResult, OutlineResult,
    ProtocolResponse, SuccessResponse, PROTOCOL_VERSION,
};
use serde_json::Value;

use crate::{
    protocol_request, validate_navigation_response, NavigationFailureLayer,
    NavigationInvocationTrace, OperationInput,
};

#[test]
fn protocol_request_maps_core_inputs_to_operation_arguments() {
    let request = protocol_request(OperationInput {
        operation: Operation::Outline,
        document_path: "docs/guide.md".to_owned(),
        ref_id: None,
        query: None,
        page: Some(positive_result(1).unwrap()),
        limit: Some(positive_result(80).unwrap()),
        options: Some(docnav_protocol::Options::from_iter([(
            "max_heading_level".to_owned(),
            2.into(),
        )])),
    })
    .expect("outline request");

    assert_eq!(request.protocol_version, PROTOCOL_VERSION);
    assert_eq!(request.operation, Operation::Outline);
    assert_eq!(request.document.path, "docs/guide.md");
    match request.arguments {
        OperationArguments::Outline(arguments) => {
            assert_eq!(arguments.page.get(), 1);
            assert_eq!(arguments.limit.get(), 80);
            assert_eq!(
                arguments
                    .options
                    .as_ref()
                    .and_then(|options| options.get("max_heading_level"))
                    .and_then(Value::as_i64),
                Some(2)
            );
        }
        arguments => panic!("expected outline arguments, got {arguments:?}"),
    }
}

#[test]
fn protocol_request_rejects_missing_read_ref() {
    let error = protocol_request(OperationInput {
        operation: Operation::Read,
        document_path: "docs/guide.md".to_owned(),
        ref_id: None,
        query: None,
        page: Some(positive_result(1).unwrap()),
        limit: Some(positive_result(80).unwrap()),
        options: None,
    })
    .expect_err("read ref required");

    assert_eq!(error.field(), "ref");
    assert_eq!(error.reason(), "read requires ref");
}

#[test]
fn response_validation_failure_carries_result_validation_layer() {
    let response = ProtocolResponse::Success(SuccessResponse {
        protocol_version: PROTOCOL_VERSION.to_owned(),
        request_id: "req-invalid-result".to_owned(),
        operation: Operation::Read,
        ok: true,
        result: OperationResult::Outline(OutlineResult::structured(Vec::new(), None)),
    });
    let mut trace = NavigationInvocationTrace {
        operation: Operation::Read,
        selected_adapter_id: Some("docnav-test".to_owned()),
        request_id: Some("req-invalid-result".to_owned()),
        failure_layer: None,
    };

    let error = validate_navigation_response(response, &mut trace).expect_err("invalid response");

    assert_eq!(
        error.failure_layer(),
        Some(NavigationFailureLayer::ResultValidation)
    );
    assert_eq!(error.selected_adapter_id(), Some("docnav-test"));
    assert_eq!(error.request_id(), Some("req-invalid-result"));
}
