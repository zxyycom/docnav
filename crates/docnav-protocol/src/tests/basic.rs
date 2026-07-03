use super::*;
use docnav_diagnostics::{
    typed_codes, AdapterConfigSourceDetails, DiagnosticCode, DiagnosticRecordDraft,
    DiagnosticSource, DiagnosticStack, FieldReasonDetails, ProtocolDiagnosticCode,
};

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
                label: "Guide".to_owned(),
                kind: None,
                location: None,
                summary: None,
                excerpt: None,
                rank: None,
                cost: None,
                metadata: None,
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
    assert_eq!(value["result"]["entries"][0]["label"], "Guide");
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
        ProtocolResponse::failure_for_request(&request, ProtocolError::ref_not_found("missing"));

    match request_failure {
        ProtocolResponse::Failure(response) => {
            assert_eq!(response.operation, Some(Operation::Read));
            response.validate().expect("failure validates");
        }
        ProtocolResponse::Success(_) => panic!("expected failure"),
    }

    let unparsed = FailureResponse::unparsed(ProtocolError::invalid_request("request", "not json"));
    assert_eq!(unparsed.protocol_version, PROTOCOL_VERSION);
    assert_eq!(unparsed.operation, None);
    unparsed.validate().expect("unparsed failure validates");
}

#[test]
fn protocol_error_codes_use_diagnostic_categories() {
    let cases = [
        (
            ProtocolDiagnosticCode::InvalidRequest,
            ProtocolErrorCategory::Request,
        ),
        (
            ProtocolDiagnosticCode::DocumentNotFound,
            ProtocolErrorCategory::Document,
        ),
        (
            ProtocolDiagnosticCode::DocumentPathInvalid,
            ProtocolErrorCategory::Document,
        ),
        (
            ProtocolDiagnosticCode::DocumentEncodingUnsupported,
            ProtocolErrorCategory::Document,
        ),
        (
            ProtocolDiagnosticCode::FormatUnknown,
            ProtocolErrorCategory::Document,
        ),
        (
            ProtocolDiagnosticCode::FormatAmbiguous,
            ProtocolErrorCategory::Document,
        ),
        (
            ProtocolDiagnosticCode::RefNotFound,
            ProtocolErrorCategory::Document,
        ),
        (
            ProtocolDiagnosticCode::RefAmbiguous,
            ProtocolErrorCategory::Document,
        ),
        (
            ProtocolDiagnosticCode::RefInvalid,
            ProtocolErrorCategory::Document,
        ),
        (
            ProtocolDiagnosticCode::AdapterUnavailable,
            ProtocolErrorCategory::AdapterBoundary,
        ),
        (
            ProtocolDiagnosticCode::InternalError,
            ProtocolErrorCategory::Internal,
        ),
    ];

    for (code, expected) in cases {
        assert_eq!(protocol_error_category(code), expected, "{code:?}");
        assert_eq!(
            DiagnosticCode::from(code).projection_rule().protocol_code,
            Some(code.protocol_code()),
            "{code:?}"
        );
    }
}

#[test]
fn protocol_error_required_details_come_from_diagnostic_rules() {
    assert_eq!(
        ProtocolDiagnosticCode::InvalidRequest
            .required_detail_names()
            .collect::<Vec<_>>(),
        docnav_diagnostics::ProtocolDiagnosticCode::InvalidRequest
            .required_detail_names()
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ProtocolDiagnosticCode::AdapterUnavailable
            .required_detail_names()
            .collect::<Vec<_>>(),
        docnav_diagnostics::ProtocolDiagnosticCode::AdapterUnavailable
            .required_detail_names()
            .collect::<Vec<_>>()
    );
}

#[test]
fn navigation_routing_default_guidance_uses_static_registry_language() {
    for code in [
        ProtocolDiagnosticCode::FormatUnknown,
        ProtocolDiagnosticCode::AdapterUnavailable,
    ] {
        let guidance = protocol_error_default_guidance(code);

        assert!(
            guidance.contains("built-in adapter")
                || guidance.contains("current core release static registry"),
            "{code:?} guidance should mention the built-in/static registry source: {guidance}"
        );
        for removed_term in ["install", "register", "executable", "artifact"] {
            assert!(
                !guidance.contains(removed_term),
                "{code:?} guidance should not mention {removed_term}: {guidance}"
            );
        }
    }
}

#[test]
fn protocol_error_roundtrips_through_diagnostic_record_projection() {
    let error = ProtocolError::ref_not_found("R99")
        .with_owner("docnav_protocol_test")
        .with_guidance(["Run outline first."]);
    let mut diagnostics = DiagnosticStack::new();
    let id = diagnostics
        .push(
            error
                .to_record_draft(DiagnosticSource::with_stage("docnav-protocol", "test"))
                .expect("typed protocol error details convert to diagnostic record"),
        )
        .unwrap();
    let record = diagnostics.get(id).unwrap();

    assert_eq!(
        record.guidance(),
        Some(&["Run outline first.".to_owned()][..])
    );
    assert_eq!(ProtocolError::from_diagnostic_record(record), Some(error));
}

#[test]
fn protocol_error_location_uses_config_issue_path_and_field() {
    let mut details = FieldReasonDetails::new("defaults.limit", "unknown_config_field");
    details.received = Some("defaults.limit".to_owned());
    details.config_issues = Some(vec![AdapterConfigSourceDetails::new(
        "project",
        "default",
        ".docnav/docnav.json",
        "unknown_config_field",
    )
    .with_field("defaults.limit")]);

    let mut diagnostics = DiagnosticStack::new();
    let id = diagnostics
        .push(DiagnosticRecordDraft::new::<
            typed_codes::protocol::InvalidRequest,
        >(
            "Config file contains an unknown field.",
            details,
            DiagnosticSource::with_stage("docnav", "config"),
        ))
        .unwrap();
    let record = diagnostics.get(id).unwrap();
    let error = ProtocolError::from_diagnostic_record(record).unwrap();

    assert_eq!(error.owner(), "docnav_config");
    assert_eq!(
        error.location(),
        Some(&serde_json::json!({
            "config_path": ".docnav/docnav.json",
            "field": "defaults.limit"
        }))
    );
    assert_eq!(error.received(), Some(&serde_json::json!("defaults.limit")));
}
