use std::fs::{self, File};
use std::path::{Component, Path, PathBuf};

use crate::error::{AppError, AppResult};
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
    let resolved = resolve_document_path(project, input);

    let metadata = fs::metadata(&resolved).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            AppError::document_not_found(normalize_path_for_error(&resolved))
        } else {
            AppError::document_path_invalid(normalize_path_for_error(&resolved), error.to_string())
        }
    })?;
    if !metadata.is_file() {
        return Err(AppError::document_path_invalid(
            normalize_path_for_error(&resolved),
            "path is not a file",
        ));
    }
    File::open(&resolved).map_err(|error| {
        AppError::document_path_invalid(normalize_path_for_error(&resolved), error.to_string())
    })?;

    let absolute_path = fs::canonicalize(&resolved).map_err(|error| {
        AppError::document_path_invalid(normalize_path_for_error(&resolved), error.to_string())
    })?;
    let adapter_path = path_to_slash(&absolute_path);

    Ok(NormalizedDocumentPath {
        adapter_path,
        absolute_path,
    })
}

pub(crate) fn routing_document_pathname(project: &ProjectContext, input: &str) -> String {
    path_to_slash(&resolve_document_path(project, input))
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

pub(crate) fn resolve_document_path(project: &ProjectContext, input: &str) -> PathBuf {
    let raw_path = PathBuf::from(input);
    let resolved = if raw_path.is_absolute() {
        raw_path
    } else {
        project.cwd.join(raw_path)
    };
    normalize_lexically(&resolved)
}

fn normalize_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => match normalized.components().next_back() {
                Some(Component::Normal(_)) => {
                    normalized.pop();
                }
                Some(Component::ParentDir) | None if !path.has_root() => {
                    normalized.push(component.as_os_str());
                }
                Some(Component::Prefix(_))
                | Some(Component::RootDir)
                | Some(Component::ParentDir)
                | Some(Component::CurDir)
                | None => {}
            },
        }
    }
    normalized
}
