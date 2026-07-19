use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::config::{ConfigContext, CoreConfig};
use crate::project_context::{ProjectContext, SelectedConfigPath, SelectedConfigPaths};

use super::TempWorkspace;

pub(in crate::runtime::tests) fn markdown_project(
    name: &str,
    content: &str,
) -> (TempWorkspace, PathBuf) {
    let workspace = temp_workspace(name);
    let project_root = workspace.path().join("project");
    let docs_dir = project_root.join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(docs_dir.join("guide.md"), content).unwrap();
    (workspace, project_root)
}

pub(in crate::runtime::tests) fn write_native_option_config(path: &Path, value: Value) {
    write_config_file(
        path,
        json!({
            "options": {
                "docnav-markdown": {
                    "max_heading_level": value
                }
            }
        }),
    );
}

pub(in crate::runtime::tests) fn write_config_file(path: &Path, value: Value) {
    fs::create_dir_all(path.parent().expect("config parent")).unwrap();
    fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

pub(in crate::runtime::tests) fn default_context(project_root: PathBuf) -> ConfigContext {
    ConfigContext {
        project: project_context(project_root.clone(), project_root),
        project_config: CoreConfig::default(),
        user_config: CoreConfig::default(),
    }
}

pub(in crate::runtime::tests) fn project_context(
    project_root: PathBuf,
    cwd: PathBuf,
) -> ProjectContext {
    ProjectContext {
        cwd,
        config_paths: SelectedConfigPaths {
            project: SelectedConfigPath::default(project_root.join(".docnav").join("docnav.json")),
            user: SelectedConfigPath::default(
                project_root.join(".docnav-user").join("docnav.json"),
            ),
        },
        project_root,
    }
}

pub(in crate::runtime::tests) fn temp_workspace(name: &str) -> TempWorkspace {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir()
        .join("docnav-runtime-tests")
        .join(format!("{name}-{suffix}"));
    fs::create_dir_all(&path).unwrap();
    TempWorkspace { path }
}
