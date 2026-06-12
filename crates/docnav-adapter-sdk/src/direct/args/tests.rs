use super::super::warnings::{DirectCliWarningEffect, DirectCliWarningId};
use super::super::NativeOptionValueSpec;
use super::*;

const MAX_HEADING_LEVEL_OPERATIONS: &[Operation] = &[Operation::Outline, Operation::Find];
const MAX_HEADING_LEVEL: NativeOptionSpec = NativeOptionSpec {
    flag: "--max-heading-level",
    option_key: "max_heading_level",
    operations: MAX_HEADING_LEVEL_OPERATIONS,
    value: NativeOptionValueSpec::IntegerRange { min: 1, max: 6 },
    default: Some(NativeOptionDefault::Integer(3)),
};

#[test]
fn unknown_flag_does_not_consume_following_positional() {
    let options = parse_operation_options(
        Operation::Outline,
        &args(&["--future", "doc.md"]),
        6000,
        &[],
    )
    .expect("parse options");

    assert_eq!(options.path, "doc.md");
    assert_eq!(
        options.warnings,
        vec![DirectCliWarning::unknown_flag("--future")]
    );
}

#[test]
fn unknown_flag_with_equals_is_one_ignored_token() {
    let options = parse_operation_options(
        Operation::Outline,
        &args(&["--future=value", "doc.md"]),
        6000,
        &[],
    )
    .expect("parse options");

    assert_eq!(options.warnings.len(), 1);
    assert_eq!(options.warnings[0].id, DirectCliWarningId::CliArgvIgnored);
    assert_eq!(
        options.warnings[0].effect,
        DirectCliWarningEffect::OperationContinued
    );
    assert_eq!(options.warnings[0].details.tokens, ["--future=value"]);
}

#[test]
fn extra_positional_warns_after_path_slot_is_filled() {
    let options =
        parse_operation_options(Operation::Outline, &args(&["doc.md", "extra"]), 6000, &[])
            .expect("parse options");

    assert_eq!(options.path, "doc.md");
    assert_eq!(options.warnings.len(), 1);
    assert_eq!(options.warnings[0].id, DirectCliWarningId::CliArgvIgnored);
    assert_eq!(options.warnings[0].details.tokens, ["extra"]);
}

#[test]
fn operation_only_validates_flags_it_uses() {
    let read = parse_operation_options(
        Operation::Read,
        &args(&["doc.md", "--ref", "L1:Guide", "--max-heading-level", "nope"]),
        6000,
        &[MAX_HEADING_LEVEL],
    )
    .expect("unused native value should be ignored");
    assert_eq!(
        read.warnings[0].details.tokens,
        ["--max-heading-level", "nope"]
    );

    let info = parse_operation_options(
        Operation::Info,
        &args(&["doc.md", "--limit-chars", "nope"]),
        6000,
        &[],
    )
    .expect("unused core value should be ignored");
    assert_eq!(info.limit_chars.get(), 6000);
    assert_eq!(info.warnings[0].details.tokens, ["--limit-chars", "nope"]);
}

#[test]
fn unused_value_flag_consumes_value_that_looks_like_flag() {
    let options = parse_operation_options(
        Operation::Read,
        &args(&["doc.md", "--ref", "L1:Guide", "--query", "--future-value"]),
        6000,
        &[],
    )
    .expect("parse options");

    assert_eq!(options.ref_id.as_deref(), Some("L1:Guide"));
    assert_eq!(
        options.warnings[0].details.tokens,
        ["--query", "--future-value"]
    );
}

#[test]
fn used_value_flag_accepts_value_that_looks_like_flag() {
    let options = parse_operation_options(
        Operation::Read,
        &args(&["doc.md", "--ref", "--future-value"]),
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
    assert_eq!(warnings[0].details.tokens, ["--future"]);
    assert_eq!(warnings[1].details.tokens, ["extra"]);
}

#[test]
fn probe_path_can_follow_unknown_flag() {
    let parsed = parse_probe(
        &args(&["--future", "doc.md", "--output", "protocol-json"]),
        &[],
    )
    .expect("parse probe options");

    assert_eq!(parsed.path, "doc.md");
    assert_eq!(parsed.warnings[0].details.tokens, ["--future"]);
}

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}
