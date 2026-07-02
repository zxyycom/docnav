use std::fs::{self, File};
use std::path::{Path, PathBuf};

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
    let raw_path = PathBuf::from(input);
    let resolved = if raw_path.is_absolute() {
        raw_path
    } else {
        project.cwd.join(raw_path)
    };

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
