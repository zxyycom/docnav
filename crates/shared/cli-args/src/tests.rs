// @case WB-CLIARGS-BOUNDARY-001
use super::*;

#[test]
fn unknown_flag_does_not_consume_following_positional() {
    let result = scan(&["--future", "doc.md"], 1, &[]).unwrap();

    assert_eq!(result.retained_args, ["doc.md"]);
    assert_eq!(
        result.rejected,
        [RejectedArg::UnknownFlag {
            token: "--future".to_owned()
        }]
    );
}

#[test]
fn used_value_flag_is_retained_and_consumes_value() {
    let flags = [KnownValueFlag::used("--ref")];
    let result = scan(&["doc.md", "--ref", "--future-value"], 1, &flags).unwrap();

    assert_eq!(result.retained_args, ["doc.md", "--ref", "--future-value"]);
    assert!(result.rejected.is_empty());
}

#[test]
fn used_value_flag_requires_a_value_before_known_value_flag() {
    let flags = [
        KnownValueFlag::used("--project-config"),
        KnownValueFlag::used("--user-config"),
    ];
    let error = scan(&["doc.md", "--project-config", "--user-config"], 1, &flags).unwrap_err();

    assert_eq!(error.flag(), "--project-config");
}

#[test]
fn used_value_flag_allows_unknown_hyphen_value() {
    let flags = [KnownValueFlag::used("--project-config")];
    let result = scan(&["doc.md", "--project-config", "--custom-path"], 1, &flags).unwrap();

    assert_eq!(
        result.retained_args,
        ["doc.md", "--project-config", "--custom-path"]
    );
    assert!(result.rejected.is_empty());
}

#[test]
fn unused_value_flag_records_fact_without_validating_value() {
    let flags = [KnownValueFlag::unused("--page")];
    let result = scan(&["doc.md", "--page", "nope"], 1, &flags).unwrap();

    assert_eq!(result.retained_args, ["doc.md"]);
    assert_eq!(
        result.rejected,
        [RejectedArg::UnusedValueFlag {
            flag: "--page".to_owned(),
            value: Some("nope".to_owned()),
            command: "info".to_owned()
        }]
    );
}

#[test]
fn unused_value_flag_requires_a_value() {
    let flags = [KnownValueFlag::unused("--page")];
    let error = scan(&["doc.md", "--page"], 1, &flags).unwrap_err();

    assert_eq!(error.flag(), "--page");
}

#[test]
fn switch_flags_are_retained_without_consuming_value() {
    let config = ArgBoundaryScan::new("config get", 1, &[]).with_known_switch_flags(&["--user"]);
    let result = scan_arg_boundaries(&args(&["--user", "key"]), &config).unwrap();

    assert_eq!(result.retained_args, ["--user", "key"]);
    assert!(result.rejected.is_empty());
}

fn scan(
    values: &[&str],
    positional_limit: usize,
    known_value_flags: &[KnownValueFlag<'_>],
) -> Result<ArgBoundaryScanResult, MissingValue> {
    let config = ArgBoundaryScan::new("info", positional_limit, known_value_flags);
    scan_arg_boundaries(&args(values), &config)
}

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}
