use crate::error::DocnavExitCode;

use super::{assert_diagnostic, parse};

#[test]
fn auto_read_rejects_missing_duplicate_and_inapplicable_input_structurally() {
    let missing =
        parse(["outline", "doc.md", "--auto-read"]).expect_err("auto-read requires a value");
    assert_diagnostic(missing, "--auto-read", "missing_value");

    let duplicate = parse([
        "outline",
        "doc.md",
        "--auto-read",
        "disabled",
        "--auto-read",
        "unique-ref",
    ])
    .expect_err("auto-read is a single-value flag");
    assert_diagnostic(duplicate, "argv", "invalid command line arguments");

    for args in [
        vec![
            "read",
            "doc.md",
            "--ref",
            "doc:full",
            "--auto-read",
            "disabled",
        ],
        vec!["info", "doc.md", "--auto-read", "disabled"],
        vec!["version", "--auto-read", "disabled"],
    ] {
        parse(args).expect_err("unsupported command must reject auto-read");
    }
}

#[test]
fn max_heading_level_is_rejected_for_unsupported_operations() {
    for args in [
        vec![
            "read",
            "doc.md",
            "--ref",
            "doc:full",
            "--max-heading-level",
            "2",
        ],
        vec!["info", "doc.md", "--max-heading-level", "2"],
    ] {
        let error = parse(args).expect_err("operation should not accept max heading level");
        assert_diagnostic(error, "--max-heading-level", "unsupported_argument");
    }
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
        "readable-view",
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
