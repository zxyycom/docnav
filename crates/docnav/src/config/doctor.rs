use serde_json::{json, Value};

use crate::cli::ConfigPathArgs;
use crate::error::{AppResult, DocnavExitCode};
use crate::output::CommandOutcome;
use crate::project_context::{ProjectContext, SelectedConfigPath};
use crate::registry::{self, AdapterRegistry};

use super::store::{path_string, read_selected_config, ConfigFileSource};

pub fn doctor(config_paths: ConfigPathArgs) -> AppResult<CommandOutcome> {
    let project = ProjectContext::discover_with_cli_config_paths(
        config_paths.project_config.as_deref(),
        config_paths.user_config.as_deref(),
    )?;
    let registry = AdapterRegistry::load(&project)?;
    let mut checks = Vec::new();
    checks.push(check_config_file(
        "project_config",
        ConfigFileSource::Project.selected_path(&project),
        &registry,
        ConfigFileSource::Project,
    ));
    checks.push(check_config_file(
        "user_config",
        ConfigFileSource::User.selected_path(&project),
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
    selection: &SelectedConfigPath,
    registry: &AdapterRegistry,
    source: ConfigFileSource,
) -> Value {
    match read_selected_config(selection, registry, source) {
        Ok(_) if selection.path.exists() => json!({
            "name": name,
            "status": "pass",
            "path": path_string(&selection.path),
            "message": "config file is readable"
        }),
        Ok(_) => json!({
            "name": name,
            "status": "pass",
            "path": path_string(&selection.path),
            "message": "config file is absent; built-in defaults apply"
        }),
        Err(error) => {
            let diagnostic = error.diagnostic();
            json!({
                "name": name,
                "status": "fail",
                "path": path_string(&selection.path),
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde_json::{json, Value};

    use super::*;
    use crate::output::write_outcome;

    #[test]
    fn doctor_reports_explicit_missing_config_as_failure() {
        let workspace = temp_workspace("doctor-explicit-missing");
        let project_config = workspace.join("missing-project.json");
        let user_config = workspace.join("user.json");
        write_json(&user_config, json!({}));

        let (exit_code, output) = run_doctor(&project_config, &user_config);

        assert_eq!(exit_code, DocnavExitCode::InputError.code());
        let project_check = check_by_name(&output, "project_config");
        assert_eq!(project_check["status"], "fail");
        assert_eq!(
            project_check["details"]["config_issues"][0]["path_origin"],
            "explicit_cli"
        );
        assert_eq!(
            project_check["details"]["config_issues"][0]["reason_code"],
            "missing_explicit_cli"
        );

        let _ = fs::remove_dir_all(workspace);
    }

    fn check_by_name<'a>(output: &'a Value, name: &str) -> &'a Value {
        output["checks"]
            .as_array()
            .and_then(|checks| {
                checks
                    .iter()
                    .find(|check| check["name"].as_str() == Some(name))
            })
            .unwrap_or_else(|| panic!("missing {name} check: {output}"))
    }

    fn run_doctor(project_config: &Path, user_config: &Path) -> (i32, Value) {
        let outcome = doctor(ConfigPathArgs {
            project_config: Some(project_config.display().to_string()),
            user_config: Some(user_config.display().to_string()),
        })
        .expect("doctor returns check output");
        outcome_json(outcome)
    }

    fn outcome_json(outcome: CommandOutcome) -> (i32, Value) {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit_code = write_outcome(outcome, &mut stdout, &mut stderr);
        assert!(
            stderr.is_empty(),
            "stderr: {}",
            String::from_utf8_lossy(&stderr)
        );
        (exit_code, serde_json::from_slice(&stdout).unwrap())
    }

    fn write_json(path: &Path, value: Value) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
    }

    fn temp_workspace(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir()
            .join("docnav-doctor-tests")
            .join(format!("{name}-{suffix}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
