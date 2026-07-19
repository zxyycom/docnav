use super::*;
use docnav_diagnostics::{
    typed_codes, AdapterConfigSourceDetails, DiagnosticCode, DiagnosticRecordDraft,
    DiagnosticSource, FieldReasonDetails, ProtocolDiagnosticCode,
};

// @case WB-PROTO-BASIC-001
#[test]
fn positive_integer_constructors_reject_zero() {
    assert_eq!(try_positive(0), None);

    let error = positive_result(0).expect_err("zero is not a positive integer");
    assert_eq!(error.value(), 0);
}

#[test]
fn constructs_outline_success_response() {
    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "req-outline-001",
        OperationResult::Outline(OutlineResult::structured(
            vec![Entry {
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
            Some(positive(2)),
        )),
    );

    let value = serde_json::to_value(response).expect("response serializes");
    assert_eq!(value["protocol_version"], PROTOCOL_VERSION);
    assert_eq!(value["request_id"], "req-outline-001");
    assert_eq!(value["operation"], "outline");
    assert_eq!(value["ok"], true);
    assert_eq!(value["result"]["kind"], "structured");
    assert_eq!(value["result"]["entries"][0]["ref"], "L1:Guide");
    assert_eq!(value["result"]["entries"][0]["label"], "Guide");
    assert_eq!(value["result"]["page"], 2);
    assert!(value["result"].get("markdown_heading_path").is_none());
}

#[test]
fn constructs_unstructured_outline_success_response() {
    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "req-outline-unstructured-001",
        OperationResult::Outline(OutlineResult::unstructured(
            UnstructuredOutlineReason::PathRule,
            "full body",
            "text/plain",
            Cost {
                measurements: Vec::new(),
            },
        )),
    );

    let value = serde_json::to_value(response).expect("response serializes");
    validate_protocol_response_value(&value).expect("unstructured outline validates");
    assert_eq!(value["operation"], "outline");
    assert_eq!(value["ok"], true);
    assert_eq!(value["result"]["kind"], "unstructured");
    assert_eq!(value["result"]["reason"], "path_rule");
    assert_eq!(value["result"]["content"], "full body");
    assert_eq!(value["result"]["content_type"], "text/plain");
    assert_eq!(
        value["result"]["cost"]["measurements"]
            .as_array()
            .expect("measurements array")
            .len(),
        0
    );
    assert!(value["result"].get("entries").is_none());
    assert!(value["result"].get("ref").is_none());
    assert!(value["result"].get("page").is_none());
    assert!(value["result"].get("continuation").is_none());
}

#[test]
fn constructs_outline_auto_read_success_with_base_fields_and_outer_operation() {
    let entry = Entry {
        ref_id: "H:L1:H1".to_owned(),
        label: "Guide".to_owned(),
        kind: Some("heading".to_owned()),
        location: None,
        summary: None,
        excerpt: None,
        rank: None,
        cost: None,
        metadata: None,
    };
    let read = ReadResult {
        ref_id: entry.ref_id.clone(),
        content: "# Guide".to_owned(),
        content_type: "text/markdown".to_owned(),
        cost: Cost {
            measurements: vec![Measurement {
                unit: "bytes".to_owned(),
                value: 7,
                scope: None,
            }],
        },
        page: Some(positive(2)),
    };
    let result = OutlineResult::Structured(StructuredOutlineResult {
        entries: vec![entry],
        page: Some(positive(3)),
        auto_read: Some(AutoReadResult::unique_ref(read.clone())),
    });

    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "req-outline-auto-read-001",
        OperationResult::Outline(result),
    );
    let value = serde_json::to_value(&response).expect("response serializes");

    assert_eq!(value["operation"], "outline");
    assert_eq!(value["result"]["kind"], "structured");
    assert_eq!(value["result"]["entries"][0]["ref"], "H:L1:H1");
    assert_eq!(value["result"]["page"], 3);
    assert_eq!(
        value["result"]["auto_read"],
        serde_json::json!({
            "reason": "unique_ref",
            "read": {
                "ref": "H:L1:H1",
                "content": "# Guide",
                "content_type": "text/markdown",
                "cost": {
                    "measurements": [
                        { "unit": "bytes", "value": 7 }
                    ]
                },
                "page": 2
            }
        })
    );
    assert_eq!(
        decode_protocol_response_value(value).expect("composed outline response decodes"),
        response
    );
}

