use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use super::*;
use crate::cli::{ConfigInspect, ConfigPathArgs};
use crate::output::{write_outcome, CommandOutcome};

const VALID_INSPECTION_GOLDEN: &str = include_str!("fixtures/config-inspect-valid.json");
const INVALID_JSON_INSPECTION_GOLDEN: &str =
    include_str!("fixtures/config-inspect-invalid-json.json");

#[test]
// @case WB-CORE-CONFIG-PATH-002
fn config_inspect_reports_selected_sources_and_parameter_facts_without_writing() {
    let workspace = temp_workspace("config-inspect-selected-sources");
    let project_config = workspace.join("selected-project.json");
    let user_config = workspace.join("selected-user.json");
    write_json(
        &project_config,
        json!({
            "defaults": {
                "output": "readable-json"
            },
            "options": {
                "docnav-markdown": {
                    "max_heading_level": 2
                }
            }
        }),
    );
    write_json(
        &user_config,
        json!({
            "defaults": {
                "pagination": {
                    "limit": 321
                }
            }
        }),
    );
    let project_before = fs::read_to_string(&project_config).unwrap();
    let user_before = fs::read_to_string(&user_config).unwrap();

    let output = execute(ConfigCommand::Inspect(ConfigInspect {
        config_paths: config_paths(&project_config, &user_config),
    }))
    .expect("config inspect");
    let mut output = outcome_json(output);
    normalize_dynamic_paths(&mut output, &project_config, &user_config);
    assert_json_golden(&output, VALID_INSPECTION_GOLDEN);
    assert!(
        parameter_fact(
            &output["inspection"],
            crate::parameter_catalog::PAGE_IDENTITY
        )
        .is_none(),
        "CLI-only page must not appear in config parameter facts"
    );
    assert_eq!(fs::read_to_string(&project_config).unwrap(), project_before);
    assert_eq!(fs::read_to_string(&user_config).unwrap(), user_before);

    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn config_inspect_omits_optional_non_json_null_parameter_fact() {
    let workspace = temp_workspace("config-inspect-null-parameter-fact");
    let project_config = workspace.join("project.json");
    let user_config = workspace.join("user.json");
    write_json(
        &project_config,
        json!({
            "options": {
                "docnav-markdown": {
                    "max_heading_level": null
                }
            }
        }),
    );
    write_json(&user_config, json!({}));

    let output = execute(ConfigCommand::Inspect(ConfigInspect {
        config_paths: config_paths(&project_config, &user_config),
    }))
    .expect("config inspect");
    let output = outcome_json(output);
    let inspection = &output["inspection"];

    assert!(
        parameter_fact(
            inspection,
            "docnav.adapters.docnav-markdown.options.max_heading_level"
        )
        .is_none(),
        "optional non-JSON null should suppress its default without becoming a fact: {inspection}"
    );

    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn config_inspect_reports_validation_diagnostics_without_failing() {
    let workspace = temp_workspace("config-inspect-diagnostics");
    let project_config = workspace.join("project.json");
    let user_config = workspace.join("user.json");
    write_json(
        &project_config,
        json!({
            "defaults": {
                "pagination": {
                    "limit": 0
                }
            }
        }),
    );
    write_json(&user_config, json!({}));

    let output = execute(ConfigCommand::Inspect(ConfigInspect {
        config_paths: config_paths(&project_config, &user_config),
    }))
    .expect("inspect should report, not fail");
    let output = outcome_json(output);
    let diagnostics = output["inspection"]["sources"][0]["diagnostics"]
        .as_array()
        .expect("diagnostics array");

    assert_eq!(diagnostics[0]["field"], "defaults.pagination.limit");
    assert_eq!(diagnostics[0]["reason"], "range_invalid");

    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn config_inspect_reports_catalog_adapter_range_with_exact_source() {
    let workspace = temp_workspace("config-inspect-catalog-adapter-range");
    let project_config = workspace.join("project.json");
    let user_config = workspace.join("user.json");
    write_json(
        &project_config,
        json!({
            "options": {
                "docnav-markdown": {
                    "max_heading_level": 7
                }
            }
        }),
    );
    write_json(&user_config, json!({}));

    let output = execute(ConfigCommand::Inspect(ConfigInspect {
        config_paths: config_paths(&project_config, &user_config),
    }))
    .expect("inspect should report, not fail");
    let output = outcome_json(output);
    let diagnostic = &output["inspection"]["sources"][0]["diagnostics"][0];

    assert_eq!(
        diagnostic["field"],
        "options.docnav-markdown.max_heading_level"
    );
    assert_eq!(diagnostic["reason"], "range_invalid");
    assert_eq!(diagnostic["path"], path_string(&project_config));

    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn config_inspect_reports_explicit_source_load_status_without_failing() {
    let workspace = temp_workspace("config-inspect-source-status");
    let user_config = workspace.join("user.json");
    write_json(&user_config, json!({}));

    let missing_config = workspace.join("missing-project.json");
    assert_inspect_project_source_status(
        &missing_config,
        &user_config,
        false,
        "missing_explicit_cli",
    );

    let non_object = workspace.join("non-object-project.json");
    write_raw(&non_object, "[]");
    assert_inspect_project_source_status(&non_object, &user_config, true, "non_object");

    let not_file = workspace.join("not-file-project.json");
    fs::create_dir_all(&not_file).unwrap();
    assert_inspect_project_source_status(&not_file, &user_config, true, "not_file");

    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn config_inspect_serializes_complete_invalid_json_load_status() {
    let workspace = temp_workspace("config-inspect-invalid-json-golden");
    let project_config = workspace.join("invalid-project.json");
    let user_config = workspace.join("user.json");
    write_raw(&project_config, "{invalid");
    write_json(&user_config, json!({}));

    let output = execute(ConfigCommand::Inspect(ConfigInspect {
        config_paths: config_paths(&project_config, &user_config),
    }))
    .expect("inspect should serialize invalid source status");
    let mut output = outcome_json(output);
    normalize_dynamic_paths(&mut output, &project_config, &user_config);
    assert_json_golden(&output, INVALID_JSON_INSPECTION_GOLDEN);

    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn init_creates_and_preserves_selected_project_config_file() {
    let workspace = temp_workspace("init-selected-project-config");
    let project_config = workspace.join("nested").join("selected-project.json");
    let config_paths = ConfigPathArgs {
        project_config: Some(project_config.display().to_string()),
        user_config: None,
    };

    let created =
        outcome_json(init_project(config_paths.clone()).expect("init creates selected config"));
    assert_eq!(created["created"], true);
    assert_eq!(created["config_path"], path_string(&project_config));
    assert!(project_config.is_file());

    fs::write(
        &project_config,
        "{\"defaults\":{\"output\":\"readable-json\"}}\n",
    )
    .unwrap();
    let preserved =
        outcome_json(init_project(config_paths).expect("init preserves selected config"));
    assert_eq!(preserved["created"], false);
    assert_eq!(
        fs::read_to_string(&project_config).unwrap(),
        "{\"defaults\":{\"output\":\"readable-json\"}}\n"
    );

    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn init_rejects_selected_project_config_directory() {
    let workspace = temp_workspace("init-project-config-directory");
    let project_config = workspace.join("selected-project-dir");
    fs::create_dir_all(&project_config).unwrap();

    let error = match init_project(ConfigPathArgs {
        project_config: Some(project_config.display().to_string()),
        user_config: None,
    }) {
        Ok(_) => panic!("directory is not an exact config JSON file path"),
        Err(error) => error,
    };
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["field"], "project_config");
    assert!(
        details["reason"]
            .as_str()
            .is_some_and(|reason| reason.contains("not a file")),
        "expected not-a-file reason, got {details}"
    );

    let _ = fs::remove_dir_all(workspace);
}

fn parameter_fact<'a>(inspection: &'a Value, identity: &str) -> Option<&'a Value> {
    inspection["parameter_facts"]
        .as_array()?
        .iter()
        .find(|fact| fact["identity"].as_str() == Some(identity))
}

fn assert_inspect_project_source_status(
    project_config: &Path,
    user_config: &Path,
    exists: bool,
    reason_code: &str,
) {
    let output = execute(ConfigCommand::Inspect(ConfigInspect {
        config_paths: config_paths(project_config, user_config),
    }))
    .expect("inspect should report source status");
    let output = outcome_json(output);
    let source = &output["inspection"]["sources"][0];
    let diagnostics = source["diagnostics"].as_array().expect("diagnostics array");

    assert_eq!(source["scope"], "project");
    assert_eq!(source["path"], project_config.display().to_string());
    assert_eq!(source["origin"], "explicit_cli");
    assert_eq!(source["exists"], exists);
    assert_eq!(source["load_state"], reason_code);
    assert_eq!(
        source["summary"],
        json!({
            "top_level_fields": [],
            "field_count": 0
        })
    );
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["source_level"], "project");
    assert_eq!(diagnostics[0]["path_origin"], "explicit_cli");
    assert_eq!(diagnostics[0]["path"], project_config.display().to_string());
    assert_eq!(diagnostics[0]["field"], Value::Null);
    assert_eq!(diagnostics[0]["reason_code"], reason_code);
}

fn normalize_dynamic_paths(output: &mut Value, project_config: &Path, user_config: &Path) {
    let current_dir = std::env::current_dir().expect("config inspect test current directory");
    let expected_project_root =
        path_string(&crate::project_context::find_project_root(&current_dir));
    assert_eq!(
        output["project_root"], expected_project_root,
        "config inspect must report the ProjectContext owner root"
    );
    output["project_root"] = json!("<project-root>");
    replace_exact_string(
        output,
        &project_config.display().to_string(),
        "<project-config>",
    );
    replace_exact_string(output, &user_config.display().to_string(), "<user-config>");
}

fn replace_exact_string(value: &mut Value, before: &str, after: &str) {
    match value {
        Value::String(current) if current == before => *current = after.to_owned(),
        Value::Array(values) => {
            for value in values {
                replace_exact_string(value, before, after);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                replace_exact_string(value, before, after);
            }
        }
        _ => {}
    }
}

fn assert_json_golden(actual: &Value, golden: &str) {
    let expected: Value = serde_json::from_str(golden).expect("valid config inspect golden");
    assert_eq!(actual, &expected);
}

fn config_paths(project_config: &Path, user_config: &Path) -> ConfigPathArgs {
    ConfigPathArgs {
        project_config: Some(project_config.display().to_string()),
        user_config: Some(user_config.display().to_string()),
    }
}

fn outcome_json(outcome: CommandOutcome) -> Value {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit_code = write_outcome(outcome, &mut stdout, &mut stderr);
    assert_eq!(exit_code, 0, "stderr: {}", String::from_utf8_lossy(&stderr));
    assert!(
        stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&stderr)
    );
    serde_json::from_slice(&stdout).unwrap()
}

fn write_json(path: &Path, value: Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

fn write_raw(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn temp_workspace(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir()
        .join("docnav-config-commands-tests")
        .join(format!("{name}-{suffix}"));
    fs::create_dir_all(&path).unwrap();
    path
}
