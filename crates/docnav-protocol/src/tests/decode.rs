use super::*;

#[test]
fn decode_protocol_request_runs_contract_before_raw_decode() {
    let schema_invalid = serde_json::json!({
        "protocol_version": "0.1",
        "request_id": "req-1",
        "operation": "outline",
        "document": { "path": "doc.md" },
        "arguments": { "limit": 80, "page": 1 },
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
}

// @case WB-PROTO-DECODE-001
#[test]
fn decode_protocol_request_rejects_unmapped_arguments() {
    let request = serde_json::json!({
        "protocol_version": "0.1",
        "request_id": "req-1",
        "operation": "outline",
        "document": { "path": "doc.md" },
        "arguments": { "future": true }
    });

    let error = decode_protocol_request_value(request)
        .expect_err("unmapped arguments should fail schema first");
    assert_eq!(error.stage(), DecodePipelineStage::Schema);
}

#[test]
fn decode_protocol_request_preserves_defaultable_arguments() {
    let request = serde_json::json!({
        "protocol_version": "0.1",
        "request_id": "req-1",
        "operation": "outline",
        "document": { "path": "doc.md" },
        "arguments": {}
    });

    let decoded = decode_protocol_request_value(request).expect("raw request decodes");
    assert_eq!(decoded.operation, Operation::Outline);
    assert_eq!(decoded.arguments, serde_json::json!({}));
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
fn decode_protocol_response_keeps_operation_result_pairing_semantic() {
    let semantic_invalid = serde_json::json!({
        "protocol_version": "0.1",
        "request_id": "req-1",
        "operation": "read",
        "ok": true,
        "result": {
            "entries": [],
            "page": null
        }
    });

    let error = decode_protocol_response_value(semantic_invalid)
        .expect_err("operation and result mismatch should fail semantics");
    assert_eq!(error.stage(), DecodePipelineStage::Semantic);
    match error {
        DecodePipelineError::Semantic { error, .. } => {
            assert!(matches!(
                error,
                ProtocolValidationError::ResultOperationMismatch {
                    operation: Operation::Read,
                    result_operation: Operation::Outline
                }
            ));
        }
        _ => panic!("expected semantic error"),
    }
}
