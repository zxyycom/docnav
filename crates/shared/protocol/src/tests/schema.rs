use super::*;

const PROTOCOL_REQUEST_SCHEMA: &str =
    include_str!("../../../../../docs/schemas/protocol-request.schema.json");
const PROTOCOL_RESPONSE_SCHEMA: &str =
    include_str!("../../../../../docs/schemas/protocol-response.schema.json");
const MANIFEST_SCHEMA: &str = include_str!("../../../../../docs/schemas/manifest.schema.json");
const PROBE_RESULT_SCHEMA: &str =
    include_str!("../../../../../docs/schemas/probe-result.schema.json");

// @case WB-PROTO-SCHEMA-001
#[test]
fn parses_protocol_fixtures_into_shared_types() {
    for operation in ["outline", "read", "find", "info"] {
        let request_value = read_json_fixture(&format!("protocol-{operation}-request.json"));
        assert_public_schema_valid(PROTOCOL_REQUEST_SCHEMA, &request_value);
        validate_protocol_request_value(&request_value).expect("request fixture schema");
        let request: RequestEnvelope =
            serde_json::from_value(request_value).expect("request fixture parses");
        request
            .operation_arguments()
            .expect("arguments match operation");

        let response_value = read_json_fixture(&format!("protocol-{operation}-response.json"));
        assert_public_schema_valid(PROTOCOL_RESPONSE_SCHEMA, &response_value);
        validate_protocol_response_value(&response_value).expect("response fixture schema");
        let response: ProtocolResponse =
            serde_json::from_value(response_value).expect("response fixture parses");
        response.validate().expect("response validates");
    }

    let manifest_value = read_json_fixture("manifest.json");
    assert_public_schema_valid(MANIFEST_SCHEMA, &manifest_value);
    validate_manifest_value(&manifest_value).expect("manifest fixture schema");
    let manifest: Manifest =
        serde_json::from_value(manifest_value).expect("manifest fixture parses");
    manifest
        .validate_semantics()
        .expect("manifest fixture semantics");

    let probe_value = read_json_fixture("probe-result.json");
    assert_public_schema_valid(PROBE_RESULT_SCHEMA, &probe_value);
    validate_probe_result_value(&probe_value).expect("probe fixture schema");
    let probe: ProbeResult = serde_json::from_value(probe_value).expect("probe fixture parses");
    assert_eq!(probe.probe_version, PROBE_VERSION);
    probe.validate_semantics().expect("probe fixture semantics");
}

#[test]
fn protocol_request_schema_rejects_an_empty_required_string() {
    let value = serde_json::json!({
        "protocol_version": "0.1",
        "request_id": "",
        "operation": "outline",
        "document": { "path": "doc.md" },
        "arguments": { "limit": 80, "page": 1 }
    });

    assert_public_schema_invalid(PROTOCOL_REQUEST_SCHEMA, &value);
    assert!(validate_protocol_request_value(&value).is_err());
}

#[test]
fn protocol_request_contract_rejects_schema_backed_field_failures() {
    let cases = [
        serde_json::json!({
            "protocol_version": "0.2",
            "request_id": "req-1",
            "operation": "outline",
            "document": { "path": "doc.md" },
            "arguments": { "limit": 80, "page": 1 }
        }),
        serde_json::json!({
            "protocol_version": "0.1",
            "request_id": "req-1",
            "operation": "outline",
            "document": { "path": 1 },
            "arguments": { "limit": 80, "page": 1 }
        }),
        serde_json::json!({
            "protocol_version": "0.1",
            "request_id": "req-1",
            "operation": "outline",
            "document": { "path": "doc.md" },
            "arguments": { "limit": 0, "page": 1 }
        }),
        serde_json::json!({
            "protocol_version": "0.1",
            "request_id": "req-1",
            "operation": "read",
            "document": { "path": "doc.md" },
            "arguments": { "limit": 80, "page": 1 }
        }),
        serde_json::json!({
            "protocol_version": "0.1",
            "request_id": "req-1",
            "operation": "outline",
            "document": { "path": "doc.md", "extra": true },
            "arguments": { "limit": 80, "page": 1 }
        }),
    ];

    for value in cases {
        assert_public_schema_invalid(PROTOCOL_REQUEST_SCHEMA, &value);
        assert!(validate_protocol_request_value(&value).is_err());
    }
}

