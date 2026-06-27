use std::path::{Path, PathBuf};

use docnav_standard_parameters::{
    ConfigPathOrigin as StandardConfigPathOrigin, ConfigSourceLevel as StandardConfigSourceLevel,
    StandardParameterConfigSourceDescriptor,
};

const DOCNAV_DIR: &str = ".docnav";

#[derive(Default)]
pub(super) struct ConfigPathOverrides {
    pub(super) project: Option<PathBuf>,
    pub(super) user: Option<PathBuf>,
}

pub(super) struct AdapterDirectCliConfigSourceDescriptors {
    pub(super) project: StandardParameterConfigSourceDescriptor,
    pub(super) user: StandardParameterConfigSourceDescriptor,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConfigPathOrigin {
    Default,
    Override,
}

pub(super) fn adapter_direct_cli_config_source_descriptors(
    adapter_id: &str,
    default_user_config_dir: Option<&Path>,
    startup_cwd: &Path,
    overrides: ConfigPathOverrides,
) -> AdapterDirectCliConfigSourceDescriptors {
    let (project, user) =
        resolve_config_paths(adapter_id, default_user_config_dir, startup_cwd, overrides);
    AdapterDirectCliConfigSourceDescriptors {
        project: standard_descriptor(project),
        user: standard_descriptor(user),
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

fn standard_descriptor(
    source: DirectCliConfigSourcePath,
) -> StandardParameterConfigSourceDescriptor {
    StandardParameterConfigSourceDescriptor::new(
        standard_level(source.level),
        standard_origin(source.origin),
        source.path,
    )
}

fn standard_level(level: ConfigSourceLevel) -> StandardConfigSourceLevel {
    match level {
        ConfigSourceLevel::Project => StandardConfigSourceLevel::Project,
        ConfigSourceLevel::User => StandardConfigSourceLevel::User,
    }
}

fn standard_origin(origin: ConfigPathOrigin) -> StandardConfigPathOrigin {
    match origin {
        ConfigPathOrigin::Default => StandardConfigPathOrigin::Default,
        ConfigPathOrigin::Override => StandardConfigPathOrigin::Override,
    }
}

#[cfg(test)]
mod tests;
