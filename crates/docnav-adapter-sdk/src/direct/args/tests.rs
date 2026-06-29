use super::super::warnings::{DirectCliWarning, CLI_ARGV_IGNORED};
use super::super::{NativeOptionDefault, NativeOptionValueSpec};
use super::*;
use docnav_diagnostics::DiagnosticEffect;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_HEADING_LEVEL_OPERATIONS: &[Operation] = &[Operation::Outline, Operation::Find];
const MAX_HEADING_LEVEL: NativeOptionSpec = NativeOptionSpec {
    flag: "--max-heading-level",
    option_key: "max_heading_level",
    operations: MAX_HEADING_LEVEL_OPERATIONS,
    value: NativeOptionValueSpec::IntegerRange { min: 1, max: 6 },
    default: Some(NativeOptionDefault::Integer(3)),
};
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

// @case WB-SDK-DIRECT-ARGS-001
#[test]
fn unknown_flag_does_not_consume_following_positional() {
    let options = parse_for_test(Operation::Outline, &["--future", "doc.md"], 6000, &[])
        .expect("parse options");

    assert_eq!(options.path, "doc.md");
    assert_eq!(
        options.warnings,
        vec![DirectCliWarning::unknown_flag("--future")]
    );
}

#[test]
fn unknown_flag_with_equals_is_one_ignored_token() {
    let options = parse_for_test(Operation::Outline, &["--future=value", "doc.md"], 6000, &[])
        .expect("parse options");

    assert_eq!(options.warnings.len(), 1);
    assert_eq!(options.warnings[0].code(), CLI_ARGV_IGNORED);
    assert_eq!(
        options.warnings[0].effect(),
        DiagnosticEffect::OperationContinued
    );
    assert_eq!(warning_tokens(&options.warnings[0]), ["--future=value"]);
}

#[test]
fn extra_positional_warns_after_path_slot_is_filled() {
    let options =
        parse_for_test(Operation::Outline, &["doc.md", "extra"], 6000, &[]).expect("parse options");

    assert_eq!(options.path, "doc.md");
    assert_eq!(options.warnings.len(), 1);
    assert_eq!(options.warnings[0].code(), CLI_ARGV_IGNORED);
    assert_eq!(warning_tokens(&options.warnings[0]), ["extra"]);
}

#[test]
fn operation_only_validates_flags_it_uses() {
    let read = parse_for_test(
        Operation::Read,
        &["doc.md", "--ref", "L1:Guide", "--max-heading-level", "nope"],
        6000,
        &[MAX_HEADING_LEVEL],
    )
    .expect("unused native value should be ignored");
    assert_eq!(
        warning_tokens(&read.warnings[0]),
        ["--max-heading-level", "nope"]
    );

    let info = parse_for_test(Operation::Info, &["doc.md", "--limit", "nope"], 6000, &[])
        .expect("unused core value should be ignored");
    assert_eq!(info.limit.get(), 6000);
    assert_eq!(warning_tokens(&info.warnings[0]), ["--limit", "nope"]);
}

#[test]
fn unused_value_flag_consumes_value_that_looks_like_flag() {
    let options = parse_for_test(
        Operation::Read,
        &["doc.md", "--ref", "L1:Guide", "--query", "--future-value"],
        6000,
        &[],
    )
    .expect("parse options");

    assert_eq!(options.ref_id.as_deref(), Some("L1:Guide"));
    assert_eq!(
        warning_tokens(&options.warnings[0]),
        ["--query", "--future-value"]
    );
}

#[test]
fn used_value_flag_accepts_value_that_looks_like_flag() {
    let options = parse_for_test(
        Operation::Read,
        &["doc.md", "--ref", "--future-value"],
        6000,
        &[],
    )
    .expect("parse options");

    assert_eq!(options.ref_id.as_deref(), Some("--future-value"));
    assert!(options.warnings.is_empty());
}

