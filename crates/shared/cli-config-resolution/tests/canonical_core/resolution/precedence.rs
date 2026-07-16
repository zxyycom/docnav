use cli_config_resolution::{resolve, TypedValue};
use serde_json::json;

use crate::support::{candidate, identity, mode_field_set, source};

#[test]
fn higher_priority_source_wins() {
    let fields = mode_field_set();
    let low = source("low", 10, [candidate("mode", json!("low"))]);
    let high = source("high", 20, [candidate("mode", json!("high"))]);
    let resolved = resolve(&fields, &[low, high]).expect("valid input");
    assert_eq!(
        resolved.materialize().expect("resolved")[&identity("mode")],
        TypedValue::String("high".to_owned())
    );
}

#[test]
fn later_source_wins_at_equal_priority() {
    let fields = mode_field_set();
    let earlier = source("earlier", 30, [candidate("mode", json!("earlier"))]);
    let later = source("later", 30, [candidate("mode", json!("later"))]);
    let resolved = resolve(&fields, &[earlier, later]).expect("valid input");
    assert_eq!(
        resolved.materialize().expect("resolved")[&identity("mode")],
        TypedValue::String("later".to_owned())
    );
    assert_eq!(
        resolved
            .trace(&identity("mode"))
            .expect("trace")
            .selected
            .as_ref()
            .expect("selected")
            .source_id
            .as_str(),
        "later"
    );
}
