use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::{AppError, AppResult};

use super::{path_string, CoreConfig};

static CONFIG_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::config) enum ConfigCreateOutcome {
    Created,
    AlreadyExists,
}

pub(in crate::config) fn create_config_if_absent(
    path: &Path,
    config: &CoreConfig,
) -> AppResult<ConfigCreateOutcome> {
    ensure_parent_directory(path)?;
    let content = serde_json::to_string_pretty(config)
        .map_err(|_| AppError::internal("serialize-config-failed"))?;
    create_config_content_if_absent_with(path, format!("{content}\n").as_bytes(), |file, bytes| {
        file.write_all(bytes)
    })
}

fn ensure_parent_directory(path: &Path) -> AppResult<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    fs::create_dir_all(parent).map_err(|error| {
        AppError::invalid_request(
            "config",
            format!("failed to create {}: {error}", path_string(parent)),
        )
    })
}

pub(super) fn create_config_content_if_absent_with(
    path: &Path,
    content: &[u8],
    write_content: impl FnOnce(&mut File, &[u8]) -> io::Result<()>,
) -> AppResult<ConfigCreateOutcome> {
    if target_exists(path)? {
        return Ok(ConfigCreateOutcome::AlreadyExists);
    }

    let mut temporary = TemporaryConfigFile::create_for(path).map_err(|error| {
        AppError::invalid_request(
            "config",
            format!(
                "failed to prepare {} for atomic creation: {error}",
                path_string(path)
            ),
        )
    })?;
    if let Err(error) = write_content(temporary.file_mut(), content) {
        return Err(config_create_failure_with_cleanup(
            path,
            "write",
            error,
            &mut temporary,
        ));
    }

    temporary.close();
    let outcome = match fs::hard_link(temporary.path(), path) {
        Ok(()) => ConfigCreateOutcome::Created,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            ConfigCreateOutcome::AlreadyExists
        }
        Err(error) => {
            return Err(config_create_failure_with_cleanup(
                path,
                "publish",
                error,
                &mut temporary,
            ));
        }
    };
    cleanup_temporary_config(path, &mut temporary)?;
    Ok(outcome)
}

fn target_exists(path: &Path) -> AppResult<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(AppError::invalid_request(
            "config",
            format!("failed to inspect {}: {error}", path_string(path)),
        )),
    }
}

fn config_create_failure_with_cleanup(
    target: &Path,
    action: &str,
    failure: io::Error,
    temporary: &mut TemporaryConfigFile,
) -> AppError {
    let temporary_path = temporary.path().to_path_buf();
    match temporary.cleanup() {
        Ok(()) => AppError::invalid_request(
            "config",
            format!(
                "failed to {action} {}: {failure}",
                path_string(target)
            ),
        ),
        Err(cleanup_failure) => AppError::invalid_request(
            "config",
            format!(
                "failed to {action} {}: {failure}; failed to remove temporary file {}: {cleanup_failure}",
                path_string(target),
                path_string(&temporary_path)
            ),
        ),
    }
}

fn cleanup_temporary_config(target: &Path, temporary: &mut TemporaryConfigFile) -> AppResult<()> {
    let temporary_path = temporary.path().to_path_buf();
    temporary.cleanup().map_err(|error| {
        AppError::invalid_request(
            "config",
            format!(
                "created or preserved {}; failed to remove temporary file {}: {error}",
                path_string(target),
                path_string(&temporary_path)
            ),
        )
    })
}

struct TemporaryConfigFile {
    path: PathBuf,
    file: Option<File>,
}

impl TemporaryConfigFile {
    fn create_for(target: &Path) -> io::Result<Self> {
        let parent = target.parent().unwrap_or_else(|| Path::new("."));
        let target_name = target
            .file_name()
            .unwrap_or_else(|| OsStr::new("docnav-config"));
        for _ in 0..128 {
            let sequence = CONFIG_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let mut name = OsString::from(".");
            name.push(target_name);
            name.push(format!(".tmp-{}-{sequence}", std::process::id()));
            let path = parent.join(name);
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => {
                    return Ok(Self {
                        path,
                        file: Some(file),
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not allocate a unique temporary config file",
        ))
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn file_mut(&mut self) -> &mut File {
        self.file
            .as_mut()
            .expect("temporary config file remains open while being written")
    }

    fn close(&mut self) {
        self.file.take();
    }

    fn cleanup(&mut self) -> io::Result<()> {
        self.close();
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}

impl Drop for TemporaryConfigFile {
    fn drop(&mut self) {
        self.close();
        let _ = fs::remove_file(&self.path);
    }
}
