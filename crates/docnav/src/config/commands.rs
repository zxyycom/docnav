use std::fs;
use std::path::PathBuf;

use serde_json::json;

use crate::cli::{ConfigCommand, ConfigGet, ConfigList, ConfigPathArgs, ConfigSet, ConfigUnset};
use crate::error::{AppError, AppResult};
use crate::output::CommandOutcome;
use crate::project_context::{ProjectContext, SelectedConfigPath};
use crate::registry::AdapterRegistry;
use crate::runtime::DocnavRuntime;

use super::keys::{
    config_value_to_json, effective_key_value, effective_values, ensure_supported_key, set_key,
    supported_values_for_scope, unset_key,
};
use super::model::{ConfigSource, CoreConfig};
use super::store::{
    load_context, path_string, read_config_for_update, read_selected_config, write_config,
    ConfigFileSource,
};

pub fn execute<T: DocnavRuntime>(command: ConfigCommand, runtime: &T) -> AppResult<CommandOutcome> {
    match command {
        ConfigCommand::Get(command) => config_get(command),
        ConfigCommand::Set(command) => config_set(command),
        ConfigCommand::Unset(command) => config_unset(command),
        ConfigCommand::List(command) => config_list(command, runtime),
    }
}

pub fn init_project(config_paths: ConfigPathArgs) -> AppResult<CommandOutcome> {
    let target =
        ProjectContext::discover_project_config_target(config_paths.project_config.as_deref())?;
    let config_path = target.config_path.path;
    let config_dir = config_path.parent().ok_or_else(|| {
        AppError::invalid_request("project_config", "project config path has no parent")
    })?;
    fs::create_dir_all(config_dir)
        .map_err(|error| AppError::invalid_request("project_config", error.to_string()))?;
    let created = if config_path.exists() {
        if !config_path.is_file() {
            return Err(AppError::invalid_request(
                "project_config",
                format!("{} is not a file", path_string(&config_path)),
            ));
        }
        false
    } else {
        write_config(&config_path, &CoreConfig::default())?;
        true
    };

    Ok(CommandOutcome::json(json!({
        "ok": true,
        "created": created,
        "project_root": path_string(&target.project_root),
        "config_path": path_string(&config_path),
    })))
}

fn config_get(command: ConfigGet) -> AppResult<CommandOutcome> {
    if command.user {
        let (registry, user_config) = load_user_config_values(&command.config_paths)?;
        ensure_supported_key(&command.key, &registry)?;
        let value = super::keys::scoped_key_value(
            &command.key,
            &user_config,
            ConfigSource::User,
            &registry,
        )?;
        return Ok(CommandOutcome::json(json!({
            "key": command.key,
            "value": value.value,
            "source": value.source,
        })));
    }

    let context = load_context(
        command.config_paths.project_config.as_deref(),
        command.config_paths.user_config.as_deref(),
    )?;
    let registry = AdapterRegistry::load(&context.project)?;
    ensure_supported_key(&command.key, &registry)?;
    let value = effective_key_value(&command.key, &context, &registry)?;
    Ok(CommandOutcome::json(json!({
        "key": command.key,
        "value": value.value,
        "source": value.source,
    })))
}

fn config_set(command: ConfigSet) -> AppResult<CommandOutcome> {
    let (registry, target_path, mut target_config) =
        load_config_mutation_target(&command.config_paths, &command.key, command.user)?;
    set_key(
        &mut target_config,
        &command.key,
        &command.value,
        Some(&target_path),
        &registry,
    )?;
    write_config(&target_path, &target_config)?;
    Ok(CommandOutcome::json(json!({
        "ok": true,
        "scope": if command.user { "user" } else { "project" },
        "path": path_string(&target_path),
        "key": command.key,
        "value": config_value_to_json(&command.key, &target_config, &registry)?,
    })))
}

fn config_unset(command: ConfigUnset) -> AppResult<CommandOutcome> {
    let (registry, target_path, mut target_config) =
        load_config_mutation_target(&command.config_paths, &command.key, command.user)?;
    unset_key(&mut target_config, &command.key, &registry)?;
    write_config(&target_path, &target_config)?;
    Ok(CommandOutcome::json(json!({
        "ok": true,
        "scope": if command.user { "user" } else { "project" },
        "path": path_string(&target_path),
        "key": command.key,
    })))
}

