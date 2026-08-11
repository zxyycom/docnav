use super::support::*;
use super::*;

pub(super) fn assert_envelope_and_result_constraints() {
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

    let typed_invalid =
        serde_json::from_value::<ProtocolResponse>(protocol_outline_response_with(|response| {
            response["result"]["kind"] = serde_json::json!("structured");
            response["result"]["entries"][0]["ref"] = serde_json::json!("")
        }))
        .expect("empty ref remains representable by the wire-shaped Rust type");
    assert!(matches!(
        typed_invalid.validate_contract(),
        Err(ProtocolResponseContractError::Schema(_))
    ));
}

pub(super) fn assert_typed_error_field_constraints() {
    let cases = [
        protocol_format_unknown_error_with(|response| {
            response["error"]["message"] = serde_json::json!("")
        }),
        protocol_format_unknown_error_with(|response| {
            response["error"]["location"] = serde_json::json!({})
        }),
        protocol_format_unknown_error_with(|response| {
            response["error"]["guidance"] = serde_json::json!([])
        }),
        protocol_format_unknown_error_with(|response| {
            response["error"]["guidance"] = serde_json::json!([""])
        }),
    ];
    for value in cases {
        assert_public_schema_invalid(PROTOCOL_RESPONSE_SCHEMA, &value);
        let response = serde_json::from_value::<ProtocolResponse>(value)
            .expect("schema-backed error constraints remain Rust-representable");
        assert!(matches!(
            response.validate_contract(),
            Err(ProtocolResponseContractError::Schema(_))
        ));
    }
}

pub(super) fn assert_option_issue_constraints() {
    assert_complete_option_issue_accepted();
    assert_invalid_option_issues_rejected();
}

fn assert_complete_option_issue_accepted() {
    let value = protocol_invalid_request_option_issue_with(|_| {});
    assert_public_schema_valid(PROTOCOL_RESPONSE_SCHEMA, &value);
    validate_protocol_response_value(&value)
        .expect("runtime contract accepts a complete option issue");
    serde_json::from_value::<ProtocolResponse>(value)
        .expect("complete option issue remains Rust-representable")
        .validate_contract()
        .expect("typed contract accepts a complete option issue");
}

fn assert_invalid_option_issues_rejected() {
    let mut non_array_option_issues = protocol_invalid_request_option_issue_with(|_| {});
    non_array_option_issues["error"]["details"]["option_issues"] = serde_json::json!({});
    let cases = [
        non_array_option_issues,
        protocol_invalid_request_option_issue_with(|issue| {
            issue.as_object_mut().unwrap().remove("owner");
        }),
        protocol_invalid_request_option_issue_with(|issue| {
            issue["namespace"] = serde_json::json!("");
        }),
        protocol_invalid_request_option_issue_with(|issue| {
            issue["undocumented"] = serde_json::json!(true);
        }),
        protocol_invalid_request_option_issue_with(|issue| {
            issue["location"] = serde_json::json!({});
        }),
    ];
    for value in cases {
        assert_public_schema_invalid(PROTOCOL_RESPONSE_SCHEMA, &value);
        assert!(validate_protocol_response_value(&value).is_err());
        let response = serde_json::from_value::<ProtocolResponse>(value)
            .expect("invalid option issue remains Rust-representable");
        assert!(matches!(
            response.validate_contract(),
            Err(ProtocolResponseContractError::Schema(_))
        ));
    }
}

pub(super) fn assert_exact_error_details() {
    let cases = [
        protocol_format_unknown_error_with(|response| {
            response["error"]["details"]["reason"] = serde_json::json!("UNEXPECTED_REASON")
        }),
        protocol_format_unknown_error_with(|response| {
            response["error"]["details"]["candidates"] = serde_json::json!([{}])
        }),
        protocol_format_unknown_error_with(|response| {
            response["error"]["details"]["evidence"] = serde_json::json!([])
        }),
        protocol_document_content_invalid_error_with(|response| {
            response["error"]["details"]["reason"] = serde_json::json!("PARSER_INTERNAL")
        }),
        protocol_document_content_invalid_error_with(|response| {
            response["error"]["details"]["parser_message"] = serde_json::json!("unstable")
        }),
        protocol_adapter_unavailable_error_with(|response| {
            response["error"]["details"]["reason"] = serde_json::json!("ADAPTER_UNAVAILABLE")
        }),
        protocol_adapter_unavailable_error_with(|response| {
            response["error"]["details"]["stage"] = serde_json::json!("dispatch")
        }),
        protocol_adapter_unavailable_error_with(|response| {
            response["error"]["details"]
                .as_object_mut()
                .unwrap()
                .remove("selection_source");
        }),
    ];

    for value in cases {
        assert_public_schema_invalid(PROTOCOL_RESPONSE_SCHEMA, &value);
        assert!(decode_protocol_response_value(value).is_err());
    }
}
