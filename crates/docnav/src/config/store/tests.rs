use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use super::{read_config, ConfigFileSource};
use crate::project_context::ProjectContext;
use crate::registry::AdapterRegistry;

#[test]
fn unknown_config_field_reports_structured_config_issue() {
    let root = temp_root("unknown-field");
    let path = write_project_config(
        &root,
        json!({
            "defaults": {
                "limit": 12
            }
        }),
    );
    let registry = registry_for_root(&root);

    let error = read_config(&path, &registry, ConfigFileSource::Project).unwrap_err();
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["field"], "defaults.limit");
    assert_eq!(details["reason"], "unknown_config_field");
    assert_eq!(details["received"], "defaults.limit");
    assert_eq!(details["accepted"], json!(["defaults.pagination.limit"]));
    assert_eq!(
        details["config_issues"][0]["source_level"],
        Value::String("project".to_owned())
    );
    assert_eq!(
        details["config_issues"][0]["path_origin"],
        Value::String("default".to_owned())
    );
    assert_eq!(details["config_issues"][0]["field"], "defaults.limit");
    assert_eq!(
        details["config_issues"][0]["reason_code"],
        "unknown_config_field"
    );
}

#[test]
fn registered_native_option_config_key_keeps_raw_value() {
    let root = temp_root("registered-option");
    let path = write_project_config(
        &root,
        json!({
            "options": {
                "max_heading_level": "wide"
            }
        }),
    );
    let registry = registry_for_root(&root);

    let config = read_config(&path, &registry, ConfigFileSource::Project).unwrap();

    assert_eq!(
        config.options.value_for_key("max_heading_level"),
        Some(&json!("wide"))
    );
}

#[test]
fn nested_non_object_config_field_reports_structured_config_issue() {
    let root = temp_root("nested-non-object");
    let path = write_project_config(
        &root,
        json!({
            "defaults": {
                "pagination": false
            }
        }),
    );
    let registry = registry_for_root(&root);

    let error = read_config(&path, &registry, ConfigFileSource::Project).unwrap_err();
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["field"], "defaults.pagination");
    assert_eq!(details["reason"], "invalid_config_object");
    assert_eq!(details["received"], "defaults.pagination");
    assert_eq!(
        details["config_issues"][0]["source_level"],
        Value::String("project".to_owned())
    );
    assert_eq!(details["config_issues"][0]["field"], "defaults.pagination");
    assert_eq!(
        details["config_issues"][0]["reason_code"],
        "invalid_config_object"
    );
}

fn write_project_config(root: &Path, value: Value) -> PathBuf {
    let path = root.join(".docnav").join("docnav.json");
    fs::create_dir_all(path.parent().expect("config parent")).unwrap();
    fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
    path
}

fn registry_for_root(root: &Path) -> AdapterRegistry {
    AdapterRegistry::load(&ProjectContext {
        cwd: root.to_owned(),
        project_root: root.to_owned(),
        project_config_path: root.join(".docnav").join("docnav.json"),
        user_config_path: root.join(".user-config").join("docnav.json"),
    })
    .unwrap()
}

fn temp_root(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("docnav-config-store-{name}-{nonce}"))
}
