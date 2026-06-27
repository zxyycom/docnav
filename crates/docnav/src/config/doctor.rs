use serde_json::{json, Value};

use crate::adapter_output_contract::manifest_from_output;
use crate::adapter_process::run_manifest;
use crate::error::{AppResult, DocnavExitCode};
use crate::output::CommandOutcome;
use crate::project_context::ProjectContext;
use crate::registry::{self, AdapterRegistry};

use super::store::{path_string, read_config};

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

fn check_config_file(name: &str, path: &std::path::Path) -> Value {
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
