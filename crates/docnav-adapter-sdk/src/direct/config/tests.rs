// @case WB-SDK-DIRECT-CONFIG-001
use super::*;
use std::fs;
use std::path::PathBuf;
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

fn temp_dir(name: &str) -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "docnav-adapter-sdk-config-test-{}-{id}-{name}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}
