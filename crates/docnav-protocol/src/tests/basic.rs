use super::*;
use docnav_diagnostics::{DiagnosticSource, DiagnosticStack, ProtocolDiagnosticCode};
use serde_json::Value;
use std::collections::BTreeMap;

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
        assert_eq!(
            code.diagnostic().projection_rule().protocol_code,
            Some(code.diagnostic_code().protocol_code()),
            "{code:?}"
        );
    }
}

#[test]
fn stable_error_required_details_come_from_diagnostic_rules() {
    assert_eq!(
        StableErrorCode::InvalidRequest.required_details(),
        docnav_diagnostics::ProtocolDiagnosticCode::InvalidRequest.required_detail_names()
    );
    assert_eq!(
        StableErrorCode::AdapterInvokeFailed.required_details(),
        docnav_diagnostics::ProtocolDiagnosticCode::AdapterInvokeFailed.required_detail_names()
    );
}

#[test]
fn stable_error_roundtrips_through_diagnostic_record_projection() {
    let error = StableError::ref_not_found("R99").with_guidance(["Run outline first."]);
    let mut diagnostics = DiagnosticStack::new();
    let id = diagnostics
        .push(error.to_record_draft(DiagnosticSource::with_stage("docnav-protocol", "test")))
        .unwrap();
    let record = diagnostics.get(id).unwrap();

    assert_eq!(
        record.guidance.as_deref(),
        Some(&["Run outline first.".to_owned()][..])
    );
    assert_eq!(StableError::from_diagnostic_record(record), Some(error));
}

#[test]
fn protocol_response_schema_error_projection_matches_diagnostic_rules() {
    let schema: Value = serde_json::from_str(include_str!(
        "../../../../docs/schemas/protocol-response.schema.json"
    ))
    .expect("protocol response schema parses");
    let mut schema_rules = protocol_schema_error_details(&schema);
    let expected = [
        ProtocolDiagnosticCode::InvalidRequest,
        ProtocolDiagnosticCode::DocumentNotFound,
        ProtocolDiagnosticCode::DocumentPathInvalid,
        ProtocolDiagnosticCode::DocumentEncodingUnsupported,
        ProtocolDiagnosticCode::FormatUnknown,
        ProtocolDiagnosticCode::FormatAmbiguous,
        ProtocolDiagnosticCode::CapabilityUnsupported,
        ProtocolDiagnosticCode::RefNotFound,
        ProtocolDiagnosticCode::RefAmbiguous,
        ProtocolDiagnosticCode::RefInvalid,
        ProtocolDiagnosticCode::AdapterUnavailable,
        ProtocolDiagnosticCode::AdapterInvokeFailed,
        ProtocolDiagnosticCode::InternalError,
    ];

    for code in expected {
        let actual = schema_rules
            .remove(code.protocol_code())
            .unwrap_or_else(|| panic!("missing schema projection for {}", code.protocol_code()));
        assert_eq!(actual, code.required_detail_names(), "{code:?}");
    }
    assert!(
        schema_rules.is_empty(),
        "unexpected schema rules: {schema_rules:?}"
    );
}

fn protocol_schema_error_details(schema: &Value) -> BTreeMap<String, Vec<String>> {
    let defs = schema
        .get("$defs")
        .and_then(Value::as_object)
        .expect("schema $defs");
    let error_schema = defs["failure"]["properties"]["error"]
        .as_object()
        .expect("failure error schema");
    let code_enum = error_schema["properties"]["code"]["enum"]
        .as_array()
        .expect("error code enum")
        .iter()
        .map(|value| value.as_str().expect("error code string").to_owned())
        .collect::<Vec<_>>();
    let branches = error_schema["allOf"].as_array().expect("error allOf");
    let mut rules = BTreeMap::new();

    for branch in branches {
        let code = branch["if"]["properties"]["code"]["const"]
            .as_str()
            .expect("branch code")
            .to_owned();
        assert!(
            code_enum.contains(&code),
            "{code} missing from schema code enum"
        );
        let details_schema = &branch["then"]["properties"]["details"];
        assert!(
            rules
                .insert(code, required_details_from_schema(defs, details_schema))
                .is_none(),
            "duplicate error details branch"
        );
    }

    assert_eq!(rules.len(), code_enum.len());
    rules
}

fn required_details_from_schema(
    defs: &serde_json::Map<String, Value>,
    details_schema: &Value,
) -> Vec<String> {
    if let Some(ref_value) = details_schema.get("$ref") {
        let ref_path = ref_value.as_str().expect("details $ref string");
        let def_name = ref_path
            .strip_prefix("#/$defs/")
            .expect("details $ref targets schema $defs");
        return required_details(defs.get(def_name).expect("referenced details def"));
    }

    required_details(details_schema)
}

fn required_details(details_schema: &Value) -> Vec<String> {
    details_schema["required"]
        .as_array()
        .expect("details required")
        .iter()
        .map(|value| value.as_str().expect("required field string").to_owned())
        .collect()
}
