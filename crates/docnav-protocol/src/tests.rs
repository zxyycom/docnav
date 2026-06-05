use super::*;
use serde_json::Value;
use std::collections::BTreeMap;
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
fn selects_highest_compatible_protocol_version() {
    let docnav = ProtocolRange::new(ProtocolVersion::new(0, 1), ProtocolVersion::new(0, 2))
        .expect("valid range");
    let adapter = ProtocolRange::v0_1();

    let selected = select_highest_compatible(&docnav, &adapter).expect("compatible");

    assert_eq!(selected, ProtocolVersion::new(0, 1));
}

#[test]
fn incompatible_protocol_range_builds_stable_error() {
    let docnav = ProtocolRange::new(ProtocolVersion::new(1, 0), ProtocolVersion::new(1, 1))
        .expect("valid range");
    let adapter = ProtocolRange::v0_1();

    let error = select_highest_compatible(&docnav, &adapter).expect_err("incompatible");

    assert_eq!(error.code, StableErrorCode::ProtocolIncompatible);
    assert_eq!(error.details["requested"], "1.0..1.1");
    assert_eq!(error.details["supported_min"], "0.1");
    assert_eq!(error.details["supported_max"], "0.1");
    error.validate_required_details().expect("stable details");
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

    let unparsed = FailureResponse::unparsed(
        StableError::invalid_request("request", "not json"),
        &ProtocolRange::v0_1(),
    );
    assert_eq!(unparsed.operation, None);
    unparsed.validate().expect("unparsed failure validates");
}

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
    let manifest: Manifest =
        serde_json::from_value(manifest_value).expect("manifest fixture parses");
    assert_eq!(manifest.protocol, ProtocolRange::v0_1());
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
fn manifest_schema_rejects_info_recommended_parameters() {
    let value = serde_json::json!({
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
        "capabilities": ["outline", "read", "find", "info"],
        "recommended_parameters": {
            "info": {
                "limit_chars": 80
            }
        }
    });

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

#[test]
fn manifest_semantics_reject_invalid_protocol_range() {
    let manifest = Manifest {
        manifest_version: MANIFEST_VERSION.to_owned(),
        adapter: AdapterIdentity {
            id: "stub".to_owned(),
            name: "Stub".to_owned(),
            version: "0.1.0".to_owned(),
        },
        protocol: ProtocolRange {
            min: ProtocolVersion::new(0, 2),
            max: ProtocolVersion::new(0, 1),
        },
        formats: vec![FormatDescriptor {
            id: "stub".to_owned(),
            extensions: vec![".stub".to_owned()],
            content_types: vec!["text/stub".to_owned()],
        }],
        capabilities: vec![Operation::Outline],
        recommended_parameters: BTreeMap::new(),
    };

    assert_eq!(
        manifest.validate_semantics(),
        Err(ManifestValidationError::InvalidProtocolRange {
            min: ProtocolVersion::new(0, 2),
            max: ProtocolVersion::new(0, 1),
        })
    );
}
