use std::env;
use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectContext {
    pub cwd: PathBuf,
    pub project_root: PathBuf,
    pub project_config_path: PathBuf,
    pub user_config_path: PathBuf,
}

impl ProjectContext {
    pub fn discover() -> AppResult<Self> {
        let cwd = env::current_dir()
            .map_err(|error| AppError::invalid_request("cwd", error.to_string()))?;
        let project_root = find_project_root(&cwd);
        let project_config_path = project_root.join(".docnav").join("docnav.json");
        let user_config_path = user_config_path()?;
        Ok(Self {
            cwd,
            project_root,
            project_config_path,
            user_config_path,
        })
    }
}

pub fn find_project_root(cwd: &Path) -> PathBuf {
    for ancestor in cwd.ancestors() {
        if ancestor.join(".docnav").is_dir() {
            return ancestor.to_path_buf();
        }
    }
    cwd.to_path_buf()
}

fn user_config_path() -> AppResult<PathBuf> {
    if let Some(config_dir) = env::var_os("DOCNAV_CONFIG_DIR") {
        return Ok(PathBuf::from(config_dir).join("docnav.json"));
    }
    if let Some(appdata) = env::var_os("APPDATA") {
        return Ok(PathBuf::from(appdata).join("docnav").join("docnav.json"));
    }
    if let Some(config_home) = env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(config_home)
            .join("docnav")
            .join("docnav.json"));
    }
    if let Some(home) = env::var_os("HOME") {
        return Ok(PathBuf::from(home)
            .join(".config")
            .join("docnav")
            .join("docnav.json"));
    }
    if let Some(userprofile) = env::var_os("USERPROFILE") {
        return Ok(PathBuf::from(userprofile)
            .join("AppData")
            .join("Roaming")
            .join("docnav")
            .join("docnav.json"));
    }
    Err(AppError::invalid_request(
        "user_config",
        "could not determine user config directory",
    ))
}
