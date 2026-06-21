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

mod basic;
mod decode;
mod schema;
