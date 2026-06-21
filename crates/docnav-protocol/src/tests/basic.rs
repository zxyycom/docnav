use super::*;

// @case WB-PROTO-BASIC-001
#[test]
fn positive_integer_constructors_do_not_panic_on_zero() {
    assert_eq!(try_positive(0), None);

    let error = positive_result(0).expect_err("zero is not a positive integer");
    assert_eq!(error.value(), 0);
}

#[test]
fn constructs_outline_success_response() {
    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "req-outline-001",
        OperationResult::Outline(OutlineResult {
            entries: vec![Entry {
                ref_id: "L1:Guide".to_owned(),
                display: "9 lines | 0.1 KB".to_owned(),
            }],
            page: Some(positive(2)),
        }),
    );

    let value = serde_json::to_value(response).expect("response serializes");
    assert_eq!(value["protocol_version"], PROTOCOL_VERSION);
    assert_eq!(value["request_id"], "req-outline-001");
    assert_eq!(value["operation"], "outline");
    assert_eq!(value["ok"], true);
    assert_eq!(value["result"]["entries"][0]["ref"], "L1:Guide");
    assert_eq!(value["result"]["page"], 2);
    assert!(value["result"].get("markdown_heading_path").is_none());
}

#[test]
fn generated_request_id_uses_docnav_prefix_and_numeric_suffix() {
    let request_id = generate_request_id();
    let suffix = request_id
        .strip_prefix(GENERATED_REQUEST_ID_PREFIX)
        .expect("generated id prefix");

    assert!(!suffix.is_empty());
    suffix.parse::<u128>().expect("generated suffix is nanos");
}

#[test]
fn failure_response_rules_preserve_or_null_operation() {
    let request: RequestEnvelope =
        serde_json::from_str(&read_fixture("protocol-read-request.json")).expect("request parses");
    let request_failure =
        ProtocolResponse::failure_for_request(&request, StableError::ref_not_found("missing"));

    match request_failure {
        ProtocolResponse::Failure(response) => {
            assert_eq!(response.operation, Some(Operation::Read));
            response.validate().expect("failure validates");
        }
        ProtocolResponse::Success(_) => panic!("expected failure"),
    }

    let unparsed = FailureResponse::unparsed(StableError::invalid_request("request", "not json"));
    assert_eq!(unparsed.protocol_version, PROTOCOL_VERSION);
    assert_eq!(unparsed.operation, None);
    unparsed.validate().expect("unparsed failure validates");
}

#[test]
fn stable_error_codes_have_shared_categories() {
    let cases = [
        (
            StableErrorCode::InvalidRequest,
            StableErrorCategory::Request,
        ),
        (
            StableErrorCode::CapabilityUnsupported,
            StableErrorCategory::Request,
        ),
        (
            StableErrorCode::DocumentNotFound,
            StableErrorCategory::Document,
        ),
        (
            StableErrorCode::DocumentPathInvalid,
            StableErrorCategory::Document,
        ),
        (
            StableErrorCode::DocumentEncodingUnsupported,
            StableErrorCategory::Document,
        ),
        (
            StableErrorCode::FormatUnknown,
            StableErrorCategory::Document,
        ),
        (
            StableErrorCode::FormatAmbiguous,
            StableErrorCategory::Document,
        ),
        (StableErrorCode::RefNotFound, StableErrorCategory::Document),
        (StableErrorCode::RefAmbiguous, StableErrorCategory::Document),
        (StableErrorCode::RefInvalid, StableErrorCategory::Document),
        (
            StableErrorCode::AdapterUnavailable,
            StableErrorCategory::AdapterBoundary,
        ),
        (
            StableErrorCode::AdapterInvokeFailed,
            StableErrorCategory::AdapterBoundary,
        ),
        (
            StableErrorCode::InternalError,
            StableErrorCategory::Internal,
        ),
    ];

    for (code, expected) in cases {
        assert_eq!(code.category(), expected, "{code:?}");
    }
}
