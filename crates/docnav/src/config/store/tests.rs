use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

// @case WB-CORE-CONFIG-SOURCE-001
use super::{read_selected_config, write_config, ConfigFileSource};
use crate::project_context::{
    ConfigPathOrigin, ProjectContext, SelectedConfigPath, SelectedConfigPaths,
};
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

    let error = read_selected_config(
        &SelectedConfigPath::default(path),
        &registry,
        ConfigFileSource::Project,
    )
    .unwrap_err();
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
fn adapter_id_native_option_config_key_is_typed_validated() {
    let root = temp_root("adapter-id-option");
    let path = write_project_config(
        &root,
        json!({
            "options": {
                "docnav-markdown": {
                    "max_heading_level": 2
                }
            }
        }),
    );
    let registry = registry_for_root(&root);

    let config = read_selected_config(
        &SelectedConfigPath::default(path),
        &registry,
        ConfigFileSource::Project,
    )
    .unwrap();

    assert_eq!(
        config.options.value_for_key("docnav-markdown"),
        Some(&json!({"max_heading_level": 2}))
    );
}

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
    let registry = registry_for_root(&root);

    let error = read_selected_config(
        &SelectedConfigPath::default(path),
        &registry,
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
    let registry = registry_for_root(&root);

    let error = read_selected_config(
        &SelectedConfigPath::default(path),
        &registry,
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
fn navigation_owned_outline_config_is_accepted_and_preserved() {
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
                "output": "readable-json"
            },
            "outline": outline.clone()
        }),
    );
    let registry = registry_for_root(&root);

    let config = read_selected_config(
        &SelectedConfigPath::default(path),
        &registry,
        ConfigFileSource::Project,
    )
    .unwrap();
    assert_eq!(config.outline.as_ref(), Some(&outline));

    let rewritten = root.join(".docnav").join("rewritten.json");
    write_config(&rewritten, &config).unwrap();
    let value: Value = serde_json::from_str(&fs::read_to_string(&rewritten).unwrap()).unwrap();
    assert_eq!(value["outline"], outline);
    assert_eq!(value["defaults"]["output"], "readable-json");
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
    let registry = registry_for_root(&root);

    let error = read_selected_config(
        &SelectedConfigPath::default(path),
        &registry,
        ConfigFileSource::Project,
    )
    .unwrap_err();
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["field"], "invocation_log.path");
    assert_eq!(details["reason"], "length_invalid");
}

#[test]
fn direct_config_file_rejects_empty_invocation_log_content_capture_root() {
    let root = temp_root("empty-invocation-log-content-root");
    let path = write_project_config(
        &root,
        json!({
            "invocation_log": {
                "content_capture": {
                    "root": ""
                }
            }
        }),
    );
    let registry = registry_for_root(&root);

    let error = read_selected_config(
        &SelectedConfigPath::default(path),
        &registry,
        ConfigFileSource::Project,
    )
    .unwrap_err();
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["field"], "invocation_log.content_capture.root");
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
    let registry = registry_for_root(&root);

    let error = read_selected_config(
        &SelectedConfigPath::default(path),
        &registry,
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
    let registry = registry_for_root(&root);
    let path = root.join(".docnav").join("missing.json");

    let config = read_selected_config(
        &SelectedConfigPath::default(path),
        &registry,
        ConfigFileSource::Project,
    )
    .unwrap();

    assert_eq!(config, super::CoreConfig::default());
}

#[test]
fn explicit_missing_config_path_reports_blocking_issue() {
    let root = temp_root("explicit-missing");
    let registry = registry_for_root(&root);
    let path = root.join("selected-project.json");

    let error = read_selected_config(
        &SelectedConfigPath {
            path,
            origin: ConfigPathOrigin::ExplicitCli,
        },
        &registry,
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

fn registry_for_root(root: &Path) -> AdapterRegistry {
    AdapterRegistry::load(&ProjectContext {
        cwd: root.to_owned(),
        project_root: root.to_owned(),
        config_paths: SelectedConfigPaths {
            project: SelectedConfigPath::default(root.join(".docnav").join("docnav.json")),
            user: SelectedConfigPath::default(root.join(".user-config").join("docnav.json")),
        },
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
