use super::*;
use serde_json::json;

#[test]
fn options_preserve_the_plain_json_object_wire_shape() {
    let options = Options::from_iter([
        ("max_heading_level".to_owned(), json!(4)),
        ("mode".to_owned(), json!("structured")),
    ]);

    assert_eq!(
        serde_json::to_value(&options).expect("serialize protocol options"),
        json!({
            "max_heading_level": 4,
            "mode": "structured"
        })
    );
    let decoded: Options = serde_json::from_value(json!({
        "max_heading_level": 4,
        "mode": "structured"
    }))
    .expect("deserialize protocol options");
    assert_eq!(decoded, options);
    assert_eq!(decoded.len(), 2);
    assert_eq!(options.get("mode"), Some(&json!("structured")));
}
