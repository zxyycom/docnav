// @case WB-CORE-CONFIG-PATH-001
use super::super::parse;
use crate::cli::{CliCommand, ConfigCommand};

#[test]
fn document_command_parses_config_file_paths_as_exact_values() {
    let parsed = parse([
        "outline",
        "doc.md",
        "--project-config",
        "./fixtures/project.json",
        "--user-config=./fixtures/user.json",
    ])
    .expect("parse document config paths");

    match parsed.command {
        CliCommand::Document(command) => {
            assert_eq!(
                command.config_paths.project_config.as_deref(),
                Some("./fixtures/project.json")
            );
            assert_eq!(
                command.config_paths.user_config.as_deref(),
                Some("./fixtures/user.json")
            );
            assert!(
                command.native_options.is_empty(),
                "config path flags must not become adapter native options"
            );
        }
        command => panic!("expected document command, got {command:?}"),
    }
}

#[test]
fn config_inspect_parses_selected_config_file_paths() {
    let parsed = parse([
        "config",
        "inspect",
        "--project-config",
        "project.json",
        "--user-config",
        "user.json",
    ])
    .expect("parse config inspect config paths");

    match parsed.command {
        CliCommand::Config(ConfigCommand::Inspect(command)) => {
            assert_eq!(
                command.config_paths.project_config.as_deref(),
                Some("project.json")
            );
            assert_eq!(
                command.config_paths.user_config.as_deref(),
                Some("user.json")
            );
        }
        command => panic!("expected config inspect command, got {command:?}"),
    }
}

#[test]
fn legacy_config_subcommands_are_rejected() {
    for subcommand in ["get", "set", "unset", "list"] {
        let error = parse(["config", subcommand]).expect_err("legacy config command should fail");
        let details = error.diagnostic().details().to_value();

        assert_eq!(details["field"], "config");
        assert!(
            details["reason"]
                .as_str()
                .is_some_and(|reason| reason.contains("unknown config subcommand")),
            "unexpected error details for {subcommand}: {details}"
        );
    }
}

#[test]
fn config_inspect_rejects_document_context_flags() {
    let parsed = parse(["config", "inspect", "--path", "docs/guide.md"])
        .expect_err("config inspect should not accept document context flags");
    let details = parsed.diagnostic().details().to_value();

    assert_eq!(details["field"], "--path");
    assert_eq!(details["reason"], "unsupported_argument");
}

#[test]
fn init_and_doctor_parse_config_file_paths() {
    let init =
        parse(["init", "--project-config", "custom-project.json"]).expect("parse init config path");
    match init.command {
        CliCommand::Init(config_paths) => {
            assert_eq!(
                config_paths.project_config.as_deref(),
                Some("custom-project.json")
            );
        }
        command => panic!("expected init command, got {command:?}"),
    }

    let doctor = parse([
        "doctor",
        "--project-config",
        "project.json",
        "--user-config",
        "user.json",
    ])
    .expect("parse doctor config paths");
    match doctor.command {
        CliCommand::Doctor(config_paths) => {
            assert_eq!(config_paths.project_config.as_deref(), Some("project.json"));
            assert_eq!(config_paths.user_config.as_deref(), Some("user.json"));
        }
        command => panic!("expected doctor command, got {command:?}"),
    }
}

#[test]
fn config_path_flag_before_known_flag_is_missing_value_input_error() {
    let error = parse(["config", "inspect", "--project-config", "--user-config"])
        .expect_err("project config path should not consume user config flag");
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["field"], "--project-config");
    assert_eq!(details["reason"], "missing_value");

    let error = parse(["config", "inspect", "--user-config", "--output"])
        .expect_err("user config path should not consume known output flag");
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["field"], "--user-config");
    assert_eq!(details["reason"], "missing_value");
}

#[test]
fn inline_config_path_value_can_start_with_known_flag_text() {
    let parsed = parse(["config", "inspect", "--project-config=--user-config"])
        .expect("inline exact path should allow known flag text");

    match parsed.command {
        CliCommand::Config(ConfigCommand::Inspect(command)) => {
            assert_eq!(
                command.config_paths.project_config.as_deref(),
                Some("--user-config")
            );
        }
        command => panic!("expected config inspect command, got {command:?}"),
    }
}

#[test]
fn unsupported_config_path_flag_is_input_error() {
    let error = parse(["version", "--project-config", "project.json"])
        .expect_err("version should not accept config path flags");
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["field"], "--project-config");
    assert_eq!(details["reason"], "unsupported_argument");
}

#[test]
fn init_rejects_user_config_path_flag() {
    let error = parse(["init", "--user-config", "user.json"])
        .expect_err("init should not accept user config path");
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["field"], "--user-config");
    assert_eq!(details["reason"], "unsupported_argument");
}
