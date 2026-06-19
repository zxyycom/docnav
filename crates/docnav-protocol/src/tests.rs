use super::*;
use serde_json::Value;
use std::path::PathBuf;

fn positive(value: u32) -> PositiveInteger {
    try_positive(value).expect("test positive integer")
}

#[test]
fn positive_integer_constructors_do_not_panic_on_zero() {
    assert_eq!(try_positive(0), None);

    let error = positive_result(0).expect_err("zero is not a positive integer");
    assert_eq!(error.value(), 0);
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("examples")
        .join("json")
        .join(name)
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name)).expect("fixture should be readable")
}

fn read_json_fixture(name: &str) -> Value {
    serde_json::from_str(&read_fixture(name)).expect("fixture is JSON")
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
// @case WB-PROTO-DECODE-001
fn decode_protocol_request_runs_schema_before_deserialize() {
    let schema_invalid = serde_json::json!({
        "protocol_version": "0.1",
        "request_id": "req-1",
        "operation": "outline",
        "document": { "path": "doc.md" },
        "arguments": { "limit_chars": 80, "page": 1 },
        "extra": true
    });

    let error = decode_protocol_request_value(schema_invalid)
        .expect_err("unknown field should fail schema first");
    assert_eq!(error.stage(), DecodePipelineStage::Schema);
    match error {
        DecodePipelineError::Schema(error) => {
            assert_eq!(error.schema, "protocol-request.schema.json");
        }
        _ => panic!("expected schema error"),
    }

    let deserialize_invalid = serde_json::json!({
        "protocol_version": "0.1",
        "request_id": "req-1",
        "operation": "outline",
        "document": { "path": "doc.md" },
        "arguments": { "limit_chars": 4_294_967_296u64, "page": 1 }
    });

    let error = decode_protocol_request_value(deserialize_invalid)
        .expect_err("u64 larger than NonZeroU32 should fail typed decode");
    assert_eq!(error.stage(), DecodePipelineStage::Deserialize);
}

#[test]
fn decode_probe_result_returns_semantic_error_with_typed_value() {
    let semantic_invalid = serde_json::json!({
        "probe_version": "0.1",
        "adapter_id": "stub",
        "path": "doc.stub",
        "supported": true,
        "format": null,
        "confidence": 1.0,
        "reasons": [
            { "code": "EXTENSION_MATCH", "detail": "extension matched" }
        ]
    });

    let error = decode_probe_result_value(semantic_invalid)
        .expect_err("supported probe without format should fail semantics");
    assert_eq!(error.stage(), DecodePipelineStage::Semantic);
    match error {
        DecodePipelineError::Semantic { value, error } => {
            assert!(value.supported);
            assert_eq!(error, ProbeValidationError::SupportedWithoutFormat);
        }
        _ => panic!("expected semantic error"),
    }
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

#[test]
// @case WB-PROTO-SCHEMA-001
fn parses_protocol_fixtures_into_shared_types() {
    for operation in ["outline", "read", "find", "info"] {
        let request_value = read_json_fixture(&format!("protocol-{operation}-request.json"));
        validate_protocol_request_value(&request_value).expect("request fixture schema");
        let request: RequestEnvelope =
            serde_json::from_value(request_value).expect("request fixture parses");
        request
            .operation_arguments()
            .expect("arguments match operation");

        let response_value = read_json_fixture(&format!("protocol-{operation}-response.json"));
        validate_protocol_response_value(&response_value).expect("response fixture schema");
        let response: ProtocolResponse =
            serde_json::from_value(response_value).expect("response fixture parses");
        response.validate().expect("response validates");
    }

    let manifest_value = read_json_fixture("manifest.json");
    validate_manifest_value(&manifest_value).expect("manifest fixture schema");
    assert!(manifest_value.get("protocol").is_none());
    assert!(manifest_value.get("recommended_parameters").is_none());
    let manifest: Manifest =
        serde_json::from_value(manifest_value).expect("manifest fixture parses");
    manifest
        .validate_semantics()
        .expect("manifest fixture semantics");

    let probe_value = read_json_fixture("probe-result.json");
    validate_probe_result_value(&probe_value).expect("probe fixture schema");
    let probe: ProbeResult = serde_json::from_value(probe_value).expect("probe fixture parses");
    assert_eq!(probe.probe_version, PROBE_VERSION);
    probe.validate_semantics().expect("probe fixture semantics");
}

#[test]
fn protocol_request_schema_rejects_empty_required_strings() {
    let cases = [
        serde_json::json!({
            "protocol_version": "0.1",
            "request_id": "",
            "operation": "outline",
            "document": { "path": "doc.md" },
            "arguments": { "limit_chars": 80, "page": 1 }
        }),
        serde_json::json!({
            "protocol_version": "0.1",
            "request_id": "req-1",
            "operation": "outline",
            "document": { "path": "" },
            "arguments": { "limit_chars": 80, "page": 1 }
        }),
        serde_json::json!({
            "protocol_version": "0.1",
            "request_id": "req-1",
            "operation": "read",
            "document": { "path": "doc.md" },
            "arguments": { "ref": "", "limit_chars": 80, "page": 1 }
        }),
        serde_json::json!({
            "protocol_version": "0.1",
            "request_id": "req-1",
            "operation": "find",
            "document": { "path": "doc.md" },
            "arguments": { "query": "", "limit_chars": 80, "page": 1 }
        }),
    ];

    for value in cases {
        assert!(validate_protocol_request_value(&value).is_err());
    }
}

#[test]
fn manifest_schema_rejects_removed_manifest_fields() {
    let current = serde_json::json!({
        "manifest_version": "0.1",
        "adapter": {
            "id": "stub",
            "name": "Stub",
            "version": "0.1.0"
        },
        "formats": [
            {
                "id": "stub",
                "extensions": [".stub"],
                "content_types": ["text/stub"]
            }
        ],
        "capabilities": ["outline", "read", "find", "info"]
    });

    validate_manifest_value(&current).expect("current manifest schema");
    serde_json::from_value::<Manifest>(current).expect("current manifest parses");

    let old_protocol = serde_json::json!({
        "manifest_version": "0.1",
        "adapter": {
            "id": "stub",
            "name": "Stub",
            "version": "0.1.0"
        },
        "protocol": {
            "min": "0.1",
            "max": "0.1"
        },
        "formats": [
            {
                "id": "stub",
                "extensions": [".stub"],
                "content_types": ["text/stub"]
            }
        ],
        "capabilities": ["outline", "read", "find", "info"]
    });

    let old_recommended_parameters = serde_json::json!({
        "manifest_version": "0.1",
        "adapter": {
            "id": "stub",
            "name": "Stub",
            "version": "0.1.0"
        },
        "formats": [
            {
                "id": "stub",
                "extensions": [".stub"],
                "content_types": ["text/stub"]
            }
        ],
        "capabilities": ["outline", "read", "find", "info"],
        "recommended_parameters": {
            "outline": {
                "limit_chars": 80
            }
        }
    });

    for value in [old_protocol, old_recommended_parameters] {
        assert!(validate_manifest_value(&value).is_err());
        assert!(serde_json::from_value::<Manifest>(value).is_err());
    }
}

#[test]
fn probe_schema_rejects_missing_reasons_and_bad_confidence() {
    let missing_reasons = serde_json::json!({
        "probe_version": "0.1",
        "adapter_id": "stub",
        "path": "doc.stub",
        "supported": true,
        "format": "stub",
        "confidence": 1.0,
        "reasons": []
    });
    let bad_confidence = serde_json::json!({
        "probe_version": "0.1",
        "adapter_id": "stub",
        "path": "doc.stub",
        "supported": true,
        "format": "stub",
        "confidence": 1.5,
        "reasons": [
            { "code": "EXTENSION_MATCH", "detail": "extension matched" }
        ]
    });

    assert!(validate_probe_result_value(&missing_reasons).is_err());
    assert!(validate_probe_result_value(&bad_confidence).is_err());

    let probe: ProbeResult = serde_json::from_value(missing_reasons).expect("shape parses");
    assert_eq!(
        probe.validate_semantics(),
        Err(ProbeValidationError::MissingReasons)
    );
    let probe: ProbeResult = serde_json::from_value(bad_confidence).expect("shape parses");
    assert_eq!(
        probe.validate_semantics(),
        Err(ProbeValidationError::ConfidenceOutOfRange(1.5))
    );
}
