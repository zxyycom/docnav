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

    parse([
        "read",
        "doc.md",
        "--ref",
        "doc:full",
        "--auto-read",
        "disabled",
    ])
    .expect_err("unsupported command must reject auto-read");
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