#[test]
fn manifest_contract_rejects_schema_backed_field_failures() {
    let root_extra = manifest_with(|manifest| manifest["extra"] = serde_json::json!(true));
    let cases = [
        manifest_with(|manifest| manifest["manifest_version"] = serde_json::json!("0.2")),
        manifest_with(|manifest| manifest["adapter"]["id"] = serde_json::json!("")),
        manifest_with(|manifest| manifest["formats"][0]["extensions"][0] = serde_json::json!("md")),
        manifest_with(|manifest| manifest["formats"][0]["extra"] = serde_json::json!(true)),
        root_extra.clone(),
    ];

    for value in cases {
        assert_public_schema_invalid(MANIFEST_SCHEMA, &value);
        assert!(validate_manifest_value(&value).is_err());
    }

    assert!(serde_json::from_value::<Manifest>(root_extra).is_err());
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
        ]
    })
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

    assert_public_schema_invalid(PROBE_RESULT_SCHEMA, &missing_reasons);
    assert_public_schema_invalid(PROBE_RESULT_SCHEMA, &bad_confidence);
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

#[test]
fn probe_contract_rejects_schema_backed_field_failures() {
    let cases = [
        probe_with(|probe| probe["probe_version"] = serde_json::json!("0.2")),
        probe_with(|probe| probe["supported"] = serde_json::json!("yes")),
        probe_with(|probe| probe["reasons"][0]["code"] = serde_json::json!("UNKNOWN")),
        probe_with(|probe| probe["reasons"][0]["extra"] = serde_json::json!(true)),
    ];

    for value in cases {
        assert_public_schema_invalid(PROBE_RESULT_SCHEMA, &value);
        assert!(validate_probe_result_value(&value).is_err());
    }
}

#[test]
fn protocol_response_contract_rejects_schema_backed_field_failures() {
    let cases = [
        protocol_outline_response_with(|response| {
            response["protocol_version"] = serde_json::json!("0.2")
        }),
        protocol_outline_response_with(|response| response["request_id"] = serde_json::json!("")),
        protocol_outline_response_with(|response| {
            response["result"]["entries"][0]["ref"] = serde_json::json!("")
        }),
        protocol_outline_response_with(|response| {
            response["result"]["entries"][0]["extra"] = serde_json::json!(true)
        }),
        serde_json::json!({
            "protocol_version": "0.1",
            "request_id": "req-1",
            "operation": "info",
            "ok": true,
            "result": { "undocumented": true }
        }),
    ];

    for value in cases {
        assert_public_schema_invalid(PROTOCOL_RESPONSE_SCHEMA, &value);
        assert!(validate_protocol_response_value(&value).is_err());
    }
}

#[test]
fn protocol_auto_read_contract_accepts_exact_outline_and_find_success_objects() {
    for value in [
        protocol_outline_auto_read_response(),
        protocol_find_auto_read_response(),
    ] {
        assert_public_schema_valid(PROTOCOL_RESPONSE_SCHEMA, &value);
        validate_protocol_response_value(&value).expect("contract validator accepts auto-read");
        decode_protocol_response_value(value).expect("typed auto-read response decodes");
    }
}

#[test]
fn protocol_auto_read_contract_rejects_status_error_and_extra_fields() {
    let cases = [
        protocol_outline_auto_read_response_with(|response| {
            response["result"]["auto_read"]["status"] = serde_json::json!("success")
        }),
        protocol_outline_auto_read_response_with(|response| {
            response["result"]["auto_read"]["error"] =
                serde_json::json!({ "code": "INTERNAL_ERROR" })
        }),
        protocol_outline_auto_read_response_with(|response| {
            response["result"]["auto_read"]["extra"] = serde_json::json!(true)
        }),
    ];

    for value in cases {
        assert_public_schema_invalid(PROTOCOL_RESPONSE_SCHEMA, &value);
        assert!(validate_protocol_response_value(&value).is_err());
        let error = decode_protocol_response_value(value)
            .expect_err("closed auto-read object should fail schema decoding");
        assert_eq!(error.stage(), DecodePipelineStage::Schema);
    }
}

#[test]
fn protocol_auto_read_contract_rejects_unstructured_read_and_info_placement() {
    let cases = [
        serde_json::json!({
            "protocol_version": "0.1",
            "request_id": "req-unstructured-auto-read",
            "operation": "outline",
            "ok": true,
            "result": {
                "kind": "unstructured",
                "reason": "path_rule",
                "content": "whole document",
                "content_type": "text/markdown",
                "cost": { "measurements": [] },
                "auto_read": auto_read_value()
            }
        }),
        serde_json::json!({
            "protocol_version": "0.1",
            "request_id": "req-read-auto-read",
            "operation": "read",
            "ok": true,
            "result": {
                "ref": "H:L1:H1",
                "content": "# Guide",
                "content_type": "text/markdown",
                "cost": {
                    "measurements": [
                        { "unit": "bytes", "value": 7 }
                    ]
                },
                "page": null,
                "auto_read": auto_read_value()
            }
        }),
        serde_json::json!({
            "protocol_version": "0.1",
            "request_id": "req-info-auto-read",
            "operation": "info",
            "ok": true,
            "result": {
                "document": { "content_type": "text/markdown" },
                "auto_read": auto_read_value()
            }
        }),
    ];

    for value in cases {
        assert_public_schema_invalid(PROTOCOL_RESPONSE_SCHEMA, &value);
        assert!(validate_protocol_response_value(&value).is_err());
        let error = decode_protocol_response_value(value)
            .expect_err("auto-read is only valid on structured outline and find");
        assert_eq!(error.stage(), DecodePipelineStage::Schema);
    }
}