#[test]
fn protocol_only_command_warns_without_losing_required_output() {
    let warnings = parse_protocol_only_options(
        &args(&["--future", "extra", "--output", "protocol-json"]),
        &[],
    )
    .expect("parse protocol-only options");

    assert_eq!(warnings.len(), 2);
    assert_eq!(warning_tokens(&warnings[0]), ["--future"]);
    assert_eq!(warning_tokens(&warnings[1]), ["extra"]);
}

#[test]
fn probe_path_can_follow_unknown_flag() {
    let parsed = parse_probe(
        &args(&["--future", "doc.md", "--output", "protocol-json"]),
        &[],
    )
    .expect("parse probe options");

    assert_eq!(parsed.path, "doc.md");
    assert_eq!(warning_tokens(&parsed.warnings[0]), ["--future"]);
}

#[test]
fn probe_help_does_not_advertise_native_options() {
    let mut root = direct_cli_command("docnav-test", &[MAX_HEADING_LEVEL], 6000);
    let probe = root
        .find_subcommand_mut(command_names::PROBE)
        .expect("probe command registered");
    let help = probe.render_long_help().to_string();

    assert!(help.contains("--output"));
    assert!(!help.contains("--max-heading-level"));
}

#[test]
fn config_sources_merge_before_typed_operation_options() {
    let dir = temp_dir("merge-precedence");
    let project_config = dir.join("project.json");
    let user_config = dir.join("user.json");
    write_merge_precedence_configs(&project_config, &user_config);
    let project_config_arg = path_arg(&project_config);
    let user_config_arg = path_arg(&user_config);

    let options = parse_for_test(
        Operation::Outline,
        &[
            "doc.md",
            "--project-config-path",
            &project_config_arg,
            "--user-config-path",
            &user_config_arg,
            "--limit",
            "300",
            "--max-heading-level",
            "5",
        ],
        6000,
        &[MAX_HEADING_LEVEL],
    )
    .expect("parse merged config options");

    assert_eq!(options.limit.get(), 300);
    assert_eq!(options.output, DirectOutputMode::ProtocolJson);
    assert_eq!(
        options.protocol_options().unwrap()["max_heading_level"],
        json!(5)
    );
    assert!(options.warnings.is_empty());
}

#[test]
fn pagination_disabled_finalizes_limit_after_source_merge() {
    let dir = temp_dir("pagination-disabled");
    let project_config = dir.join("project.json");
    write_json(
        &project_config,
        json!({
            "defaults": {
                "pagination": {
                    "enabled": false,
                    "limit": 20
                }
            }
        }),
    );
    let project_config_arg = path_arg(&project_config);

    let options = parse_for_test(
        Operation::Outline,
        &[
            "doc.md",
            "--project-config-path",
            &project_config_arg,
            "--limit",
            "300",
        ],
        6000,
        &[],
    )
    .expect("parse disabled pagination options");

    assert_eq!(options.limit.get(), u32::MAX);
    assert!(options.warnings.is_empty());
}

#[test]
fn legacy_defaults_limit_config_path_is_rejected() {
    let dir = temp_dir("legacy-limit");
    let project_config = dir.join("project.json");
    write_json(
        &project_config,
        json!({
            "defaults": {
                "limit": 200
            }
        }),
    );
    let project_config_arg = path_arg(&project_config);

    let error = parse_for_test(
        Operation::Outline,
        &["doc.md", "--project-config-path", &project_config_arg],
        6000,
        &[],
    )
    .expect_err("legacy defaults.limit must fail");

    assert!(error.contains("unsupported defaults.limit"));
    assert!(error.contains("defaults.pagination.limit"));
}

#[test]
fn invalid_pagination_enabled_config_is_typed_validation_error() {
    let dir = temp_dir("invalid-pagination-enabled");
    let project_config = dir.join("project.json");
    write_json(
        &project_config,
        json!({
            "defaults": {
                "pagination": {
                    "enabled": "disabled"
                }
            }
        }),
    );
    let project_config_arg = path_arg(&project_config);

    let error = parse_for_test(
        Operation::Outline,
        &["doc.md", "--project-config-path", &project_config_arg],
        6000,
        &[],
    )
    .expect_err("string pagination enabled must fail typed validation");

    assert!(error.contains("--pagination"));
    assert!(error.contains("enabled or disabled"));
}

