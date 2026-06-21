use super::*;

// @case WB-PROTO-DECODE-001
#[test]
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