#[test]
fn constructs_find_auto_read_success_with_base_fields_and_outer_operation() {
    let entry = Entry {
        ref_id: "H:L5:H2".to_owned(),
        label: "Install".to_owned(),
        kind: Some("heading".to_owned()),
        location: None,
        summary: None,
        excerpt: Some("Install Docnav".to_owned()),
        rank: Some(1.0),
        cost: None,
        metadata: None,
    };
    let read = ReadResult {
        ref_id: entry.ref_id.clone(),
        content: "## Install".to_owned(),
        content_type: "text/markdown".to_owned(),
        cost: Cost {
            measurements: vec![Measurement {
                unit: "bytes".to_owned(),
                value: 10,
                scope: None,
            }],
        },
        page: None,
    };
    let result = FindResult {
        matches: vec![entry],
        page: Some(positive(4)),
        auto_read: Some(AutoReadResult::unique_ref(read)),
    };

    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "req-find-auto-read-001",
        OperationResult::Find(result),
    );
    let value = serde_json::to_value(&response).expect("response serializes");

    assert_eq!(value["operation"], "find");
    assert_eq!(value["result"]["matches"][0]["ref"], "H:L5:H2");
    assert_eq!(value["result"]["page"], 4);
    assert_eq!(value["result"]["auto_read"]["reason"], "unique_ref");
    assert_eq!(
        value["result"]["auto_read"]["read"]["content"],
        "## Install"
    );
    assert_eq!(
        decode_protocol_response_value(value).expect("composed find response decodes"),
        response
    );
}

#[test]
fn base_result_constructors_omit_auto_read() {
    let outline = serde_json::to_value(OutlineResult::structured(Vec::new(), None))
        .expect("outline serializes");
    let find = serde_json::to_value(FindResult::new(Vec::new(), None)).expect("find serializes");

    assert!(outline.get("auto_read").is_none());
    assert!(find.get("auto_read").is_none());
}

#[test]
fn generated_request_id_is_non_empty() {
    let request_id = generate_request_id();
    assert!(!request_id.is_empty());
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

// @case WB-PROTO-DIAGNOSTICS-001
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
fn navigation_routing_default_guidance_uses_static_registry_language() {
    let guidance = protocol_error_default_guidance(ProtocolDiagnosticCode::FormatUnknown);

    assert!(
        guidance.contains("built-in adapter")
            || guidance.contains("current core release static registry"),
        "guidance should mention the built-in/static registry source: {guidance}"
    );
}

#[test]
fn protocol_error_roundtrips_through_diagnostic_record_projection() {
    let error = ProtocolError::ref_not_found("R99")
        .with_owner("docnav_protocol_test")
        .with_guidance(["Run outline first."]);
    let record = error
        .to_record_draft(DiagnosticSource::with_stage("docnav-protocol", "test"))
        .expect("typed protocol error details convert to diagnostic record")
        .into_record()
        .unwrap();

    assert_eq!(
        record.guidance(),
        Some(&["Run outline first.".to_owned()][..])
    );
    assert_eq!(ProtocolError::from_diagnostic_record(&record), Some(error));
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

    let record = DiagnosticRecordDraft::new::<typed_codes::protocol::InvalidRequest>(
        "Config file contains an unknown field.",
        details,
        DiagnosticSource::with_stage("docnav", "config"),
    )
    .into_record()
    .unwrap();
    let error = ProtocolError::from_diagnostic_record(&record).unwrap();

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
