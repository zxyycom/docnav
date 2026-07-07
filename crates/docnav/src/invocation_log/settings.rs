use std::path::PathBuf;

use crate::cli::DocumentCommand;
use crate::config::CoreConfig;
use crate::project_context::ProjectContext;

use super::paths::resolve_project_path;

#[derive(Clone, Debug, Default)]
pub(super) struct InvocationLogSettings {
    pub(super) sink_path: Option<PathBuf>,
    pub(super) content_capture_root: Option<PathBuf>,
}

impl InvocationLogSettings {
    pub(super) fn from_explicit_cli(command: &DocumentCommand, project: &ProjectContext) -> Self {
        Self {
            sink_path: command
                .invocation_log
                .as_deref()
                .map(|path| resolve_project_path(project, path)),
            content_capture_root: command
                .invocation_log
                .as_ref()
                .and(command.invocation_log_content_root.as_deref())
                .map(|path| resolve_project_path(project, path)),
        }
    }

    pub(super) fn resolve(
        command: &DocumentCommand,
        project: &ProjectContext,
        project_config: &CoreConfig,
        user_config: &CoreConfig,
    ) -> Self {
        let cli_log_enabled = command.invocation_log.is_some();
        let sink_path = resolve_sink_path(command, project, project_config, user_config);
        let content_capture_root = sink_path.as_ref().and_then(|_| {
            resolve_content_capture_root(
                command,
                project,
                project_config,
                user_config,
                cli_log_enabled,
            )
        });

        Self {
            sink_path,
            content_capture_root,
        }
    }
}

fn resolve_sink_path(
    command: &DocumentCommand,
    project: &ProjectContext,
    project_config: &CoreConfig,
    user_config: &CoreConfig,
) -> Option<PathBuf> {
    command
        .invocation_log
        .as_deref()
        .map(|path| resolve_project_path(project, path))
        .or_else(|| config_sink_path(project, project_config, user_config))
}

fn resolve_content_capture_root(
    command: &DocumentCommand,
    project: &ProjectContext,
    project_config: &CoreConfig,
    user_config: &CoreConfig,
    cli_log_enabled: bool,
) -> Option<PathBuf> {
    let config_root = || config_content_capture_root(project, project_config, user_config);
    if !cli_log_enabled {
        return config_root();
    }
    command
        .invocation_log_content_root
        .as_deref()
        .map(|path| resolve_project_path(project, path))
        .or_else(config_root)
}

fn config_sink_path(
    project: &ProjectContext,
    project_config: &CoreConfig,
    user_config: &CoreConfig,
) -> Option<PathBuf> {
    let enabled = project_config
        .invocation_log
        .enabled
        .or(user_config.invocation_log.enabled)
        .unwrap_or(false);
    let path = project_config
        .invocation_log
        .path
        .as_deref()
        .or(user_config.invocation_log.path.as_deref())?;
    enabled.then(|| resolve_project_path(project, path))
}

fn config_content_capture_root(
    project: &ProjectContext,
    project_config: &CoreConfig,
    user_config: &CoreConfig,
) -> Option<PathBuf> {
    let enabled = project_config
        .invocation_log
        .content_capture
        .enabled
        .or(user_config.invocation_log.content_capture.enabled)
        .unwrap_or(false);
    let root = project_config
        .invocation_log
        .content_capture
        .root
        .as_deref()
        .or(user_config.invocation_log.content_capture.root.as_deref())?;
    enabled.then(|| resolve_project_path(project, root))
}
