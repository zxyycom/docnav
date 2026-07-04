use std::path::{Component, Path, PathBuf};

use crate::{NavigationConfigSources, NavigationError};

use super::config::{
    compile_path_pattern, mode_rules, ordered_config_sources, outline_config, RuleMode,
};

pub(super) fn resolve_path_rules(
    config_sources: &NavigationConfigSources,
    normalized_path: &str,
) -> Result<Option<RuleMode>, NavigationError> {
    let mut matched = None;
    for source in ordered_config_sources(config_sources) {
        let Some(outline) = outline_config(source)? else {
            continue;
        };
        for rule in mode_rules(source, outline)? {
            let regex = compile_path_pattern(source, "outline.mode_rules[].path", &rule.pattern)?;
            if regex.is_match(normalized_path) {
                matched = Some(rule.mode);
            }
        }
    }
    Ok(matched)
}

pub(super) fn normalized_document_path(
    document_path: &str,
    config_sources: &NavigationConfigSources,
) -> String {
    let path = Path::new(document_path);
    let relative = project_root(&config_sources.project.path)
        .and_then(|root| path.strip_prefix(root).ok())
        .unwrap_or(path);
    normalize_path_separators(relative)
}

fn project_root(project_config_path: &str) -> Option<PathBuf> {
    let path = Path::new(project_config_path);
    let parent = path.parent()?;
    if parent.file_name().is_some_and(|name| name == ".docnav") {
        return parent.parent().map(Path::to_path_buf);
    }
    parent.parent().map(Path::to_path_buf)
}

fn normalize_path_separators(path: &Path) -> String {
    let mut parts = Vec::<String>::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => parts.push(prefix.as_os_str().to_string_lossy().into()),
            Component::RootDir => parts.push(String::new()),
            Component::CurDir => {}
            Component::ParentDir => parts.push("..".to_owned()),
            Component::Normal(value) => parts.push(value.to_string_lossy().into()),
        }
    }
    if parts.is_empty() {
        return String::new();
    }
    parts.join("/")
}
