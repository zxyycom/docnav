use serde_json::Value;

pub(super) fn minimal_manifest() -> Value {
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
                "filenames": [],
                "content_types": ["text/stub"]
            }
        ]
    })
}

pub(super) fn manifest_with(update: impl FnOnce(&mut Value)) -> Value {
    let mut manifest = minimal_manifest();
    update(&mut manifest);
    manifest
}

pub(super) fn protocol_outline_response_with(update: impl FnOnce(&mut Value)) -> Value {
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

pub(super) fn protocol_outline_auto_read_response() -> Value {
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

pub(super) fn protocol_outline_auto_read_response_with(update: impl FnOnce(&mut Value)) -> Value {
    let mut response = protocol_outline_auto_read_response();
    update(&mut response);
    response
}

pub(super) fn protocol_find_auto_read_response() -> Value {
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

pub(super) fn auto_read_value() -> Value {
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

pub(super) fn protocol_format_unknown_error_with(update: impl FnOnce(&mut Value)) -> Value {
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
                "reason": "FORMAT_NOT_RECOGNIZED",
                "candidates": []
            }
        }
    });
    update(&mut response);
    response
}

pub(super) fn protocol_document_content_invalid_error_with(
    update: impl FnOnce(&mut Value),
) -> Value {
    let mut response = serde_json::json!({
        "protocol_version": "0.1",
        "request_id": "req-1",
        "operation": "outline",
        "ok": false,
        "error": {
            "code": "DOCUMENT_CONTENT_INVALID",
            "message": "Document content is invalid.",
            "owner": "adapter",
            "details": {
                "path": "docs/file.json",
                "reason": "JSON_SYNTAX_INVALID"
            }
        }
    });
    update(&mut response);
    response
}

pub(super) fn protocol_adapter_unavailable_error_with(update: impl FnOnce(&mut Value)) -> Value {
    let mut response = serde_json::json!({
        "protocol_version": "0.1",
        "request_id": "req-1",
        "operation": "outline",
        "ok": false,
        "error": {
            "code": "ADAPTER_UNAVAILABLE",
            "message": "Adapter is unavailable.",
            "owner": "docnav_navigation_routing",
            "details": {
                "adapter_id": "missing-adapter",
                "reason": "ADAPTER_NOT_FOUND",
                "selection_source": "explicit",
                "stage": "resolve"
            }
        }
    });
    update(&mut response);
    response
}

pub(super) fn protocol_invalid_request_option_issue_with(update: impl FnOnce(&mut Value)) -> Value {
    let mut response = serde_json::json!({
        "protocol_version": "0.1",
        "request_id": "req-1",
        "operation": "outline",
        "ok": false,
        "error": {
            "code": "INVALID_REQUEST",
            "message": "Native option value is invalid.",
            "owner": "adapter_options",
            "details": {
                "field": "arguments.options.max_heading_level",
                "reason": "range_invalid",
                "option_issues": [{
                    "owner": "docnav-markdown",
                    "namespace": "options",
                    "key": "max_heading_level",
                    "source": "standard_input",
                    "type_variant": "integer",
                    "reason_code": "range_invalid",
                    "received": "7",
                    "expected": "integer in range 1..6",
                    "location": {
                        "field": "arguments.options.max_heading_level"
                    }
                }]
            }
        }
    });
    update(&mut response["error"]["details"]["option_issues"][0]);
    response
}

pub(super) fn assert_public_schema_valid(schema_source: &str, value: &Value) {
    let errors = public_schema_errors(schema_source, value);
    assert!(
        errors.is_empty(),
        "public JSON Schema should accept value, got {errors:?}"
    );
}

pub(super) fn assert_public_schema_invalid(schema_source: &str, value: &Value) {
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
