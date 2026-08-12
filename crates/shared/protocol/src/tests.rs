use super::*;
use serde_json::Value;
use std::path::PathBuf;

fn positive(value: u32) -> PositiveInteger {
    try_positive(value).expect("test positive integer")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("docs")
        .join("examples")
        .join("json")
        .join(name)
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name)).expect("fixture should be readable")
}

fn read_json_fixture(name: &str) -> Value {
    serde_json::from_str(&read_fixture(name)).expect("fixture is JSON")
}

#[test]
fn cost_unit_has_stable_text_and_serde_mapping() {
    for (unit, value) in [
        (CostUnit::Lines, "lines"),
        (CostUnit::Bytes, "bytes"),
        (CostUnit::Tokens, "tokens"),
    ] {
        assert_eq!(unit.as_str(), value);
        assert_eq!(unit.to_string(), value);
        assert_eq!(value.parse::<CostUnit>(), Ok(unit));
        assert_eq!(
            serde_json::to_string(&unit).unwrap(),
            format!(r#""{value}""#)
        );
        assert_eq!(
            serde_json::from_str::<CostUnit>(&format!(r#""{value}""#)).unwrap(),
            unit
        );
    }

    assert_eq!(
        "characters".parse::<CostUnit>().unwrap_err().to_string(),
        "invalid cost unit: characters"
    );
}

mod basic;
mod decode;
mod options;
mod schema;
