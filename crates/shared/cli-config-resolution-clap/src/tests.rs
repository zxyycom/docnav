use clap::error::ErrorKind;
use clap::{Arg, Command};
use cli_config_resolution::{
    CandidateInput, ExpectedFieldShape, FieldDef, FieldDefSet, FieldValidation, JsonValue,
    ProcessStrategy, ProcessingId, ProcessingLocator, SourceId, SourceKind, SourceLocator,
    ValueKind,
};
use serde_json::json;

use super::{augment_command, extract_cli, ClapProjectionError};

// @case WB-PARAM-CLAP-001
fn id(value: &str) -> ProcessingId {
    ProcessingId::new(value).expect("valid processing id")
}

fn canonical_fields() -> FieldDefSet {
    FieldDefSet::builder()
        .field(
            FieldDef::builder("name")
                .process("cli", ProcessStrategy::cli_flag("--name"))
                .validation(FieldValidation::string()),
            ExpectedFieldShape::optional(),
        )
        .field(
            FieldDef::builder("count")
                .process("cli", ProcessStrategy::cli_flag("--count"))
                .validation(FieldValidation::int()),
            ExpectedFieldShape::optional(),
        )
        .field(
            FieldDef::builder("ratio")
                .process("cli", ProcessStrategy::cli_flag("--ratio"))
                .validation(FieldValidation::num()),
            ExpectedFieldShape::optional(),
        )
        .field(
            FieldDef::builder("verbose")
                .process("cli", ProcessStrategy::cli_flag("--verbose"))
                .validation(FieldValidation::boolean()),
            ExpectedFieldShape::optional(),
        )
        .field(
            FieldDef::builder("include")
                .process("cli", ProcessStrategy::cli_flag("--include"))
                .validation(FieldValidation::array()),
            ExpectedFieldShape::optional(),
        )
        .field(
            FieldDef::builder("define")
                .process("cli", ProcessStrategy::cli_flag("--define"))
                .validation(FieldValidation::object()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("canonical fields")
}

#[test]
fn canonical_metadata_registers_and_extracts_supported_cli_values() {
    let fields = canonical_fields();
    let command = augment_command(Command::new("demo"), &fields, &id("cli")).expect("command");
    let matches = command
        .try_get_matches_from([
            "demo",
            "--name",
            "docs",
            "--count",
            "42",
            "--ratio",
            "1.5",
            "--verbose",
            "--include",
            "src",
            "--include",
            "tests",
            "--define",
            "team=docs",
            "--define",
            "stage=review",
        ])
        .expect("matches");

    let source = extract_cli(
        &matches,
        &fields,
        &id("cli"),
        SourceId::new("cli").expect("source id"),
        40,
    )
    .expect("CLI source");

    assert_eq!(source.id().as_str(), "cli");
    assert_eq!(source.kind(), &SourceKind::Cli);
    assert_eq!(source.priority(), 40);
    assert_candidate(&source, "name", json!("docs"));
    assert_candidate(&source, "count", json!(42));
    assert_candidate(&source, "ratio", json!(1.5));
    assert_candidate(&source, "verbose", json!(true));
    assert_candidate(&source, "include", json!(["src", "tests"]));
    assert_candidate(
        &source,
        "define",
        json!({"stage": "review", "team": "docs"}),
    );
    assert!(source.candidates().iter().all(|candidate| {
        matches!(candidate.locator(), SourceLocator::CliFlag(flag) if flag.starts_with("--"))
    }));
}

#[test]
fn omitted_set_true_flag_does_not_produce_a_candidate() {
    let fields = FieldDefSet::builder()
        .field(
            FieldDef::builder("verbose")
                .process("cli", ProcessStrategy::cli_flag("--verbose"))
                .validation(FieldValidation::boolean()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("fields");
    let command = augment_command(Command::new("demo"), &fields, &id("cli")).expect("command");
    let matches = command.try_get_matches_from(["demo"]).expect("matches");

    assert_eq!(
        matches.value_source("verbose"),
        Some(clap::parser::ValueSource::DefaultValue)
    );
    let source = extract_cli(
        &matches,
        &fields,
        &id("cli"),
        SourceId::new("cli").expect("source id"),
        40,
    )
    .expect("CLI source");
    assert!(source.candidates().is_empty());
}

#[test]
fn decoded_cli_failure_is_an_invalid_candidate_with_raw_input() {
    let fields = canonical_fields();
    let command = augment_command(Command::new("demo"), &fields, &id("cli")).expect("command");
    let matches = command
        .try_get_matches_from(["demo", "--count", "many"])
        .expect("clap tokenization succeeds");

    let source = extract_cli(
        &matches,
        &fields,
        &id("cli"),
        SourceId::new("cli").expect("source id"),
        40,
    )
    .expect("invalid input is a candidate fact");
    let candidate = candidate(&source, "count");
    assert_eq!(
        candidate.locator(),
        &SourceLocator::CliFlag("--count".to_owned())
    );
    assert_eq!(
        candidate.input(),
        &CandidateInput::Invalid {
            raw: json!("many"),
            reason: "expected integer CLI value".to_owned(),
        }
    );
}

#[test]
fn unregistered_flag_uses_clap_native_unknown_argument_error() {
    let fields = canonical_fields();
    let command = augment_command(Command::new("demo"), &fields, &id("cli")).expect("command");

    let error = command
        .try_get_matches_from(["demo", "--unregistered"])
        .expect_err("clap rejects unknown flags");
    assert_eq!(error.kind(), ErrorKind::UnknownArgument);
}

#[test]
fn non_cli_processing_locator_is_an_adapter_error() {
    let env_fields = FieldDefSet::builder()
        .field(
            FieldDef::builder("name")
                .process("shared", ProcessStrategy::env_var("APP_NAME"))
                .validation(FieldValidation::string()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("env fields");
    let error = augment_command(Command::new("demo"), &env_fields, &id("shared"))
        .expect_err("non-CLI locator is rejected");
    assert!(matches!(
        error,
        ClapProjectionError::UnsupportedLocator {
            locator: ProcessingLocator::EnvVar(name),
            ..
        } if name == "APP_NAME"
    ));
}

#[test]
fn unsupported_json_value_kind_is_an_adapter_error() {
    let json_fields = FieldDefSet::builder()
        .field(
            FieldDef::builder("payload")
                .process("cli", ProcessStrategy::cli_flag("--payload"))
                .validation(FieldValidation::json()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("JSON field");
    let error = augment_command(Command::new("demo"), &json_fields, &id("cli"))
        .expect_err("generic JSON has no implicit CLI syntax");
    assert!(matches!(
        error,
        ClapProjectionError::UnsupportedValueKind {
            value_kind: ValueKind::Json,
            ..
        }
    ));
}

#[test]
fn existing_command_flag_conflict_returns_an_adapter_error() {
    let fields = single_string_field("--name");
    let command = Command::new("demo").arg(Arg::new("existing-name").long("name"));
    assert!(matches!(
        augment_command(command, &fields, &id("cli")),
        Err(ClapProjectionError::ArgumentConflict { flag, .. }) if flag == "--name"
    ));
}

#[test]
fn mismatched_match_set_returns_an_adapter_error() {
    let fields = single_string_field("--name");
    let unrelated_matches = Command::new("demo")
        .try_get_matches_from(["demo"])
        .expect("matches");
    assert!(matches!(
        extract_cli(
            &unrelated_matches,
            &fields,
            &id("cli"),
            SourceId::new("cli").expect("source id"),
            40,
        ),
        Err(ClapProjectionError::MatchRead { flag, .. }) if flag == "--name"
    ));
}

#[test]
fn implicit_group_flag_conflict_is_an_adapter_error() {
    let flag = "--shared";
    let fields = single_string_field(flag);
    let command = Command::new("demo").arg(Arg::new("mode").long("mode").group("shared"));

    assert!(matches!(
        augment_command(command, &fields, &id("cli")),
        Err(ClapProjectionError::ArgumentConflict {
            flag: conflicting_flag,
            ..
        }) if conflicting_flag == flag
    ));
}

#[test]
fn direct_subcommand_flag_conflict_is_an_adapter_error() {
    let flag = "--sync-now";
    let fields = single_string_field(flag);
    let command = Command::new("demo").subcommand(Command::new("sync").long_flag("sync-now"));

    assert!(matches!(
        augment_command(command, &fields, &id("cli")),
        Err(ClapProjectionError::ArgumentConflict {
            flag: conflicting_flag,
            ..
        }) if conflicting_flag == flag
    ));
}

#[test]
fn negative_integer_and_number_tokens_are_extracted_as_json_numbers() {
    let fields = FieldDefSet::builder()
        .field(
            FieldDef::builder("offset")
                .process("cli", ProcessStrategy::cli_flag("--offset"))
                .validation(FieldValidation::int()),
            ExpectedFieldShape::optional(),
        )
        .field(
            FieldDef::builder("ratio")
                .process("cli", ProcessStrategy::cli_flag("--ratio"))
                .validation(FieldValidation::num()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("numeric fields");
    let command = augment_command(Command::new("demo"), &fields, &id("cli")).expect("command");
    let matches = command
        .try_get_matches_from(["demo", "--offset", "-12", "--ratio", "-0.25"])
        .expect("negative numeric tokens");
    let source = extract_cli(
        &matches,
        &fields,
        &id("cli"),
        SourceId::new("cli").expect("source id"),
        40,
    )
    .expect("CLI source");

    assert_candidate(&source, "offset", json!(-12));
    assert_candidate(&source, "ratio", json!(-0.25));
}

fn single_string_field(flag: &str) -> FieldDefSet {
    FieldDefSet::builder()
        .field(
            FieldDef::builder("value")
                .process("cli", ProcessStrategy::cli_flag(flag))
                .validation(FieldValidation::string()),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("field")
}

fn assert_candidate(source: &cli_config_resolution::Source, field: &str, expected: JsonValue) {
    assert_eq!(
        candidate(source, field).input(),
        &CandidateInput::Value(expected)
    );
}

fn candidate<'a>(
    source: &'a cli_config_resolution::Source,
    field: &str,
) -> &'a cli_config_resolution::SourceCandidate {
    source
        .candidates()
        .iter()
        .find(|candidate| candidate.field().as_str() == field)
        .expect("candidate")
}
