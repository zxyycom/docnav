use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use super::{read_selected_config, ConfigFileSource};
use crate::project_context::{ConfigPathOrigin, SelectedConfigPath};

#[test]
fn bare_native_option_config_path_is_unknown() {
    let root = temp_root("bare-option-unknown");
    let path = write_project_config(
        &root,
        json!({
            "options": {
                "max_heading_level": 2
            }
        }),
    );
    let error = read_selected_config(
        &SelectedConfigPath::default(path),
        ConfigFileSource::Project,
    )
    .unwrap_err();
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["field"], "options.max_heading_level");
    assert_eq!(details["reason"], "unknown_config_field");
}

#[test]
fn invalid_adapter_id_native_option_value_is_rejected() {
    let root = temp_root("invalid-adapter-id-option");
    let path = write_project_config(
        &root,
        json!({
            "options": {
                "docnav-markdown": {
                    "max_heading_level": 9
                }
            }
        }),
    );
    let error = read_selected_config(
        &SelectedConfigPath::default(path),
        ConfigFileSource::Project,
    )
    .unwrap_err();
    let details = error.diagnostic().details().to_value();

    assert_eq!(
        details["field"],
        "options.docnav-markdown.max_heading_level"
    );
    assert_eq!(details["reason"], "range_invalid");
    assert_eq!(
        details["config_issues"][0]["field"],
        "options.docnav-markdown.max_heading_level"
    );
}

#[test]
fn navigation_owned_outline_config_is_accepted() {
    let root = temp_root("outline-preserve");
    let outline = json!({
        "mode_rules": [
            {
                "path": "docs/raw\\.md",
                "mode": "unstructured_full"
            }
        ],
        "auto_full_read": {
            "thresholds": [
                {
                    "adapter": "docnav-markdown",
                    "unit": "bytes",
                    "value": 4096
                }
            ]
        }
    });
    let path = write_project_config(
        &root,
        json!({
            "defaults": {
                "output": "readable-view"
            },
            "outline": outline.clone()
        }),
    );
    let config = read_selected_config(
        &SelectedConfigPath::default(path),
        ConfigFileSource::Project,
    )
    .unwrap();
    assert_eq!(config.outline.as_ref(), Some(&outline));
}

#[test]
fn direct_config_file_rejects_empty_invocation_log_path() {
    let root = temp_root("empty-invocation-log-path");
    let path = write_project_config(
        &root,
        json!({
            "invocation_log": {
                "path": ""
            }
        }),
    );
    let error = read_selected_config(
        &SelectedConfigPath::default(path),
        ConfigFileSource::Project,
    )
    .unwrap_err();
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["field"], "invocation_log.path");
    assert_eq!(details["reason"], "length_invalid");
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
    let error = read_selected_config(
        &SelectedConfigPath::default(path),
        ConfigFileSource::Project,
    )
    .unwrap_err();
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

#[test]
fn default_missing_config_path_is_absent() {
    let root = temp_root("default-missing");
    let path = root.join(".docnav").join("missing.json");

    let config = read_selected_config(
        &SelectedConfigPath::default(path),
        ConfigFileSource::Project,
    )
    .unwrap();

    assert_eq!(config, super::CoreConfig::default());
}

#[test]
fn explicit_missing_config_path_reports_blocking_issue() {
    let root = temp_root("explicit-missing");
    let path = root.join("selected-project.json");

    let error = read_selected_config(
        &SelectedConfigPath {
            path,
            origin: ConfigPathOrigin::ExplicitCli,
        },
        ConfigFileSource::Project,
    )
    .unwrap_err();
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["field"], "config");
    assert_eq!(details["reason"], "missing_explicit_cli");
    assert_eq!(
        details["config_issues"][0]["source_level"],
        Value::String("project".to_owned())
    );
    assert_eq!(
        details["config_issues"][0]["path_origin"],
        Value::String("explicit_cli".to_owned())
    );
    assert_eq!(
        details["config_issues"][0]["reason_code"],
        "missing_explicit_cli"
    );
}

fn write_project_config(root: &Path, value: Value) -> PathBuf {
    let path = root.join(".docnav").join("docnav.json");
    fs::create_dir_all(path.parent().expect("config parent")).unwrap();
    fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
    path
}

struct TempRoot {
    path: PathBuf,
}

impl Deref for TempRoot {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn temp_root(name: &str) -> TempRoot {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    TempRoot {
        path: std::env::temp_dir().join(format!("docnav-config-store-{name}-{nonce}")),
    }
}