fn write_merge_precedence_configs(project_config: &Path, user_config: &Path) {
    write_json(
        project_config,
        json!({
            "defaults": {"pagination": {"limit": 200}, "output": "protocol-json"},
            "options": {"max_heading_level": 2}
        }),
    );
    write_json(
        user_config,
        json!({
            "defaults": {"pagination": {"limit": 100}, "output": "readable-json"},
            "options": {"max_heading_level": 4}
        }),
    );
}

#[test]
fn config_source_warning_keeps_operation_running_with_other_sources() {
    let dir = temp_dir("source-warning");
    let user_config = dir.join("user.json");
    write_json(
        &user_config,
        json!({
            "defaults": {
                "output": "readable-json"
            }
        }),
    );
    let missing_project_arg = dir.join("missing.json").to_string_lossy().into_owned();
    let user_config_arg = path_arg(&user_config);

    let options = parse_for_test(
        Operation::Outline,
        &[
            "doc.md",
            "--project-config-path",
            &missing_project_arg,
            "--user-config-path",
            &user_config_arg,
        ],
        6000,
        &[MAX_HEADING_LEVEL],
    )
    .expect("missing project override should not fail operation parsing");

    assert_eq!(options.output, DirectOutputMode::ReadableJson);
    assert_eq!(options.warnings.len(), 1);
    let warning = serde_json::to_value(&options.warnings[0]).unwrap();
    let details = &warning["details"];
    assert_eq!(
        (
            warning["id"].as_str(),
            warning["effect"].as_str(),
            details["source_level"].as_str(),
            details["path_origin"].as_str(),
            details["reason_code"].as_str(),
        ),
        (
            Some("adapter_config_source_skipped"),
            Some("operation_continued"),
            Some("project"),
            Some("override"),
            Some("missing_override"),
        )
    );
}

#[test]
fn unsupported_config_native_option_does_not_enter_operation_request() {
    let dir = temp_dir("unsupported-native");
    let project_config = dir.join("project.json");
    write_json(
        &project_config,
        json!({
            "options": {
                "max_heading_level": 2
            }
        }),
    );
    let project_config_arg = path_arg(&project_config);

    let options = parse_for_test(
        Operation::Read,
        &[
            "doc.md",
            "--ref",
            "H:L1:H1",
            "--project-config-path",
            &project_config_arg,
        ],
        6000,
        &[MAX_HEADING_LEVEL],
    )
    .expect("parse read options");

    assert!(options.protocol_options().is_none());
}

fn warning_tokens(warning: &DirectCliWarning) -> Vec<&str> {
    let docnav_diagnostics::DiagnosticDetails::CliArgv { tokens } = warning.details() else {
        panic!("expected CLI argv warning details");
    };
    tokens.iter().map(String::as_str).collect()
}

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

fn parse_for_test(
    operation: Operation,
    values: &[&str],
    default_limit: u32,
    native_options: &[NativeOptionSpec],
) -> Result<DirectOperationOptions, String> {
    let config = test_config(default_limit, native_options);
    parse_operation_options(operation, &args(values), &config)
}

fn test_config<'a>(
    default_limit: u32,
    native_options: &'a [NativeOptionSpec],
) -> DirectCliConfig<'a> {
    DirectCliConfig {
        adapter_id: "test-adapter",
        program_name: "test-adapter",
        usage: "usage: test-adapter",
        request_id: "test-request",
        default_limit,
        default_user_config_dir: None,
        native_options,
    }
}

fn write_json(path: &Path, value: serde_json::Value) {
    fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn temp_dir(name: &str) -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "docnav-adapter-sdk-args-test-{}-{id}-{name}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}
