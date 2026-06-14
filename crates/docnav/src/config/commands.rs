use std::fs;

use serde_json::json;

use crate::cli::{ConfigCommand, ConfigGet, ConfigList, ConfigSet, ConfigUnset};
use crate::context::ProjectContext;
use crate::error::{AppError, AppResult};
use crate::output::CommandOutcome;
use crate::runtime::{self, DocnavRuntime};

use super::keys::{
    config_value_to_json, effective_key_value, effective_values, ensure_supported_key, set_key,
    supported_values_for_scope, unset_key,
};
use super::model::{ConfigSource, CoreConfig};
use super::store::{load_context, path_string, read_config, target_config_path, write_config};

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
    ensure_supported_key(&command.key)?;
    let context = load_context()?;
    let value = if command.user {
        super::keys::scoped_key_value(&command.key, &context.user_config, ConfigSource::User)?
    } else {
        effective_key_value(&command.key, &context)?
    };
    Ok(CommandOutcome::json(json!({
        "key": command.key,
        "value": value.value,
        "source": value.source,
    })))
}

fn config_set(command: ConfigSet) -> AppResult<CommandOutcome> {
    ensure_supported_key(&command.key)?;
    let context = load_context()?;
    let target_path = target_config_path(&context, command.user);
    let mut target_config = read_config(&target_path)?;
    set_key(
        &mut target_config,
        &command.key,
        &command.value,
        Some(&target_path),
    )?;
    write_config(&target_path, &target_config)?;
    Ok(CommandOutcome::json(json!({
        "ok": true,
        "scope": if command.user { "user" } else { "project" },
        "path": path_string(&target_path),
        "key": command.key,
        "value": config_value_to_json(&command.key, &target_config)?,
    })))
}

fn config_unset(command: ConfigUnset) -> AppResult<CommandOutcome> {
    ensure_supported_key(&command.key)?;
    let context = load_context()?;
    let target_path = target_config_path(&context, command.user);
    let mut target_config = read_config(&target_path)?;
    unset_key(&mut target_config, &command.key)?;
    write_config(&target_path, &target_config)?;
    Ok(CommandOutcome::json(json!({
        "ok": true,
        "scope": if command.user { "user" } else { "project" },
        "path": path_string(&target_path),
        "key": command.key,
    })))
}

fn config_list<T: DocnavRuntime>(command: ConfigList, runtime: &T) -> AppResult<CommandOutcome> {
    let context = load_context()?;
    let values = if command.user {
        supported_values_for_scope(&context.user_config, ConfigSource::User)?
    } else {
        effective_values(&context)?
    };
    let path_context = match command.path {
        Some(path) => {
            let (path, operation, defaults) =
                runtime::resolve_context_defaults(path, command.operation, &context)?;
            Some(runtime.describe_document_context(path, operation, defaults, &context)?)
        }
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