#[test]
fn protocol_response_public_schema_rejects_undocumented_format_candidates() {
    let cases = [
        protocol_format_unknown_error_with(|response| {
            response["error"]["details"]["reason"] = serde_json::json!("NO_SUPPORTED_CANDIDATE")
        }),
        protocol_format_unknown_error_with(|response| {
            response["error"]["details"]["candidates"][0]["code"] =
                serde_json::json!("ADAPTER_UNAVAILABLE")
        }),
        protocol_format_unknown_error_with(|response| {
            response["error"]["details"]["candidates"][0]["details"] = serde_json::json!({})
        }),
        protocol_format_unknown_error_with(|response| {
            response["error"]["details"]["candidates"][0]["stage"] = serde_json::json!("invoke")
        }),
    ];

    for value in cases {
        assert_public_schema_invalid(PROTOCOL_RESPONSE_SCHEMA, &value);
    }
}

fn manifest_with(update: impl FnOnce(&mut Value)) -> Value {
    let mut manifest = minimal_manifest();
    update(&mut manifest);
    manifest
}

fn probe_with(update: impl FnOnce(&mut Value)) -> Value {
    let mut probe = serde_json::json!({
        "probe_version": "0.1",
        "adapter_id": "stub",
        "path": "doc.stub",
        "supported": true,
        "format": "stub",
        "confidence": 1.0,
        "reasons": [
            { "code": "EXTENSION_MATCH", "detail": "extension matched" }
        ]
    });
    update(&mut probe);
    probe
}

fn protocol_outline_response_with(update: impl FnOnce(&mut Value)) -> Value {
    let mut response = serde_json::json!({
        "protocol_version": "0.1",
        "request_id": "req-1",
        "operation": "outline",
        "ok": true,
            "result": {
            "entries": [
                { "ref": "H:L1:H1", "label": "Heading" }
            ],
            "page": null
        }
    });
    update(&mut response);
    response
}

fn protocol_outline_auto_read_response() -> Value {
    serde_json::json!({
        "protocol_version": "0.1",
        "request_id": "req-outline-auto-read",
        "operation": "outline",
        "ok": true,
        "result": {
            "kind": "structured",
            "entries": [
                { "ref": "H:L1:H1", "label": "Guide" }
            ],
            "page": 2,
            "auto_read": auto_read_value()
        }
    })
}

fn protocol_outline_auto_read_response_with(update: impl FnOnce(&mut Value)) -> Value {
    let mut response = protocol_outline_auto_read_response();
    update(&mut response);
    response
}

fn protocol_find_auto_read_response() -> Value {
    serde_json::json!({
        "protocol_version": "0.1",
        "request_id": "req-find-auto-read",
        "operation": "find",
        "ok": true,
        "result": {
            "matches": [
                { "ref": "H:L1:H1", "label": "Guide" }
            ],
            "page": null,
            "auto_read": auto_read_value()
        }
    })
}

fn auto_read_value() -> Value {
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
            "page": 3
        }
    })
}

fn protocol_format_unknown_error_with(update: impl FnOnce(&mut Value)) -> Value {
    let mut response = serde_json::json!({
        "protocol_version": "0.1",
        "request_id": "req-1",
        "operation": "outline",
        "ok": false,
        "error": {
            "code": "FORMAT_UNKNOWN",
            "message": "Document format is unknown.",
            "owner": "docnav_navigation_routing",
            "details": {
                "path": "docs/file.data",
                "reason": "NO_SUPPORTED_ADAPTER",
                "candidates": [
                    {
                        "adapter_id": "docnav-markdown",
                        "stage": "probe",
                        "reason": "PROBE_UNSUPPORTED"
                    }
                ]
            }
        }
    });
    update(&mut response);
    response
}

fn assert_public_schema_valid(schema_source: &str, value: &Value) {
    let errors = public_schema_errors(schema_source, value);
    assert!(
        errors.is_empty(),
        "public JSON Schema should accept value, got {errors:?}"
    );
}

fn assert_public_schema_invalid(schema_source: &str, value: &Value) {
    assert!(
        !public_schema_errors(schema_source, value).is_empty(),
        "public JSON Schema should reject value"
    );
}

fn public_schema_errors(schema_source: &str, value: &Value) -> Vec<String> {
    let schema = serde_json::from_str::<Value>(schema_source).expect("schema parses");
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles");
    validator
        .iter_errors(value)
        .map(|error| error.to_string())
        .collect()
}
