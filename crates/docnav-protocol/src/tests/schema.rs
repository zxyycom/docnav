use super::*;

// @case WB-PROTO-SCHEMA-001
#[test]
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
    let current = minimal_manifest();
    validate_manifest_value(&current).expect("current manifest schema");
    serde_json::from_value::<Manifest>(current).expect("current manifest parses");

    for value in [
        manifest_with_removed_protocol(),
        manifest_with_removed_recommended_parameters(),
    ] {
        assert_manifest_rejected(value);
    }
}

fn minimal_manifest() -> Value {
    serde_json::json!({
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
    })
}

fn manifest_with_removed_protocol() -> Value {
    serde_json::json!({
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
    })
}

fn manifest_with_removed_recommended_parameters() -> Value {
    serde_json::json!({
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
    })
}

fn assert_manifest_rejected(value: Value) {
    assert!(validate_manifest_value(&value).is_err());
    assert!(serde_json::from_value::<Manifest>(value).is_err());
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
