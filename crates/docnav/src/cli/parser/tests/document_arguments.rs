use super::super::{parse, parse_with_registry};
use crate::cli::{CliCommand, OutputMode};
use crate::error::{AppError, DocnavExitCode};
use crate::output::{write_error, ErrorOutput};
use cli_config_resolution::{CandidateInput, SourceLocator};
use docnav_adapter_contracts::{
    AdapterDefinition, AdapterOptionSpec, CliBooleanEncoding, CliProcessingMetadata,
    FieldValidation, ProcessStrategy,
};
use docnav_navigation::{NavigationAdapterRef, NavigationAdapterRegistry};
use docnav_protocol::Operation;
use serde_json::{json, Value};

// @case WB-CORE-ARGS-001
#[test]
fn generated_page_keeps_canonical_identity_for_selected_validation() {
    let parsed = parse(["outline", "doc.md", "--page", "0"]).expect("structural parse");
    let CliCommand::Document(command) = parsed.command else {
        panic!("expected document command");
    };
    let candidate = candidate(&command, "docnav.document.page");
    assert_eq!(
        candidate.locator(),
        &SourceLocator::CliFlag("--page".to_owned())
    );
    assert_eq!(candidate.input(), &CandidateInput::Value(json!(0)));
}

#[test]
fn explicit_pagination_value_is_parsed() {
    let parsed = parse(["outline", "doc.md", "--pagination", "disabled"])
        .expect("parse pagination disabled");

    match parsed.command {
        CliCommand::Document(command) => {
            assert_eq!(
                candidate(&command, "docnav.defaults.pagination.enabled").input(),
                &CandidateInput::Value(json!(false))
            );
        }
        command => panic!("expected document command, got {command:?}"),
    }
}

#[test]
fn explicit_max_heading_level_value_is_parsed_for_outline() {
    let parsed =
        parse(["outline", "doc.md", "--max-heading-level", "2"]).expect("parse max heading level");

    match parsed.command {
        CliCommand::Document(command) => {
            let candidate = candidate(
                &command,
                "docnav.adapters.docnav-markdown.options.max_heading_level",
            );
            assert_eq!(
                candidate.locator(),
                &SourceLocator::CliFlag("--max-heading-level".to_owned())
            );
            assert_eq!(candidate.input(), &CandidateInput::Value(json!(2)));
        }
        command => panic!("expected document command, got {command:?}"),
    }
}

#[test]
fn generated_short_value_flag_reaches_canonical_candidate() {
    let parsed = parse_with_registry(["outline", "doc.md", "-x", "2"], &ShortOptionRegistry)
        .expect("generated short value flag parses");
    let CliCommand::Document(command) = parsed.command else {
        panic!("expected document command");
    };
    let candidate = candidate(&command, SHORT_VALUE_IDENTITY);

    assert_eq!(
        candidate.locator(),
        &SourceLocator::CliFlag("-x".to_owned())
    );
    assert_eq!(candidate.input(), &CandidateInput::Value(json!(2)));
}

#[test]
fn generated_short_presence_switch_reaches_canonical_candidate() {
    let parsed = parse_with_registry(["outline", "doc.md", "-v"], &ShortOptionRegistry)
        .expect("generated short presence switch parses");
    let CliCommand::Document(command) = parsed.command else {
        panic!("expected document command");
    };
    let candidate = candidate(&command, SHORT_SWITCH_IDENTITY);

    assert_eq!(
        candidate.locator(),
        &SourceLocator::CliFlag("-v".to_owned())
    );
    assert_eq!(candidate.input(), &CandidateInput::Value(json!(true)));
}

#[test]
fn generated_short_options_are_rejected_for_other_operation() {
    for (args, flag) in [
        (vec!["read", "doc.md", "--ref", "doc:full", "-x", "2"], "-x"),
        (vec!["read", "doc.md", "--ref", "doc:full", "-v"], "-v"),
    ] {
        let error = parse_with_registry(args, &ShortOptionRegistry)
            .expect_err("outline-only short option must be rejected for read");
        let details = error.diagnostic().details().to_value();

        assert_eq!(details.get("field").and_then(Value::as_str), Some(flag));
        assert_eq!(
            details.get("reason").and_then(Value::as_str),
            Some("unsupported_argument")
        );
    }
}

