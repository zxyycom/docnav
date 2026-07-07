// @case WB-CORE-CONFIG-PATH-002
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use docnav_protocol::Operation;
use serde_json::{json, Value};

use super::*;
use crate::cli::{ConfigGet, ConfigList, ConfigSet, ConfigUnset};
use crate::config::ResolvedValue;
use crate::output::{write_outcome, CommandOutcome};
use crate::project_context::{ConfigPathOrigin, ProjectContext};
use crate::runtime::{
    AdapterContextOutput, DocumentContextOutput, DocumentRequest, ResolvedDocumentDefaults,
};

#[test]
fn config_set_writes_selected_project_and_user_config_files() {
    let workspace = temp_workspace("config-set-selected-paths");
    let project_config = workspace.join("selected-project.json");
    let user_config = workspace.join("nested").join("selected-user.json");
    let default_project_config = workspace.join(".docnav").join("docnav.json");

    let project_output = execute(
        ConfigCommand::Set(ConfigSet {
            key: "defaults.output".to_owned(),
            value: "readable-json".to_owned(),
            user: false,
            config_paths: config_paths(&project_config, &user_config),
        }),
        &UnusedRuntime,
    )
    .expect("project config set");
    let project_json = outcome_json(project_output);
    assert_eq!(project_json["scope"], "project");
    assert_eq!(project_json["path"], path_string(&project_config));

    let user_output = execute(
        ConfigCommand::Set(ConfigSet {
            key: "defaults.pagination.limit".to_owned(),
            value: "321".to_owned(),
            user: true,
            config_paths: config_paths(&project_config, &user_config),
        }),
        &UnusedRuntime,
    )
    .expect("user config set");
    let user_json = outcome_json(user_output);
    assert_eq!(user_json["scope"], "user");
    assert_eq!(user_json["path"], path_string(&user_config));

    let project_value: Value =
        serde_json::from_str(&fs::read_to_string(&project_config).unwrap()).unwrap();
    let user_value: Value =
        serde_json::from_str(&fs::read_to_string(&user_config).unwrap()).unwrap();
    assert_eq!(project_value["defaults"]["output"], "readable-json");
    assert_eq!(user_value["defaults"]["pagination"]["limit"], 321);
    assert!(
        !default_project_config.exists(),
        "explicit project config path must not write the project-context default"
    );

    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn config_set_rejects_empty_invocation_log_paths() {
    let workspace = temp_workspace("config-set-empty-invocation-log-paths");
    let project_config = workspace.join("selected-project.json");
    let user_config = workspace.join("selected-user.json");

    for (key, reason) in [
        (
            "invocation_log.path",
            "invocation log path must not be empty",
        ),
        (
            "invocation_log.content_capture.root",
            "invocation log content capture root must not be empty",
        ),
    ] {
        let error = expect_command_error(
            execute(
                ConfigCommand::Set(ConfigSet {
                    key: key.to_owned(),
                    value: String::new(),
                    user: false,
                    config_paths: config_paths(&project_config, &user_config),
                }),
                &UnusedRuntime,
            ),
            "empty invocation log config value should fail",
        );
        let details = error.diagnostic().details().to_value();

        assert_eq!(details["field"], key);
        assert_eq!(details["reason"], reason);
    }

    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn config_list_path_context_uses_selected_config_files() {
    let workspace = temp_workspace("config-list-selected-paths");
    let project_config = workspace.join("selected-project.json");
    let user_config = workspace.join("selected-user.json");
    write_json(
        &project_config,
        json!({"defaults": {"output": "readable-json"}}),
    );
    write_json(
        &user_config,
        json!({"defaults": {"pagination": {"limit": 321}}}),
    );
    let runtime = CapturingRuntime::default();

    let output = execute(
        ConfigCommand::List(ConfigList {
            user: false,
            path: Some("docs/guide.md".to_owned()),
            operation: Some(Operation::Read),
            config_paths: config_paths(&project_config, &user_config),
        }),
        &runtime,
    )
    .expect("config list");
    let output = outcome_json(output);

    assert_eq!(output["project_config"], path_string(&project_config));
    assert_eq!(output["user_config"], path_string(&user_config));
    let seen = runtime
        .seen
        .borrow()
        .clone()
        .expect("runtime should receive document context");
    assert_eq!(seen.path, "docs/guide.md");
    assert_eq!(seen.operation, Some(Operation::Read));
    assert_eq!(seen.project_config_path, project_config);
    assert_eq!(seen.user_config_path, user_config);
    assert_eq!(seen.project_origin, ConfigPathOrigin::ExplicitCli);
    assert_eq!(seen.user_origin, ConfigPathOrigin::ExplicitCli);

    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn config_get_rejects_explicit_missing_project_config() {
    let workspace = temp_workspace("config-get-explicit-missing");
    let project_config = workspace.join("missing-project.json");
    let user_config = workspace.join("user.json");
    write_json(&user_config, json!({}));

    let error = expect_command_error(
        execute(
            ConfigCommand::Get(ConfigGet {
                key: "defaults.output".to_owned(),
                user: false,
                config_paths: config_paths(&project_config, &user_config),
            }),
            &UnusedRuntime,
        ),
        "explicit missing project config should fail",
    );
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["reason"], "missing_explicit_cli");
    assert_eq!(details["config_issues"][0]["source_level"], "project");
    assert_eq!(details["config_issues"][0]["path_origin"], "explicit_cli");

    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn config_get_user_scope_reads_only_selected_user_config_values() {
    let workspace = temp_workspace("config-get-user-only");
    let project_config = workspace.join("missing-project.json");
    let user_config = workspace.join("user.json");
    write_json(
        &user_config,
        json!({"defaults": {"output": "readable-json"}}),
    );

    let output = execute(
        ConfigCommand::Get(ConfigGet {
            key: "defaults.output".to_owned(),
            user: true,
            config_paths: config_paths(&project_config, &user_config),
        }),
        &UnusedRuntime,
    )
    .expect("user scoped get should not read project config values");
    let output = outcome_json(output);

    assert_eq!(output["key"], "defaults.output");
    assert_eq!(output["value"], "readable-json");
    assert_eq!(output["source"], "user");

    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn config_list_user_scope_without_path_reads_only_selected_user_config_values() {
    let workspace = temp_workspace("config-list-user-only");
    let project_config = workspace.join("broken-project.json");
    let user_config = workspace.join("user.json");
    write_text(&project_config, "{not-json");
    write_json(
        &user_config,
        json!({"defaults": {"pagination": {"limit": 321}}}),
    );

    let output = execute(
        ConfigCommand::List(ConfigList {
            user: true,
            path: None,
            operation: None,
            config_paths: config_paths(&project_config, &user_config),
        }),
        &UnusedRuntime,
    )
    .expect("user scoped list values should not read project config values");
    let output = outcome_json(output);
    let limit = value_for_key(&output, "defaults.pagination.limit");

    assert_eq!(limit["value"], 321);
    assert_eq!(limit["source"], "user");

    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn config_list_user_scope_with_path_delegates_selected_descriptors() {
    let workspace = temp_workspace("config-list-user-path-context");
    let project_config = workspace.join("missing-project.json");
    let user_config = workspace.join("user.json");
    write_json(&user_config, json!({}));
    let runtime = CapturingRuntime::default();

    let output = execute(
        ConfigCommand::List(ConfigList {
            user: true,
            path: Some("docs/guide.md".to_owned()),
            operation: Some(Operation::Outline),
            config_paths: config_paths(&project_config, &user_config),
        }),
        &runtime,
    )
    .expect("path context should receive selected descriptors");
    let output = outcome_json(output);

    assert_eq!(output["project_config"], path_string(&project_config));
    assert_eq!(output["user_config"], path_string(&user_config));
    let seen = runtime
        .seen
        .borrow()
        .clone()
        .expect("runtime should receive document context");
    assert_eq!(seen.path, "docs/guide.md");
    assert_eq!(seen.operation, Some(Operation::Outline));
    assert_eq!(seen.project_config_path, project_config);
    assert_eq!(seen.user_config_path, user_config);
    assert_eq!(seen.project_origin, ConfigPathOrigin::ExplicitCli);
    assert_eq!(seen.user_origin, ConfigPathOrigin::ExplicitCli);

    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn config_list_rejects_explicit_missing_user_config() {
    let workspace = temp_workspace("config-list-explicit-missing");
    let project_config = workspace.join("project.json");
    let user_config = workspace.join("missing-user.json");
    write_json(&project_config, json!({}));

    let error = expect_command_error(
        execute(
            ConfigCommand::List(ConfigList {
                user: false,
                path: None,
                operation: None,
                config_paths: config_paths(&project_config, &user_config),
            }),
            &UnusedRuntime,
        ),
        "explicit missing user config should fail",
    );
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["reason"], "missing_explicit_cli");
    assert_eq!(details["config_issues"][0]["source_level"], "user");
    assert_eq!(details["config_issues"][0]["path_origin"], "explicit_cli");

    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn config_mutations_preserve_navigation_owned_outline_config() {
    let workspace = temp_workspace("config-mutation-preserves-outline");
    let project_config = workspace.join("project.json");
    let user_config = workspace.join("user.json");
    let outline = json!({
        "mode_rules": [
            {
                "path": "docs/raw\\.md",
                "mode": "unstructured_full"
            }
        ]
    });
    write_json(
        &project_config,
        json!({
            "defaults": {
                "output": "readable-view"
            },
            "outline": outline.clone()
        }),
    );
    write_json(&user_config, json!({}));

    execute(
        ConfigCommand::Set(ConfigSet {
            key: "defaults.pagination.limit".to_owned(),
            value: "321".to_owned(),
            user: false,
            config_paths: config_paths(&project_config, &user_config),
        }),
        &UnusedRuntime,
    )
    .expect("set preserves outline config");
    let after_set: Value =
        serde_json::from_str(&fs::read_to_string(&project_config).unwrap()).unwrap();
    assert_eq!(after_set["outline"], outline);
    assert_eq!(after_set["defaults"]["pagination"]["limit"], 321);

    execute(
        ConfigCommand::Unset(ConfigUnset {
            key: "defaults.output".to_owned(),
            user: false,
            config_paths: config_paths(&project_config, &user_config),
        }),
        &UnusedRuntime,
    )
    .expect("unset preserves outline config");
    let after_unset: Value =
        serde_json::from_str(&fs::read_to_string(&project_config).unwrap()).unwrap();
    assert_eq!(after_unset["outline"], outline);
    assert!(after_unset["defaults"].get("output").is_none());

    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn init_rejects_selected_project_config_directory() {
    let workspace = temp_workspace("init-project-config-directory");
    let project_config = workspace.join("selected-project-dir");
    fs::create_dir_all(&project_config).unwrap();

    let error = expect_command_error(
        init_project(ConfigPathArgs {
            project_config: Some(project_config.display().to_string()),
            user_config: None,
        }),
        "directory is not an exact config JSON file path",
    );
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

fn expect_command_error(result: AppResult<CommandOutcome>, message: &str) -> AppError {
    match result {
        Ok(_) => panic!("{message}"),
        Err(error) => error,
    }
}

fn write_json(path: &Path, value: Value) {
    write_text(path, &serde_json::to_string_pretty(&value).unwrap());
}

fn write_text(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn value_for_key<'a>(output: &'a Value, key: &str) -> &'a Value {
    output["values"]
        .as_array()
        .and_then(|values| {
            values
                .iter()
                .find(|value| value["key"].as_str() == Some(key))
        })
        .unwrap_or_else(|| panic!("missing value for key {key}: {output}"))
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

#[derive(Default)]
struct CapturingRuntime {
    seen: RefCell<Option<SeenContext>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SeenContext {
    path: String,
    operation: Option<Operation>,
    project_config_path: PathBuf,
    user_config_path: PathBuf,
    project_origin: ConfigPathOrigin,
    user_origin: ConfigPathOrigin,
}

impl DocnavRuntime for CapturingRuntime {
    fn execute_document(&self, _request: DocumentRequest) -> AppResult<CommandOutcome> {
        unreachable!("config list test does not execute document operations")
    }

    fn describe_document_context(
        &self,
        path: String,
        operation: Option<Operation>,
        project: &ProjectContext,
    ) -> AppResult<DocumentContextOutput> {
        self.seen.replace(Some(SeenContext {
            path: path.clone(),
            operation,
            project_config_path: project.config_paths.project.path.clone(),
            user_config_path: project.config_paths.user.path.clone(),
            project_origin: project.config_paths.project.origin,
            user_origin: project.config_paths.user.origin,
        }));
        Ok(DocumentContextOutput {
            path,
            operation,
            adapter: AdapterContextOutput {
                selected: None,
                source: "test".to_owned(),
                note: "captured selected config paths".to_owned(),
            },
            defaults: ResolvedDocumentDefaults {
                adapter: ResolvedValue::built_in(json!(null)),
                pagination: None,
                output: ResolvedValue::built_in(json!("readable-view")),
                page: None,
            },
            runtime_status: "test_runtime_ready".to_owned(),
        })
    }
}

struct UnusedRuntime;

impl DocnavRuntime for UnusedRuntime {
    fn execute_document(&self, _request: DocumentRequest) -> AppResult<CommandOutcome> {
        unreachable!("config command test does not execute document operations")
    }

    fn describe_document_context(
        &self,
        _path: String,
        _operation: Option<Operation>,
        _project: &ProjectContext,
    ) -> AppResult<DocumentContextOutput> {
        unreachable!("config set test does not describe document context")
    }
}
