use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::cli::{ConfigCommand, ConfigGet, ConfigList, ConfigSet, ConfigUnset, OutputMode};
use crate::context::ProjectContext;
use crate::contract::manifest_from_output;
use crate::error::{AppError, AppResult, DocnavExitCode};
use crate::output::CommandOutcome;
use crate::process::run_manifest;
use crate::project::path_to_slash;
use crate::registry::{self, AdapterRegistry};
use crate::runtime::{self, DocnavRuntime};

const DEFAULT_LIMIT_CHARS: u32 = 6000;
const DEFAULT_OUTPUT: OutputMode = OutputMode::Text;
const SUPPORTED_KEYS: [&str; 3] = [
    "defaults.adapter",
    "defaults.limit_chars",
    "defaults.output",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigContext {
    pub project: ProjectContext,
    pub project_config: CoreConfig,
    pub user_config: CoreConfig,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CoreConfig {
    #[serde(default, skip_serializing_if = "DefaultsConfig::is_empty")]
    pub defaults: DefaultsConfig,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct DefaultsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_chars: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<OutputMode>,
}

impl DefaultsConfig {
    fn is_empty(&self) -> bool {
        self.adapter.is_none() && self.limit_chars.is_none() && self.output.is_none()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigSource {
    Explicit,
    Project,
    User,
    BuiltIn,
    Unset,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ResolvedValue {
    pub value: Value,
    pub source: String,
}

impl ResolvedValue {
    pub fn explicit(value: Value) -> Self {
        Self::new(value, ConfigSource::Explicit)
    }

    pub fn project(value: Value) -> Self {
        Self::new(value, ConfigSource::Project)
    }

    pub fn user(value: Value) -> Self {
        Self::new(value, ConfigSource::User)
    }

    pub fn built_in(value: Value) -> Self {
        Self::new(value, ConfigSource::BuiltIn)
    }

    pub fn unset() -> Self {
        Self::new(Value::Null, ConfigSource::Unset)
    }

    fn new(value: Value, source: ConfigSource) -> Self {
        let source = serde_json::to_value(source)
            .ok()
            .and_then(|value| value.as_str().map(str::to_owned))
            .unwrap_or_else(|| "unknown".to_owned());
        Self { value, source }
    }
}

pub fn execute<T: DocnavRuntime>(command: ConfigCommand, runtime: &T) -> AppResult<CommandOutcome> {
    match command {
        ConfigCommand::Get(command) => config_get(command),
        ConfigCommand::Set(command) => config_set(command),
        ConfigCommand::Unset(command) => config_unset(command),
        ConfigCommand::List(command) => config_list(command, runtime),
    }
}

pub fn load_context() -> AppResult<ConfigContext> {
    let project = ProjectContext::discover()?;
    let project_config = read_config(&project.project_config_path)?;
    let user_config = read_config(&project.user_config_path)?;
    Ok(ConfigContext {
        project,
        project_config,
        user_config,
    })
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

pub fn doctor() -> AppResult<CommandOutcome> {
    let project = ProjectContext::discover()?;
    let mut checks = Vec::new();
    checks.push(check_config_file(
        "project_config",
        &project.project_config_path,
    ));
    checks.push(check_config_file("user_config", &project.user_config_path));
    let registry_path = registry::registry_path(&project);
    let registry = AdapterRegistry::load(&project);
    checks.push(registry::registry_check(&registry_path, &registry));
    if let Ok(registry) = &registry {
        checks.extend(adapter_manifest_checks(&project, registry));
    }

    let exit_code = most_severe_exit(&checks);

    Ok(CommandOutcome::json_with_exit(
        json!({
            "project_root": path_string(&project.project_root),
            "checks": checks,
        }),
        exit_code,
    ))
}

pub fn resolve_adapter(explicit: Option<&str>, context: &ConfigContext) -> ResolvedValue {
    if let Some(adapter) = explicit {
        return ResolvedValue::explicit(json!(adapter));
    }
    if let Some(adapter) = &context.project_config.defaults.adapter {
        return ResolvedValue::project(json!(adapter));
    }
    if let Some(adapter) = &context.user_config.defaults.adapter {
        return ResolvedValue::user(json!(adapter));
    }
    ResolvedValue::unset()
}

pub fn resolve_limit_chars(
    explicit: Option<docnav_protocol::PositiveInteger>,
    context: &ConfigContext,
) -> AppResult<ResolvedValue> {
    if let Some(limit_chars) = explicit {
        return Ok(ResolvedValue::explicit(json!(limit_chars.get())));
    }
    if let Some(limit_chars) = context.project_config.defaults.limit_chars {
        validate_positive_key("defaults.limit_chars", limit_chars)?;
        return Ok(ResolvedValue::project(json!(limit_chars)));
    }
    if let Some(limit_chars) = context.user_config.defaults.limit_chars {
        validate_positive_key("defaults.limit_chars", limit_chars)?;
        return Ok(ResolvedValue::user(json!(limit_chars)));
    }
    Ok(ResolvedValue::built_in(json!(DEFAULT_LIMIT_CHARS)))
}

pub fn resolve_output(
    explicit: Option<OutputMode>,
    context: &ConfigContext,
) -> AppResult<ResolvedValue> {
    if let Some(output) = explicit {
        return Ok(ResolvedValue::explicit(json!(output.as_str())));
    }
    if let Some(output) = context.project_config.defaults.output {
        return Ok(ResolvedValue::project(json!(output.as_str())));
    }
    if let Some(output) = context.user_config.defaults.output {
        return Ok(ResolvedValue::user(json!(output.as_str())));
    }
    Ok(ResolvedValue::built_in(json!(DEFAULT_OUTPUT.as_str())))
}

fn config_get(command: ConfigGet) -> AppResult<CommandOutcome> {
    ensure_supported_key(&command.key)?;
    let context = load_context()?;
    let value = if command.user {
        scoped_key_value(&command.key, &context.user_config, ConfigSource::User)?
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
    set_key(&mut target_config, &command.key, &command.value)?;
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

fn read_config(path: &Path) -> AppResult<CoreConfig> {
    if !path.exists() {
        return Ok(CoreConfig::default());
    }
    let content = fs::read_to_string(path).map_err(|error| {
        AppError::invalid_request(
            "config",
            format!("failed to read {}: {error}", path_string(path)),
        )
    })?;
    let config: CoreConfig = serde_json::from_str(&content).map_err(|error| {
        AppError::invalid_request(
            "config",
            format!("failed to parse {}: {error}", path_string(path)),
        )
    })?;
    validate_config(&config, path)?;
    Ok(config)
}

fn write_config(path: &Path, config: &CoreConfig) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::invalid_request(
                "config",
                format!("failed to create {}: {error}", path_string(parent)),
            )
        })?;
    }
    let content = serde_json::to_string_pretty(config)
        .map_err(|error| AppError::internal(format!("serialize-config:{error}")))?;
    fs::write(path, format!("{content}\n")).map_err(|error| {
        AppError::invalid_request(
            "config",
            format!("failed to write {}: {error}", path_string(path)),
        )
    })
}

fn validate_config(config: &CoreConfig, path: &Path) -> AppResult<()> {
    if let Some(adapter) = &config.defaults.adapter {
        if adapter.is_empty() {
            return Err(AppError::invalid_request(
                "defaults.adapter",
                format!("{} contains an empty adapter id", path_string(path)),
            ));
        }
    }
    if let Some(limit_chars) = config.defaults.limit_chars {
        validate_positive_key("defaults.limit_chars", limit_chars)?;
    }
    Ok(())
}

fn check_config_file(name: &str, path: &Path) -> Value {
    match read_config(path) {
        Ok(_) if path.exists() => json!({
            "name": name,
            "status": "pass",
            "path": path_string(path),
            "message": "config file is readable"
        }),
        Ok(_) => json!({
            "name": name,
            "status": "pass",
            "path": path_string(path),
            "message": "config file is absent; built-in defaults apply"
        }),
        Err(error) => json!({
            "name": name,
            "status": "fail",
            "path": path_string(path),
            "message": error.error().message,
            "details": error.error().details,
        }),
    }
}

fn adapter_manifest_checks(project: &ProjectContext, registry: &AdapterRegistry) -> Vec<Value> {
    if registry.adapters.is_empty() {
        return vec![json!({
            "name": "adapter_manifest_checks",
            "status": "pass",
            "message": "no adapters are registered"
        })];
    }

    registry
        .adapters
        .iter()
        .map(
            |adapter| match run_manifest(&project.project_root, adapter) {
                Ok(output) => match manifest_from_output(&adapter.id, output) {
                    Ok(manifest) => json!({
                        "name": "adapter_manifest",
                        "status": "pass",
                        "adapter_id": adapter.id,
                        "command": adapter.command,
                        "manifest_adapter_id": manifest.adapter.id,
                        "message": "adapter manifest passed current contract checks"
                    }),
                    Err(reason) => json!({
                        "name": "adapter_manifest",
                        "status": "fail",
                        "adapter_id": adapter.id,
                        "command": adapter.command,
                        "message": reason,
                        "exit_code": DocnavExitCode::AdapterOrProtocolError.code(),
                    }),
                },
                Err(error) => json!({
                    "name": "adapter_manifest",
                    "status": "fail",
                    "adapter_id": adapter.id,
                    "command": adapter.command,
                    "message": error.reason,
                    "stderr": error.stderr,
                    "exit_code": DocnavExitCode::AdapterOrProtocolError.code(),
                }),
            },
        )
        .collect()
}

fn most_severe_exit(checks: &[Value]) -> DocnavExitCode {
    checks
        .iter()
        .filter(|check| check.get("status").and_then(Value::as_str) == Some("fail"))
        .filter_map(|check| check.get("exit_code").and_then(Value::as_i64))
        .map(|code| match code {
            4 => DocnavExitCode::AdapterOrProtocolError,
            3 => DocnavExitCode::DocumentError,
            2 => DocnavExitCode::InputError,
            1 => DocnavExitCode::InternalError,
            _ => DocnavExitCode::InternalError,
        })
        .max_by_key(|code| severity(*code))
        .unwrap_or_else(|| {
            if checks
                .iter()
                .any(|check| check.get("status").and_then(Value::as_str) == Some("fail"))
            {
                DocnavExitCode::InputError
            } else {
                DocnavExitCode::Success
            }
        })
}

fn severity(code: DocnavExitCode) -> u8 {
    match code {
        DocnavExitCode::Success => 0,
        DocnavExitCode::InternalError => 1,
        DocnavExitCode::InputError => 2,
        DocnavExitCode::DocumentError => 3,
        DocnavExitCode::AdapterOrProtocolError => 4,
    }
}

fn target_config_path(context: &ConfigContext, user: bool) -> PathBuf {
    if user {
        context.project.user_config_path.clone()
    } else {
        context.project.project_config_path.clone()
    }
}

fn supported_values_for_scope(config: &CoreConfig, source: ConfigSource) -> AppResult<Vec<Value>> {
    SUPPORTED_KEYS
        .iter()
        .map(|key| {
            scoped_key_value(key, config, source.clone()).map(|resolved| {
                json!({
                    "key": key,
                    "value": resolved.value,
                    "source": resolved.source,
                })
            })
        })
        .collect()
}

fn effective_values(context: &ConfigContext) -> AppResult<Vec<Value>> {
    SUPPORTED_KEYS
        .iter()
        .map(|key| {
            effective_key_value(key, context).map(|resolved| {
                json!({
                    "key": key,
                    "value": resolved.value,
                    "source": resolved.source,
                })
            })
        })
        .collect()
}

fn effective_key_value(key: &str, context: &ConfigContext) -> AppResult<ResolvedValue> {
    match key {
        "defaults.adapter" => Ok(resolve_adapter(None, context)),
        "defaults.limit_chars" => resolve_limit_chars(None, context),
        "defaults.output" => resolve_output(None, context),
        _ => Err(unknown_key(key)),
    }
}

fn scoped_key_value(
    key: &str,
    config: &CoreConfig,
    source: ConfigSource,
) -> AppResult<ResolvedValue> {
    match key {
        "defaults.adapter" => Ok(config
            .defaults
            .adapter
            .as_ref()
            .map(|value| ResolvedValue::new(json!(value), source.clone()))
            .unwrap_or_else(ResolvedValue::unset)),
        "defaults.limit_chars" => Ok(config
            .defaults
            .limit_chars
            .map(|value| ResolvedValue::new(json!(value), source.clone()))
            .unwrap_or_else(ResolvedValue::unset)),
        "defaults.output" => Ok(config
            .defaults
            .output
            .map(|value| ResolvedValue::new(json!(value.as_str()), source))
            .unwrap_or_else(ResolvedValue::unset)),
        _ => Err(unknown_key(key)),
    }
}

fn set_key(config: &mut CoreConfig, key: &str, value: &str) -> AppResult<()> {
    match key {
        "defaults.adapter" => {
            if value.is_empty() {
                return Err(AppError::invalid_request(
                    key,
                    "adapter id must not be empty",
                ));
            }
            config.defaults.adapter = Some(value.to_owned());
        }
        "defaults.limit_chars" => {
            let limit_chars = value.parse::<u32>().map_err(|_| {
                AppError::invalid_request(key, "defaults.limit_chars must be a positive integer")
            })?;
            validate_positive_key(key, limit_chars)?;
            config.defaults.limit_chars = Some(limit_chars);
        }
        "defaults.output" => {
            config.defaults.output = Some(
                value
                    .parse()
                    .map_err(|reason: String| AppError::invalid_request(key, reason))?,
            );
        }
        _ => return Err(unknown_key(key)),
    }
    Ok(())
}

fn unset_key(config: &mut CoreConfig, key: &str) -> AppResult<()> {
    match key {
        "defaults.adapter" => config.defaults.adapter = None,
        "defaults.limit_chars" => config.defaults.limit_chars = None,
        "defaults.output" => config.defaults.output = None,
        _ => return Err(unknown_key(key)),
    }
    Ok(())
}

fn config_value_to_json(key: &str, config: &CoreConfig) -> AppResult<Value> {
    Ok(match key {
        "defaults.adapter" => config
            .defaults
            .adapter
            .as_ref()
            .map(|value| json!(value))
            .unwrap_or(Value::Null),
        "defaults.limit_chars" => config
            .defaults
            .limit_chars
            .map(|value| json!(value))
            .unwrap_or(Value::Null),
        "defaults.output" => config
            .defaults
            .output
            .map(|value| json!(value.as_str()))
            .unwrap_or(Value::Null),
        _ => return Err(unknown_key(key)),
    })
}

fn ensure_supported_key(key: &str) -> AppResult<()> {
    if SUPPORTED_KEYS.contains(&key) {
        Ok(())
    } else {
        Err(unknown_key(key))
    }
}

fn unknown_key(key: &str) -> AppError {
    AppError::invalid_request("key", format!("unsupported docnav config key {key:?}"))
}

fn validate_positive_key(key: &str, value: u32) -> AppResult<()> {
    if value == 0 {
        Err(AppError::invalid_request(
            key,
            format!("{key} must be a positive integer"),
        ))
    } else {
        Ok(())
    }
}

fn path_string(path: &Path) -> String {
    path_to_slash(path)
}
