use super::*;

mod response_contract;
mod support;

use support::*;

const PROTOCOL_REQUEST_SCHEMA: &str =
    include_str!("../../../../../docs/schemas/protocol-request.schema.json");
const PROTOCOL_RESPONSE_SCHEMA: &str =
    include_str!("../../../../../docs/schemas/protocol-response.schema.json");
const MANIFEST_SCHEMA: &str = include_str!("../../../../../docs/schemas/manifest.schema.json");

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
        response
            .validate_contract()
            .expect("typed response contract validates");
    }

    let manifest_value = read_json_fixture("manifest.json");
    assert_public_schema_valid(MANIFEST_SCHEMA, &manifest_value);
    validate_manifest_value(&manifest_value).expect("manifest fixture schema");
    let manifest: Manifest =
        serde_json::from_value(manifest_value).expect("manifest fixture parses");
    manifest
        .validate_semantics()
        .expect("manifest fixture semantics");
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
        serde_json::json!({
            "protocol_version": "0.1",
            "request_id": "",
            "operation": "outline",
            "document": { "path": "doc.md" },
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
        manifest_with(|manifest| manifest["formats"][0]["extensions"][0] = serde_json::json!(".")),
        manifest_with(|manifest| {
            manifest["formats"][0]["extensions"][0] = serde_json::json!(".dir/file")
        }),
        manifest_with(|manifest| {
            manifest["formats"][0]["extensions"][0] = serde_json::json!(".dir\\file")
        }),
        manifest_with(|manifest| manifest["formats"][0]["filenames"] = serde_json::json!([""])),
        manifest_with(|manifest| manifest["formats"][0]["filenames"] = serde_json::json!(["."])),
        manifest_with(|manifest| manifest["formats"][0]["filenames"] = serde_json::json!([".."])),
        manifest_with(|manifest| {
            manifest["formats"][0]["filenames"] = serde_json::json!(["dir/file"])
        }),
        manifest_with(|manifest| manifest["formats"][0]["extra"] = serde_json::json!(true)),
        root_extra.clone(),
    ];

    for value in cases {
        assert_public_schema_invalid(MANIFEST_SCHEMA, &value);
        assert!(validate_manifest_value(&value).is_err());
    }

    assert!(serde_json::from_value::<Manifest>(root_extra).is_err());
}

#[test]
fn manifest_routing_hints_decode_and_round_trip_through_public_contract() {
    let value = serde_json::json!({
        "manifest_version": "0.1",
        "adapter": {
            "id": "stub",
            "name": "Stub",
            "version": "0.1.0"
        },
        "formats": [{
            "id": "json",
            "extensions": [".json", ".schema.json", ".配置+V1"],
            "filenames": [".prettierrc", ".watchmanconfig"],
            "content_types": ["application/json"]
        }]
    });

    assert_public_schema_valid(MANIFEST_SCHEMA, &value);
    let manifest = decode_manifest_value(value.clone()).expect("routing manifest decodes");
    assert_eq!(serde_json::to_value(manifest).unwrap(), value);
}

#[test]
fn protocol_response_contract_rejects_schema_backed_field_failures() {
    response_contract::assert_envelope_and_result_constraints();
    response_contract::assert_typed_error_field_constraints();
    response_contract::assert_option_issue_constraints();
    response_contract::assert_exact_error_details();
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
