use serde_json::{json, Map, Value};

use crate::{
    DetailFieldType, DiagnosticCode, DiagnosticDetailsError, DiagnosticDetailsRule,
    ProtocolDiagnosticCode,
};

#[test]
fn detail_rules_reject_missing_wrong_and_extra_fields_for_each_code() {
    for code in DiagnosticCode::all() {
        let rule = code.details_rule();
        let required = rule
            .fields()
            .iter()
            .find(|field| field.required)
            .expect("each diagnostic code has at least one required details field");
        let valid = valid_details_for(rule);
        assert!(
            rule.validate_value(&Value::Object(valid.clone())).is_ok(),
            "{code:?}"
        );

        let mut missing = valid.clone();
        missing.remove(required.name);
        assert!(
            matches!(
                rule.validate_value(&Value::Object(missing)),
                Err(DiagnosticDetailsError::MissingField { field }) if field == required.name
            ),
            "{code:?}"
        );

        if required.kind != DetailFieldType::Any {
            let mut wrong = valid.clone();
            wrong.insert(required.name.to_owned(), wrong_value_for(required.kind));
            assert!(
                matches!(
                    rule.validate_value(&Value::Object(wrong)),
                    Err(DiagnosticDetailsError::WrongType { field, expected })
                        if field == required.name && expected == required.kind
                ),
                "{code:?}"
            );
        }

        let mut extra = valid;
        extra.insert("extra".to_owned(), json!(true));
        assert!(
            matches!(
                rule.validate_value(&Value::Object(extra)),
                Err(DiagnosticDetailsError::ExtraField { field }) if field == "extra"
            ),
            "{code:?}"
        );
    }
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

fn valid_details_for(rule: DiagnosticDetailsRule) -> Map<String, Value> {
    rule.fields()
        .iter()
        .filter(|field| field.required)
        .map(|field| (field.name.to_owned(), value_for(field.kind)))
        .collect()
}

fn value_for(kind: DetailFieldType) -> Value {
    match kind {
        DetailFieldType::String => json!("value"),
        DetailFieldType::StringArray => json!(["value"]),
        DetailFieldType::ObjectArray => json!([{}]),
        DetailFieldType::Boolean => json!(true),
        DetailFieldType::U32 => json!(1),
        DetailFieldType::I32 => json!(-1),
        DetailFieldType::Object => json!({}),
        DetailFieldType::Any => json!({"any": true}),
    }
}

fn wrong_value_for(kind: DetailFieldType) -> Value {
    match kind {
        DetailFieldType::String => json!(1),
        DetailFieldType::StringArray => json!("value"),
        DetailFieldType::ObjectArray => json!("value"),
        DetailFieldType::Boolean => json!("true"),
        DetailFieldType::U32 => json!(-1),
        DetailFieldType::I32 => json!("1"),
        DetailFieldType::Object => json!("object"),
        DetailFieldType::Any => json!(null),
    }
}
