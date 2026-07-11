use serde_json::{json, Value};

use crate::{
    DetailFieldRule, DetailFieldType, DiagnosticCode, DiagnosticDetailsError,
    DiagnosticDetailsRule, ProtocolDiagnosticCode,
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
        "accepted": ["readable-view", "readable-json", "protocol-json"]
    });

    assert!(rule.validate_value(&valid).is_ok());

    let wrong_accepted = json!({
        "field": "defaults.output",
        "reason": "invalid output mode",
        "accepted": "readable-view"
    });
    assert!(matches!(
        rule.validate_value(&wrong_accepted),
        Err(DiagnosticDetailsError::WrongType { field, expected })
            if field == "accepted" && expected == DetailFieldType::StringArray
    ));
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
