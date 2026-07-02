use super::super::parse;
use crate::cli::CliCommand;

#[test]
fn help_returns_typed_help_command() {
    let parsed = parse(["outline", "--help"]).expect("parse help");

    match parsed.command {
        CliCommand::Help(text) => {
            assert!(text.contains("Usage:"));
            assert!(text.contains("--output"));
            assert!(text.contains("outline"));
        }
        command => panic!("expected help command, got {command:?}"),
    }
}

#[test]
fn help_text_shows_three_output_modes() {
    let parsed = parse(["outline", "--help"]).expect("parse help");

    match parsed.command {
        CliCommand::Help(text) => {
            assert!(
                text.contains("readable-view"),
                "help should list readable-view; got:\n{text}"
            );
            assert!(
                text.contains("readable-json"),
                "help should list readable-json; got:\n{text}"
            );
            assert!(
                text.contains("protocol-json"),
                "help should list protocol-json; got:\n{text}"
            );
            assert!(
                !text.contains("text|readable-json|protocol-json"),
                "help should not show legacy 'text' output value"
            );
        }
        command => panic!("expected help command, got {command:?}"),
    }
}

#[test]
fn help_command_has_no_output_mode() {
    let parsed = parse(["--help"]).expect("parse --help");
    match parsed.command {
        CliCommand::Help(_) => {}
        command => panic!("expected help command, got {command:?}"),
    }
}

#[test]
fn version_command_has_no_output_mode() {
    let parsed = parse(["version"]).expect("parse version");
    match parsed.command {
        CliCommand::Version => {}
        command => panic!("expected version command, got {command:?}"),
    }
}
