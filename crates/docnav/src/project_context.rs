use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use docnav_navigation::{
    NavigationConfigSourceDescriptor, NavigationConfigSourceDescriptors,
    NavigationConfigSourceOrigin,
};

use crate::error::{AppError, AppResult};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectContext {
    pub cwd: PathBuf,
    pub project_root: PathBuf,
    pub config_paths: SelectedConfigPaths,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectConfigPathTarget {
    pub project_root: PathBuf,
    pub config_path: SelectedConfigPath,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedConfigPaths {
    pub project: SelectedConfigPath,
    pub user: SelectedConfigPath,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedConfigPath {
    pub path: PathBuf,
    pub origin: ConfigPathOrigin,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigPathOrigin {
    Default,
    ExplicitCli,
}

impl ConfigPathOrigin {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::ExplicitCli => "explicit_cli",
        }
    }

    const fn navigation_source_origin(self) -> NavigationConfigSourceOrigin {
        match self {
            Self::Default => NavigationConfigSourceOrigin::Default,
            Self::ExplicitCli => NavigationConfigSourceOrigin::ExplicitCli,
        }
    }
}

impl SelectedConfigPath {
    pub fn default(path: PathBuf) -> Self {
        Self {
            path,
            origin: ConfigPathOrigin::Default,
        }
    }

    fn explicit_cli(cwd: &Path, path: &str) -> Self {
        Self {
            path: resolve_invocation_path(cwd, path),
            origin: ConfigPathOrigin::ExplicitCli,
        }
    }

    pub fn navigation_source_descriptor(&self) -> NavigationConfigSourceDescriptor {
        NavigationConfigSourceDescriptor::new(
            self.origin.navigation_source_origin(),
            self.path.clone(),
        )
    }
}

impl SelectedConfigPaths {
    pub fn navigation_source_descriptors(&self) -> NavigationConfigSourceDescriptors {
        NavigationConfigSourceDescriptors {
            project: self.project.navigation_source_descriptor(),
            user: self.user.navigation_source_descriptor(),
        }
    }
}

impl ProjectContext {
    pub fn discover_project_config_target(
        project_config: Option<&str>,
    ) -> AppResult<ProjectConfigPathTarget> {
        let (cwd, project_root) = discover_roots()?;
        let config_path = selected_project_config_path(&cwd, &project_root, project_config);
        Ok(ProjectConfigPathTarget {
            project_root,
            config_path,
        })
    }

    pub fn discover_with_cli_config_paths(
        project_config: Option<&str>,
        user_config: Option<&str>,
    ) -> AppResult<Self> {
        let (cwd, project_root) = discover_roots()?;
        let config_paths = SelectedConfigPaths {
            project: selected_project_config_path(&cwd, &project_root, project_config),
            user: selected_user_config_path(&cwd, user_config, |key| env::var_os(key))?,
        };
        Ok(Self {
            cwd,
            project_root,
            config_paths,
        })
    }

    #[cfg(test)]
    pub fn project_config_path(&self) -> &Path {
        &self.config_paths.project.path
    }

    #[cfg(test)]
    pub fn user_config_path(&self) -> &Path {
        &self.config_paths.user.path
    }

    pub fn navigation_config_source_descriptors(&self) -> NavigationConfigSourceDescriptors {
        self.config_paths.navigation_source_descriptors()
    }
}

fn discover_roots() -> AppResult<(PathBuf, PathBuf)> {
    let cwd = discover_cwd()?;
    let project_root = find_project_root(&cwd);
    Ok((cwd, project_root))
}

fn discover_cwd() -> AppResult<PathBuf> {
    env::current_dir().map_err(|error| AppError::invalid_request("cwd", error.to_string()))
}

pub fn find_project_root(cwd: &Path) -> PathBuf {
    for ancestor in cwd.ancestors() {
        if ancestor.join(".docnav").is_dir() {
            return ancestor.to_path_buf();
        }
    }
    cwd.to_path_buf()
}

fn selected_project_config_path(
    cwd: &Path,
    project_root: &Path,
    project_config: Option<&str>,
) -> SelectedConfigPath {
    project_config.map_or_else(
        || SelectedConfigPath::default(project_root.join(".docnav").join("docnav.json")),
        |path| SelectedConfigPath::explicit_cli(cwd, path),
    )
}

fn selected_user_config_path(
    cwd: &Path,
    user_config: Option<&str>,
    env_var: impl FnMut(&str) -> Option<OsString>,
) -> AppResult<SelectedConfigPath> {
    if let Some(user_config) = user_config {
        return Ok(SelectedConfigPath::explicit_cli(cwd, user_config));
    }
    user_config_path_from_env(env_var)
        .map(SelectedConfigPath::default)
        .ok_or_else(|| {
            AppError::invalid_request("user_config", "could not determine user config directory")
        })
}

fn user_config_path_from_env(mut env_var: impl FnMut(&str) -> Option<OsString>) -> Option<PathBuf> {
    if let Some(config_dir) = env_var("DOCNAV_CONFIG_DIR") {
        return Some(PathBuf::from(config_dir).join("docnav.json"));
    }
    if let Some(appdata) = env_var("APPDATA") {
        return Some(platform_user_config_path(PathBuf::from(appdata)));
    }
    if let Some(config_home) = env_var("XDG_CONFIG_HOME") {
        return Some(platform_user_config_path(PathBuf::from(config_home)));
    }
    if let Some(home) = env_var("HOME") {
        return Some(platform_user_config_path(PathBuf::from(home)));
    }
    if let Some(userprofile) = env_var("USERPROFILE") {
        return Some(platform_user_config_path(PathBuf::from(userprofile)));
    }
    None
}

fn platform_user_config_path(root: PathBuf) -> PathBuf {
    root.join(".docnav").join("docnav.json")
}

fn resolve_invocation_path(cwd: &Path, path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn user_config_path_prefers_docnav_config_dir_then_platform_default() {
        let mut values = BTreeMap::new();
        values.insert("DOCNAV_CONFIG_DIR", OsString::from("D:/custom-user-config"));
        values.insert("APPDATA", OsString::from("D:/appdata"));

        let path = user_config_path_from_env(|key| values.get(key).cloned()).unwrap();

        assert_eq!(
            path,
            PathBuf::from("D:/custom-user-config").join("docnav.json")
        );
    }

    #[test]
    fn user_config_path_uses_dot_docnav_under_platform_user_root() {
        let mut values = BTreeMap::new();
        values.insert("XDG_CONFIG_HOME", OsString::from("/home/example/.config"));

        let path = user_config_path_from_env(|key| values.get(key).cloned()).unwrap();

        assert_eq!(
            path,
            PathBuf::from("/home/example/.config")
                .join(".docnav")
                .join("docnav.json")
        );
    }

    #[test]
    fn explicit_config_paths_are_resolved_relative_to_invocation_cwd() {
        let cwd = PathBuf::from("D:/workspace/project/subdir");
        let project_root = PathBuf::from("D:/workspace/project");

        let project = selected_project_config_path(&cwd, &project_root, Some("../project.json"));
        let user = selected_user_config_path(&cwd, Some("user.json"), |_key| None).unwrap();

        assert_eq!(
            project.path,
            PathBuf::from("D:/workspace/project/subdir").join("../project.json")
        );
        assert_eq!(
            user.path,
            PathBuf::from("D:/workspace/project/subdir").join("user.json")
        );
        assert_eq!(project.origin, ConfigPathOrigin::ExplicitCli);
        assert_eq!(user.origin, ConfigPathOrigin::ExplicitCli);
    }

    #[test]
    fn explicit_user_config_path_does_not_require_platform_default() {
        let cwd = PathBuf::from("D:/workspace/project");

        let selection = selected_user_config_path(&cwd, Some("user.json"), |_key| None).unwrap();

        assert_eq!(selection.path, cwd.join("user.json"));
        assert_eq!(selection.origin, ConfigPathOrigin::ExplicitCli);
    }
}
