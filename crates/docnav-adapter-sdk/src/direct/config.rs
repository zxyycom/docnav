use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use super::warnings::DirectCliWarning;

const DOCNAV_DIR: &str = ".docnav";

#[derive(Debug, Default, PartialEq)]
pub(super) struct AdapterDirectCliConfig {
    pub(super) limit_chars: Option<Value>,
    pub(super) output: Option<Value>,
    pub(super) native_options: Map<String, Value>,
}

#[derive(Default)]
pub(super) struct LoadedAdapterDirectCliConfig {
    pub(super) project: AdapterDirectCliConfig,
    pub(super) user: AdapterDirectCliConfig,
    pub(super) warnings: Vec<DirectCliWarning>,
}

#[derive(Default)]
pub(super) struct ConfigPathOverrides {
    pub(super) project: Option<PathBuf>,
    pub(super) user: Option<PathBuf>,
}

struct DirectCliConfigSourcePath {
    level: ConfigSourceLevel,
    origin: ConfigPathOrigin,
    path: PathBuf,
}

#[derive(Clone, Copy)]
enum ConfigSourceLevel {
    Project,
    User,
}

impl ConfigSourceLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::User => "user",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConfigPathOrigin {
    Default,
    Override,
}

impl ConfigPathOrigin {
    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Override => "override",
        }
    }
}

#[derive(Clone, Copy)]
enum ConfigSourceSkipReason {
    MissingOverride,
    NotFile,
    Unreadable,
    InvalidJson,
    NonObject,
}

impl ConfigSourceSkipReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::MissingOverride => "missing_override",
            Self::NotFile => "not_file",
            Self::Unreadable => "unreadable",
            Self::InvalidJson => "invalid_json",
            Self::NonObject => "non_object",
        }
    }
}

pub(super) fn load_adapter_direct_cli_config(
    adapter_id: &str,
    default_user_config_dir: Option<&Path>,
    startup_cwd: &Path,
    overrides: ConfigPathOverrides,
) -> LoadedAdapterDirectCliConfig {
    let (project_path, user_path) =
        resolve_config_paths(adapter_id, default_user_config_dir, startup_cwd, overrides);
    let project = load_config_source(&project_path);
    let user = load_config_source(&user_path);

    LoadedAdapterDirectCliConfig {
        project: project.config,
        user: user.config,
        warnings: [project.warning, user.warning]
            .into_iter()
            .flatten()
            .collect(),
    }
}

fn resolve_config_paths(
    adapter_id: &str,
    default_user_config_dir: Option<&Path>,
    startup_cwd: &Path,
    overrides: ConfigPathOverrides,
) -> (DirectCliConfigSourcePath, DirectCliConfigSourcePath) {
    let config_file_name = format!("{adapter_id}.json");
    let project_root = find_project_root(startup_cwd);
    let default_project_path = project_root.join(DOCNAV_DIR).join(&config_file_name);
    let user_config_dir = default_user_config_dir.unwrap_or(startup_cwd);
    let default_user_path = user_config_dir.join(config_file_name);

    (
        config_source_path(
            ConfigSourceLevel::Project,
            startup_cwd,
            overrides.project,
            default_project_path,
        ),
        config_source_path(
            ConfigSourceLevel::User,
            startup_cwd,
            overrides.user,
            default_user_path,
        ),
    )
}

#[derive(Default)]
struct LoadedConfigSource {
    config: AdapterDirectCliConfig,
    warning: Option<DirectCliWarning>,
}

fn load_config_source(source: &DirectCliConfigSourcePath) -> LoadedConfigSource {
    match fs::metadata(&source.path) {
        Ok(metadata) if !metadata.is_file() => skipped(source, ConfigSourceSkipReason::NotFile),
        Ok(_) => read_config_source(source),
        Err(error) if error.kind() == io::ErrorKind::NotFound => missing_source(source),
        Err(_) => skipped(source, ConfigSourceSkipReason::Unreadable),
    }
}

fn read_config_source(source: &DirectCliConfigSourcePath) -> LoadedConfigSource {
    let content = match fs::read_to_string(&source.path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return missing_source(source),
        Err(_) => return skipped(source, ConfigSourceSkipReason::Unreadable),
    };
    let value = match serde_json::from_str::<Value>(&content) {
        Ok(value) => value,
        Err(_) => return skipped(source, ConfigSourceSkipReason::InvalidJson),
    };
    let Value::Object(object) = value else {
        return skipped(source, ConfigSourceSkipReason::NonObject);
    };

    LoadedConfigSource {
        config: adapter_config_values(&object),
        warning: None,
    }
}

fn missing_source(source: &DirectCliConfigSourcePath) -> LoadedConfigSource {
    match source.origin {
        ConfigPathOrigin::Default => LoadedConfigSource::default(),
        ConfigPathOrigin::Override => skipped(source, ConfigSourceSkipReason::MissingOverride),
    }
}

fn skipped(
    source: &DirectCliConfigSourcePath,
    reason: ConfigSourceSkipReason,
) -> LoadedConfigSource {
    LoadedConfigSource {
        config: AdapterDirectCliConfig::default(),
        warning: Some(DirectCliWarning::adapter_config_source_skipped(
            source.level.as_str(),
            source.origin.as_str(),
            &source.path.display().to_string(),
            reason.as_str(),
        )),
    }
}

fn adapter_config_values(object: &Map<String, Value>) -> AdapterDirectCliConfig {
    let defaults = object.get("defaults").and_then(Value::as_object);
    AdapterDirectCliConfig {
        limit_chars: defaults.and_then(|value| value.get("limit_chars")).cloned(),
        output: defaults.and_then(|value| value.get("output")).cloned(),
        native_options: object
            .get("options")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default(),
    }
}

fn find_project_root(startup_cwd: &Path) -> PathBuf {
    for candidate in startup_cwd.ancestors() {
        if candidate.join(DOCNAV_DIR).is_dir() {
            return candidate.to_path_buf();
        }
    }
    startup_cwd.to_path_buf()
}

fn config_source_path(
    level: ConfigSourceLevel,
    startup_cwd: &Path,
    override_path: Option<PathBuf>,
    default_path: PathBuf,
) -> DirectCliConfigSourcePath {
    let (origin, path) = match override_path {
        Some(path) => (ConfigPathOrigin::Override, absolutize(startup_cwd, path)),
        None => (ConfigPathOrigin::Default, default_path),
    };
    DirectCliConfigSourcePath {
        level,
        origin,
        path,
    }
}

fn absolutize(startup_cwd: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        startup_cwd.join(path)
    }
}

#[cfg(test)]
mod tests;
