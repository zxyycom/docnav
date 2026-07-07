use std::path::PathBuf;

use crate::project_context::ProjectContext;

pub(super) fn resolve_project_path(project: &ProjectContext, path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        project.project_root.join(path)
    }
}
