use std::fs;

use serde_json::json;

use crate::cli::{ConfigCommand, ConfigGet, ConfigList, ConfigSet, ConfigUnset};
use crate::error::{AppError, AppResult};
use crate::output::CommandOutcome;
use crate::project_context::ProjectContext;
use crate::registry::AdapterRegistry;
use crate::runtime::DocnavRuntime;

use super::keys::{
    config_value_to_json, effective_key_value, effective_values, ensure_supported_key, set_key,
    supported_values_for_scope, unset_key,
};
use super::model::{ConfigContext, ConfigSource, CoreConfig};
use super::store::{
    load_context, path_string, read_config, target_config_path, write_config, ConfigFileSource,
};

pub fn execute<T: DocnavRuntime>(command: ConfigCommand, runtime: &T) -> AppResult<CommandOutcome> {
    match command {
        ConfigCommand::Get(command) => config_get(command),
        ConfigCommand::Set(command) => config_set(command),
        ConfigCommand::Unset(command) => config_unset(command),
        ConfigCommand::List(command) => config_list(command, runtime),
    }
}

pub fn init_project() -> AppResult<CommandOutcome> {
    let project = ProjectContext::discover()?;
    let docnav_dir = project.project_root.join(".docnav");
    let config_path = docnav_dir.join("docnav.json");
    fs::create_dir_all(&docnav_dir)
        .map_err(|error| AppError::invalid_request("project_config", error.to_string()))?;
    let created = if config_path.exists() {
        false
    } else {
        write_config(&config_path, &CoreConfig::default())?;
        true
    };

    Ok(CommandOutcome::json(json!({
        "ok": true,
        "created": created,
        "project_root": path_string(&project.project_root),
        "config_path": path_string(&config_path),
    })))
}

fn config_get(command: ConfigGet) -> AppResult<CommandOutcome> {
    let context = load_context()?;
    let registry = AdapterRegistry::load(&context.project)?;
    ensure_supported_key(&command.key, &registry)?;
    let value = if command.user {
        super::keys::scoped_key_value(
            &command.key,
            &context.user_config,
            ConfigSource::User,
            &registry,
        )?
    } else {
        effective_key_value(&command.key, &context, &registry)?
    };
    Ok(CommandOutcome::json(json!({
        "key": command.key,
        "value": value.value,
        "source": value.source,
    })))
}

fn config_set(command: ConfigSet) -> AppResult<CommandOutcome> {
    let context = load_context()?;
    let registry = AdapterRegistry::load(&context.project)?;
    ensure_supported_key(&command.key, &registry)?;
    let (target_path, mut target_config) = load_target_config(&context, &registry, command.user)?;
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
    let context = load_context()?;
    let registry = AdapterRegistry::load(&context.project)?;
    ensure_supported_key(&command.key, &registry)?;
    let (target_path, mut target_config) = load_target_config(&context, &registry, command.user)?;
    unset_key(&mut target_config, &command.key, &registry)?;
    write_config(&target_path, &target_config)?;
    Ok(CommandOutcome::json(json!({
        "ok": true,
        "scope": if command.user { "user" } else { "project" },
        "path": path_string(&target_path),
        "key": command.key,
    })))
}

fn load_target_config(
    context: &ConfigContext,
    registry: &AdapterRegistry,
    user: bool,
) -> AppResult<(std::path::PathBuf, CoreConfig)> {
    let target_path = target_config_path(context, user);
    let target_config = read_config(&target_path, registry, config_file_source(user))?;
    Ok((target_path, target_config))
}

fn config_file_source(user: bool) -> ConfigFileSource {
    if user {
        ConfigFileSource::User
    } else {
        ConfigFileSource::Project
    }
}

fn config_list<T: DocnavRuntime>(command: ConfigList, runtime: &T) -> AppResult<CommandOutcome> {
    let context = load_context()?;
    let registry = AdapterRegistry::load(&context.project)?;
    let values = if command.user {
        supported_values_for_scope(&context.user_config, ConfigSource::User, &registry)?
    } else {
        effective_values(&context, &registry)?
    };
    let path_context = match command.path {
        Some(path) => Some(runtime.describe_document_context(path, command.operation, &context)?),
        None => None,
    };

    Ok(CommandOutcome::json(json!({
        "project_root": path_string(&context.project.project_root),
        "project_config": path_string(&context.project.project_config_path),
        "user_config": path_string(&context.project.user_config_path),
        "values": values,
        "path_context": path_context,
    })))
}