#[test]
fn max_heading_level_is_rejected_for_read() {
    let error = parse([
        "read",
        "doc.md",
        "--ref",
        "doc:full",
        "--max-heading-level",
        "2",
    ])
    .expect_err("read should not accept max heading level");
    assert_diagnostic(error, "--max-heading-level", "unsupported_argument");
}

#[test]
fn invalid_pagination_token_is_preserved_for_selected_validation() {
    let parsed = parse(["outline", "doc.md", "--pagination", "off"])
        .expect("structural parse preserves field-local decode failure");
    let CliCommand::Document(command) = parsed.command else {
        panic!("expected document command");
    };
    let candidate = candidate(&command, "docnav.defaults.pagination.enabled");
    assert!(matches!(
        candidate.input(),
        CandidateInput::Invalid { raw, reason }
            if raw == &json!("off") && reason.contains("Boolean CLI token")
    ));
}

#[test]
fn generated_value_flag_without_value_maps_clap_structural_error() {
    let error = parse(["outline", "doc.md", "--max-heading-level"])
        .expect_err("generated value flag requires a value");

    assert_eq!(error.exit_code().code(), DocnavExitCode::InputError.code());
    assert_diagnostic(error, "--max-heading-level", "missing_value");
}

#[test]
fn duplicate_generated_single_value_flag_is_rejected_structurally() {
    let error = parse([
        "outline",
        "doc.md",
        "--max-heading-level",
        "2",
        "--max-heading-level",
        "3",
    ])
    .expect_err("generated single-value flag must not repeat");

    assert_eq!(error.exit_code().code(), DocnavExitCode::InputError.code());
    assert_diagnostic(error, "argv", "invalid command line arguments");
}

#[test]
fn unused_known_argument_value_is_rejected_before_execution() {
    let error = parse([
        "info",
        "doc.md",
        "--page",
        "nope",
        "--output",
        "readable-json",
    ])
    .expect_err("unused page should fail info");

    assert_eq!(error.exit_code().code(), DocnavExitCode::InputError.code());
    assert_diagnostic(error, "--page", "unsupported_argument");
}

#[test]
fn unknown_document_argument_is_rejected() {
    let error = parse(["outline", "--future", "doc.md"]).expect_err("unknown argument should fail");

    assert_eq!(error.exit_code().code(), DocnavExitCode::InputError.code());
    assert_diagnostic(error, "argv", "unknown_argument");
}

#[test]
fn extra_document_positional_is_rejected() {
    let error = parse(["outline", "doc.md", "extra.md"]).expect_err("extra positional should fail");

    assert_eq!(error.exit_code().code(), DocnavExitCode::InputError.code());
    assert_diagnostic(error, "argv", "extra_positional");
}

// @case WB-CORE-ARGS-REPAIR-001
#[test]
fn unknown_document_argument_protocol_error_has_repair_context() {
    let error = parse([
        "outline",
        "docs/navigation.md",
        "--bogus",
        "--output",
        "protocol-json",
    ])
    .expect_err("unknown argument should fail");
    let output = protocol_error_output(&error, Operation::Outline);

    assert_protocol_error_context(&output, "unknown_argument", "--bogus");
    assert_expected_contains(&output, "supported option");
    assert_guidance_contains(&output, "Remove");
}

#[test]
fn extra_document_positional_protocol_error_has_repair_context() {
    let error = parse([
        "outline",
        "docs/navigation.md",
        "extra.md",
        "--output",
        "protocol-json",
    ])
    .expect_err("extra positional should fail");
    let output = protocol_error_output(&error, Operation::Outline);

    assert_protocol_error_context(&output, "extra_positional", "extra.md");
    assert_expected_contains(&output, "positional arguments");
    assert_guidance_contains(&output, "Remove");
}

#[test]
fn unsupported_info_page_protocol_error_has_repair_context() {
    let error = parse([
        "info",
        "docs/navigation.md",
        "--page",
        "2",
        "--output",
        "protocol-json",
    ])
    .expect_err("info should not accept page");
    let output = protocol_error_output(&error, Operation::Info);

    assert_protocol_error_context(&output, "unsupported_argument", "--page 2");
    assert_expected_contains(&output, "info <path>");
    assert_guidance_contains(&output, "Remove --page");
}