fn load_config_mutation_target(
    config_paths: &ConfigPathArgs,
    key: &str,
    user: bool,
) -> AppResult<(AdapterRegistry, PathBuf, CoreConfig)> {
    let source = config_file_source(user);
    let target = load_config_path_target(config_paths, user)?;
    let registry = AdapterRegistry::builtin();
    ensure_supported_key(key, &registry)?;
    let target_path = target.path.clone();
    let target_config = read_config_for_update(&target, &registry, source)?;
    Ok((registry, target_path, target_config))
}

fn config_file_source(user: bool) -> ConfigFileSource {
    if user {
        ConfigFileSource::User
    } else {
        ConfigFileSource::Project
    }
}

fn load_config_path_target(
    config_paths: &ConfigPathArgs,
    user: bool,
) -> AppResult<SelectedConfigPath> {
    if user {
        ProjectContext::discover_user_config_path(config_paths.user_config.as_deref())
    } else {
        ProjectContext::discover_project_config_target(config_paths.project_config.as_deref())
            .map(|target| target.config_path)
    }
}

fn config_list<T: DocnavRuntime>(command: ConfigList, runtime: &T) -> AppResult<CommandOutcome> {
    if command.user {
        return config_list_user_scope(command, runtime);
    }
    config_list_effective(command, runtime)
}

fn config_list_user_scope<T: DocnavRuntime>(
    command: ConfigList,
    runtime: &T,
) -> AppResult<CommandOutcome> {
    let (project, registry, user_config) = load_user_scope_config(&command.config_paths)?;
    let values = supported_values_for_scope(&user_config, ConfigSource::User, &registry)?;
    let path_context = describe_path_context(command.path, command.operation, &project, runtime)?;

    Ok(config_list_outcome(&project, values, path_context))
}

fn config_list_effective<T: DocnavRuntime>(
    command: ConfigList,
    runtime: &T,
) -> AppResult<CommandOutcome> {
    let context = load_context(
        command.config_paths.project_config.as_deref(),
        command.config_paths.user_config.as_deref(),
    )?;
    let registry = AdapterRegistry::load(&context.project)?;
    let values = effective_values(&context, &registry)?;
    let path_context =
        describe_path_context(command.path, command.operation, &context.project, runtime)?;

    Ok(config_list_outcome(&context.project, values, path_context))
}

fn describe_path_context<T: DocnavRuntime>(
    path: Option<String>,
    operation: Option<docnav_protocol::Operation>,
    project: &ProjectContext,
    runtime: &T,
) -> AppResult<Option<crate::runtime::DocumentContextOutput>> {
    path.map(|path| runtime.describe_document_context(path, operation, project))
        .transpose()
}

fn config_list_outcome(
    project: &ProjectContext,
    values: Vec<serde_json::Value>,
    path_context: Option<crate::runtime::DocumentContextOutput>,
) -> CommandOutcome {
    CommandOutcome::json(json!({
        "project_root": path_string(&project.project_root),
        "project_config": path_string(project.project_config_path()),
        "user_config": path_string(project.user_config_path()),
        "values": values,
        "path_context": path_context,
    }))
}

fn load_user_scope_config(
    config_paths: &ConfigPathArgs,
) -> AppResult<(ProjectContext, AdapterRegistry, CoreConfig)> {
    let project = ProjectContext::discover_with_cli_config_paths(
        config_paths.project_config.as_deref(),
        config_paths.user_config.as_deref(),
    )?;
    let registry = AdapterRegistry::load(&project)?;
    let user_config = read_selected_config(
        ConfigFileSource::User.selected_path(&project),
        &registry,
        ConfigFileSource::User,
    )?;
    Ok((project, registry, user_config))
}

fn load_user_config_values(
    config_paths: &ConfigPathArgs,
) -> AppResult<(AdapterRegistry, CoreConfig)> {
    let target = ProjectContext::discover_user_config_path(config_paths.user_config.as_deref())?;
    let registry = AdapterRegistry::builtin();
    let user_config = read_selected_config(&target, &registry, ConfigFileSource::User)?;
    Ok((registry, user_config))
}

#[cfg(test)]
mod tests;
