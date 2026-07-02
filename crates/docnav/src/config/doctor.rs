use serde_json::{json, Value};

use crate::error::{AppResult, DocnavExitCode};
use crate::output::CommandOutcome;
use crate::project_context::ProjectContext;
use crate::registry::{self, AdapterRegistry};

use super::store::{path_string, read_config, ConfigFileSource};

pub fn doctor() -> AppResult<CommandOutcome> {
    let project = ProjectContext::discover()?;
    let registry = AdapterRegistry::load(&project)?;
    let mut checks = Vec::new();
    checks.push(check_config_file(
        "project_config",
        &project.project_config_path,
        &registry,
        ConfigFileSource::Project,
    ));
    checks.push(check_config_file(
        "user_config",
        &project.user_config_path,
        &registry,
        ConfigFileSource::User,
    ));
    checks.push(registry::registry_check(&registry));
    checks.extend(registry::adapter_layer_checks(&registry));

    let exit_code = most_severe_exit(&checks);

    Ok(CommandOutcome::json_with_exit(
        json!({
            "project_root": path_string(&project.project_root),
            "checks": checks,
        }),
        exit_code,
    ))
}

fn check_config_file(
    name: &str,
    path: &std::path::Path,
    registry: &AdapterRegistry,
    source: ConfigFileSource,
) -> Value {
    match read_config(path, registry, source) {
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
        Err(error) => {
            let diagnostic = error.diagnostic();
            json!({
                "name": name,
                "status": "fail",
                "path": path_string(path),
                "message": diagnostic.summary(),
                "details": diagnostic.details().to_value(),
            })
        }
    }
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
