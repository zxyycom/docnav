use std::fs::{self, File};
use std::path::{Component, Path, PathBuf};

use docnav_protocol::StableError;

use crate::error::AppResult;
use crate::project_context::ProjectContext;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedDocumentPath {
    pub adapter_path: String,
    pub absolute_path: PathBuf,
}

pub fn normalize_document_path(
    project: &ProjectContext,
    input: &str,
) -> AppResult<NormalizedDocumentPath> {
    let raw_path = PathBuf::from(input);
    let resolved = if raw_path.is_absolute() {
        raw_path
    } else {
        project.cwd.join(raw_path)
    };

    let metadata = fs::metadata(&resolved).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            StableError::document_not_found(normalize_path_for_error(&resolved))
        } else {
            StableError::document_path_invalid(
                normalize_path_for_error(&resolved),
                error.to_string(),
            )
        }
    })?;
    if !metadata.is_file() {
        return Err(StableError::document_path_invalid(
            normalize_path_for_error(&resolved),
            "path is not a file",
        )
        .into());
    }
    File::open(&resolved).map_err(|error| {
        StableError::document_path_invalid(normalize_path_for_error(&resolved), error.to_string())
    })?;

    let absolute_path = fs::canonicalize(&resolved).map_err(|error| {
        StableError::document_path_invalid(normalize_path_for_error(&resolved), error.to_string())
    })?;
    let project_root =
        fs::canonicalize(&project.project_root).unwrap_or_else(|_| project.project_root.clone());

    let adapter_path = absolute_path
        .strip_prefix(&project_root)
        .ok()
        .filter(|relative| !relative.as_os_str().is_empty())
        .map(path_to_slash)
        .unwrap_or_else(|| path_to_slash(&absolute_path));

    Ok(NormalizedDocumentPath {
        adapter_path,
        absolute_path,
    })
}

pub fn resolve_project_relative_command(
    project_root: &Path,
    command: &str,
) -> Result<PathBuf, String> {
    if command.is_empty() {
        return Err("command must not be empty".to_owned());
    }

    let command_path = Path::new(command);
    if command_path.is_absolute() {
        return Err("command must be relative to the project root".to_owned());
    }

    let mut normalized = PathBuf::new();
    for component in command_path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err("command must not escape the project root".to_owned());
            }
            Component::Prefix(_) | Component::RootDir => {
                return Err("command must be relative to the project root".to_owned());
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err("command must not be empty".to_owned());
    }

    Ok(project_root.join(normalized))
}

pub fn path_to_slash(path: &Path) -> String {
    let mut text = path.display().to_string().replace('\\', "/");
    if let Some(stripped) = text.strip_prefix("//?/") {
        text = stripped.to_owned();
    }
    if let Some(stripped) = text.strip_prefix("//./") {
        text = stripped.to_owned();
    }
    text
}

fn normalize_path_for_error(path: &Path) -> String {
    path_to_slash(path)
}