fn assert_diagnostic(error: AppError, field: &str, reason_fragment: &str) {
    let details = error.diagnostic().details().to_value();
    assert_eq!(
        details.get("field").and_then(serde_json::Value::as_str),
        Some(field)
    );
    assert!(details
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|reason| reason.contains(reason_fragment)));
}

fn candidate<'a>(
    command: &'a crate::cli::DocumentCommand,
    identity: &str,
) -> &'a cli_config_resolution::SourceCandidate {
    command
        .cli_source
        .candidates()
        .iter()
        .find(|candidate| candidate.field().as_str() == identity)
        .unwrap_or_else(|| panic!("missing candidate {identity}"))
}

const SHORT_VALUE_IDENTITY: &str = "docnav.adapters.docnav-markdown.options.short_heading_level";
const SHORT_SWITCH_IDENTITY: &str = "docnav.adapters.docnav-markdown.options.short_verbose";

struct ShortOptionRegistry;

impl NavigationAdapterRegistry for ShortOptionRegistry {
    fn adapters(&self) -> Vec<NavigationAdapterRef<'_>> {
        let base = docnav_markdown::markdown_adapter_definition();
        let short_value = AdapterOptionSpec::builder(SHORT_VALUE_IDENTITY)
            .owner("docnav-markdown")
            .operations(&[Operation::Outline])
            .path(["options", "short_heading_level"])
            .process(
                "cli",
                ProcessStrategy::cli_flag("-x").cli_metadata(
                    CliProcessingMetadata::new()
                        .help("Set a short heading level")
                        .value_name("level"),
                ),
            )
            .validation(FieldValidation::int())
            .build();
        let short_switch = AdapterOptionSpec::builder(SHORT_SWITCH_IDENTITY)
            .owner("docnav-markdown")
            .operations(&[Operation::Outline])
            .path(["options", "short_verbose"])
            .process(
                "cli",
                ProcessStrategy::cli_flag("-v").cli_metadata(
                    CliProcessingMetadata::new()
                        .help("Enable short verbose output")
                        .boolean_encoding(CliBooleanEncoding::PresenceMeansTrue),
                ),
            )
            .validation(FieldValidation::boolean())
            .build();
        let definition = AdapterDefinition::builder(base.id())
            .adapter(base.adapter())
            .manifest(base.manifest().clone())
            .required_operation_handlers()
            .native_option(short_value)
            .native_option(short_switch)
            .build()
            .expect("short-option adapter definition");
        vec![NavigationAdapterRef::new(definition)]
    }
}

fn protocol_error_output(error: &AppError, operation: Operation) -> Value {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = write_error(ErrorOutput {
        error,
        output_mode: OutputMode::ProtocolJson,
        operation: Some(operation),
        stdout: &mut stdout,
        stderr: &mut stderr,
    });

    assert_eq!(exit, DocnavExitCode::InputError.code());
    assert!(stderr.is_empty());
    serde_json::from_slice(&stdout).expect("protocol-json failure parses")
}

fn assert_protocol_error_context(output: &Value, reason: &str, received: &str) {
    assert_eq!(output["ok"], false);
    assert_eq!(output["error"]["code"], "INVALID_REQUEST");
    assert_eq!(output["error"]["details"]["reason"], reason);
    assert_eq!(output["error"]["received"], json!(received));
    assert!(
        output["error"].get("expected").is_some(),
        "expected protocol error.expected to be present: {output}"
    );
    assert!(
        output["error"]
            .get("guidance")
            .and_then(Value::as_array)
            .is_some_and(|guidance| !guidance.is_empty()),
        "expected protocol error.guidance to be non-empty: {output}"
    );
}

fn assert_expected_contains(output: &Value, fragment: &str) {
    let expected = output["error"]["expected"]
        .as_array()
        .expect("expected is projected from accepted values");
    assert!(
        expected
            .iter()
            .filter_map(Value::as_str)
            .any(|value| value.contains(fragment)),
        "expected should contain {fragment:?}, got {expected:?}"
    );
}

fn assert_guidance_contains(output: &Value, fragment: &str) {
    let guidance = output["error"]["guidance"]
        .as_array()
        .expect("guidance is an array");
    assert!(
        guidance
            .iter()
            .filter_map(Value::as_str)
            .any(|value| value.contains(fragment)),
        "guidance should contain {fragment:?}, got {guidance:?}"
    );
}
