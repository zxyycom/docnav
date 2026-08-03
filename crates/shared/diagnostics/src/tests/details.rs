use serde_json::{json, Value};

use crate::{
    AdapterUnavailableDetails, DetailFieldRule, DetailFieldType, DiagnosticCode,
    DiagnosticDetailsError, DiagnosticDetailsRule, DocumentContentInvalidDetails,
    DocumentContentInvalidReason, ProtocolDiagnosticCode,
};

const REPRESENTATIVE_FIELD_TYPES: &[DetailFieldRule] = &[
    DetailFieldRule::required("string", DetailFieldType::String),
    DetailFieldRule::required("string_array", DetailFieldType::StringArray),
    DetailFieldRule::required("object_array", DetailFieldType::ObjectArray),
    DetailFieldRule::required("boolean", DetailFieldType::Boolean),
    DetailFieldRule::required("u32", DetailFieldType::U32),
    DetailFieldRule::required("i32", DetailFieldType::I32),
    DetailFieldRule::required("object", DetailFieldType::Object),
    DetailFieldRule::required("any", DetailFieldType::Any),
];

#[test]
fn detail_rule_validates_each_supported_field_type_once() {
    let rule = DiagnosticDetailsRule::exact(REPRESENTATIVE_FIELD_TYPES);
    let valid = representative_details();
    assert!(rule.validate_value(&valid).is_ok());

    for (field_name, field_type, wrong_value) in [
        ("string", DetailFieldType::String, json!(1)),
        ("string_array", DetailFieldType::StringArray, json!([1])),
        (
            "object_array",
            DetailFieldType::ObjectArray,
            json!(["value"]),
        ),
        ("boolean", DetailFieldType::Boolean, json!("true")),
        ("u32", DetailFieldType::U32, json!(4_294_967_296_u64)),
        ("i32", DetailFieldType::I32, json!(2_147_483_648_i64)),
        ("object", DetailFieldType::Object, json!("object")),
    ] {
        let mut wrong = valid.as_object().expect("details object").clone();
        wrong.insert(field_name.to_owned(), wrong_value);
        assert!(
            matches!(
                rule.validate_value(&Value::Object(wrong)),
                Err(DiagnosticDetailsError::WrongType { field, expected })
                    if field == field_name && expected == field_type
            ),
            "{field_name}"
        );
    }
}

#[test]
fn detail_rule_rejects_one_missing_and_extra_field() {
    let rule = DiagnosticDetailsRule::exact(REPRESENTATIVE_FIELD_TYPES);
    let valid = representative_details();

    let mut missing = valid.as_object().expect("details object").clone();
    missing.remove("string");
    assert!(matches!(
        rule.validate_value(&Value::Object(missing)),
        Err(DiagnosticDetailsError::MissingField { field }) if field == "string"
    ));

    let mut extra = valid.as_object().expect("details object").clone();
    extra.insert("extra".to_owned(), json!(true));
    assert!(matches!(
        rule.validate_value(&Value::Object(extra)),
        Err(DiagnosticDetailsError::ExtraField { field }) if field == "extra"
    ));
}

#[test]
fn invalid_request_details_accept_known_optional_context_fields() {
    let rule = DiagnosticCode::from(ProtocolDiagnosticCode::InvalidRequest).details_rule();
    let valid = json!({
        "field": "defaults.output",
        "reason": "invalid output mode",
        "path": ".docnav/docnav.json",
        "received": "text",
        "accepted": ["compact", "expanded", "detailed"]
    });

    assert!(rule.validate_value(&valid).is_ok());

    let wrong_accepted = json!({
        "field": "defaults.output",
        "reason": "invalid output mode",
        "accepted": "compact"
    });
    assert!(matches!(
        rule.validate_value(&wrong_accepted),
        Err(DiagnosticDetailsError::WrongType { field, expected })
            if field == "accepted" && expected == DetailFieldType::StringArray
    ));
}

#[test]
fn document_content_invalid_details_require_exact_path_and_reason() {
    let code = ProtocolDiagnosticCode::from_protocol_code("DOCUMENT_CONTENT_INVALID")
        .expect("selected document content failures need a stable protocol code");
    let rule = DiagnosticCode::from(code).details_rule();
    let valid = json!({
        "path": "/workspace/project/document.json",
        "reason": "JSON_SYNTAX_INVALID"
    });

    assert!(rule.validate_value(&valid).is_ok());

    for (reason, expected) in [
        (
            DocumentContentInvalidReason::JsonSyntaxInvalid,
            "JSON_SYNTAX_INVALID",
        ),
        (
            DocumentContentInvalidReason::JsonTrailingInput,
            "JSON_TRAILING_INPUT",
        ),
        (
            DocumentContentInvalidReason::JsonDuplicateMember,
            "JSON_DUPLICATE_MEMBER",
        ),
        (
            DocumentContentInvalidReason::JsonMaximumDepthExceeded,
            "JSON_MAXIMUM_DEPTH_EXCEEDED",
        ),
    ] {
        let details =
            DocumentContentInvalidDetails::new("/workspace/project/document.json", reason);
        assert_eq!(serde_json::to_value(details).unwrap()["reason"], expected);
    }
    assert!(
        serde_json::from_value::<DocumentContentInvalidDetails>(json!({
            "path": "/workspace/project/document.json",
            "reason": "PARSER_INTERNAL"
        }))
        .is_err()
    );

    let missing_reason = json!({ "path": "/workspace/project/document.json" });
    assert!(matches!(
        rule.validate_value(&missing_reason),
        Err(DiagnosticDetailsError::MissingField { field }) if field == "reason"
    ));

    let with_parser_detail = json!({
        "path": "/workspace/project/document.json",
        "reason": "JSON_SYNTAX_INVALID",
        "parser_message": "unstable parser detail"
    });
    assert!(matches!(
        rule.validate_value(&with_parser_detail),
        Err(DiagnosticDetailsError::ExtraField { field }) if field == "parser_message"
    ));
}

#[test]
fn adapter_unavailable_details_require_exact_lookup_facts() {
    let details = AdapterUnavailableDetails::new("missing-adapter", "explicit");
    assert_eq!(
        serde_json::to_value(details).unwrap(),
        json!({
            "adapter_id": "missing-adapter",
            "reason": "ADAPTER_NOT_FOUND",
            "selection_source": "explicit",
            "stage": "resolve"
        })
    );

    for invalid in [
        json!({
            "adapter_id": "missing-adapter",
            "reason": "ADAPTER_UNAVAILABLE",
            "selection_source": "explicit",
            "stage": "resolve"
        }),
        json!({
            "adapter_id": "missing-adapter",
            "reason": "ADAPTER_NOT_FOUND",
            "selection_source": "explicit",
            "stage": "dispatch"
        }),
        json!({
            "adapter_id": "missing-adapter",
            "reason": "ADAPTER_NOT_FOUND",
            "stage": "resolve"
        }),
    ] {
        assert!(serde_json::from_value::<AdapterUnavailableDetails>(invalid).is_err());
    }
}

fn representative_details() -> Value {
    json!({
        "string": "value",
        "string_array": ["value"],
        "object_array": [{}],
        "boolean": true,
        "u32": 1,
        "i32": -1,
        "object": {},
        "any": null
    })
}
