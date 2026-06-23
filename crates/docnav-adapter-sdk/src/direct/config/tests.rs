// @case WB-SDK-DIRECT-CONFIG-001
use super::*;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[test]
fn resolves_default_paths_from_startup_cwd_and_user_config_dir() {
    let root = temp_dir("path-defaults");
    let nested = root.join("docs").join("guide");
    fs::create_dir_all(root.join(".docnav")).unwrap();
    fs::create_dir_all(&nested).unwrap();
    let user_config = root.join(".user-config");

    let (project, user) = resolve_config_paths(
        "docnav-markdown",
        Some(&user_config),
        &nested,
        ConfigPathOverrides::default(),
    );

    assert_eq!(
        project.path,
        root.join(".docnav").join("docnav-markdown.json")
    );
    assert_eq!(project.origin, ConfigPathOrigin::Default);
    assert_eq!(user.path, user_config.join("docnav-markdown.json"));
    assert_eq!(user.origin, ConfigPathOrigin::Default);
}

#[test]
fn falls_back_to_startup_cwd_for_project_root_and_user_config_dir() {
    let cwd = temp_dir("cwd-fallback");

    let (project, user) =
        resolve_config_paths("adapter", None, &cwd, ConfigPathOverrides::default());

    assert_eq!(project.path, cwd.join(".docnav").join("adapter.json"));
    assert_eq!(user.path, cwd.join("adapter.json"));
}

#[test]
fn resolves_relative_overrides_against_startup_cwd() {
    let cwd = temp_dir("relative-overrides");

    let (project, user) = resolve_config_paths(
        "adapter",
        None,
        &cwd,
        ConfigPathOverrides {
            project: Some(PathBuf::from("fixtures/project.json")),
            user: Some(PathBuf::from("fixtures/user.json")),
        },
    );

    assert_eq!(project.path, cwd.join("fixtures/project.json"));
    assert_eq!(project.origin, ConfigPathOrigin::Override);
    assert_eq!(user.path, cwd.join("fixtures/user.json"));
    assert_eq!(user.origin, ConfigPathOrigin::Override);
}

#[test]
fn loads_project_and_user_config_values_with_fixed_projection() {
    let cwd = temp_dir("load-values");
    let project_dir = cwd.join(".docnav");
    let user_dir = cwd.join(".user");
    fs::create_dir_all(&project_dir).unwrap();
    fs::create_dir_all(&user_dir).unwrap();
    write_json(
        &project_dir.join("adapter.json"),
        json!({
            "defaults": {
                "limit_chars": 123,
                "output": "readable-json",
                "ignored": true
            },
            "options": {
                "max_heading_level": 2,
                "future": {"nested": true}
            },
            "unknown": "ignored"
        }),
    );
    write_json(
        &user_dir.join("adapter.json"),
        json!({
            "defaults": {"output": "protocol-json"},
            "options": {"max_heading_level": 5}
        }),
    );

    let loaded = load_adapter_direct_cli_config(
        "adapter",
        Some(&user_dir),
        &cwd,
        ConfigPathOverrides::default(),
    );

    assert!(loaded.warnings.is_empty());
    assert_eq!(loaded.project.limit_chars, Some(json!(123)));
    assert_eq!(loaded.project.output, Some(json!("readable-json")));
    assert_eq!(loaded.project.native_options["max_heading_level"], json!(2));
    assert_eq!(
        loaded.project.native_options["future"],
        json!({"nested": true})
    );
    assert_eq!(loaded.user.output, Some(json!("protocol-json")));
    assert_eq!(loaded.user.native_options["max_heading_level"], json!(5));
}

#[test]
fn default_missing_sources_do_not_warn_but_invalid_sources_warn() {
    let cwd = temp_dir("missing-and-invalid");
    let user_dir = cwd.join(".user");
    fs::create_dir_all(cwd.join(".docnav")).unwrap();
    fs::create_dir_all(&user_dir).unwrap();
    fs::write(cwd.join(".docnav").join("adapter.json"), "{ invalid").unwrap();
    fs::write(user_dir.join("adapter.json"), "[]").unwrap();

    let loaded = load_adapter_direct_cli_config(
        "adapter",
        Some(&user_dir),
        &cwd,
        ConfigPathOverrides::default(),
    );

    assert_eq!(loaded.warnings.len(), 2);
    assert_warning_reason(&loaded.warnings[0], "project", "default", "invalid_json");
    assert_warning_reason(&loaded.warnings[1], "user", "default", "non_object");

    let no_sources = load_adapter_direct_cli_config(
        "missing-adapter",
        Some(&user_dir),
        &cwd,
        ConfigPathOverrides::default(),
    );
    assert!(no_sources.warnings.is_empty());
}

#[test]
fn override_missing_source_warns_and_does_not_load_default() {
    let cwd = temp_dir("override-missing");
    fs::create_dir_all(cwd.join(".docnav")).unwrap();
    write_json(
        &cwd.join(".docnav").join("adapter.json"),
        json!({"defaults": {"limit_chars": 111}}),
    );

    let loaded = load_adapter_direct_cli_config(
        "adapter",
        None,
        &cwd,
        ConfigPathOverrides {
            project: Some(PathBuf::from("missing.json")),
            user: None,
        },
    );

    assert_eq!(loaded.project, AdapterDirectCliConfig::default());
    assert_eq!(loaded.warnings.len(), 1);
    assert_warning_reason(
        &loaded.warnings[0],
        "project",
        "override",
        "missing_override",
    );
}

#[test]
fn directory_config_source_warns_as_not_file() {
    let cwd = temp_dir("not-file");
    let directory = cwd.join("config-dir");
    fs::create_dir_all(&directory).unwrap();

    let loaded = load_adapter_direct_cli_config(
        "adapter",
        None,
        &cwd,
        ConfigPathOverrides {
            project: Some(directory),
            user: None,
        },
    );

    assert_warning_reason(&loaded.warnings[0], "project", "override", "not_file");
}

fn assert_warning_reason(
    warning: &DirectCliWarning,
    source_level: &str,
    path_origin: &str,
    reason_code: &str,
) {
    let value = serde_json::to_value(warning).unwrap();
    assert_eq!(value["id"], "adapter_config_source_skipped");
    assert_eq!(value["effect"], "operation_continued");
    assert_eq!(value["details"]["source_level"], source_level);
    assert_eq!(value["details"]["path_origin"], path_origin);
    assert_eq!(value["details"]["reason_code"], reason_code);
}

fn write_json(path: &Path, value: Value) {
    fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

fn temp_dir(name: &str) -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "docnav-adapter-sdk-config-test-{}-{id}-{name}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}
