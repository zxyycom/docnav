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
fn help_text_scopes_catalog_parameters_to_supported_operations() {
    let outline = parse(["outline", "--help"]).expect("parse outline help");
    let read = parse(["read", "--help"]).expect("parse read help");

    match (outline.command, read.command) {
        (CliCommand::Help(outline_text), CliCommand::Help(read_text)) => {
            assert!(
                outline_text.contains("--max-heading-level"),
                "outline help should list the Markdown catalog parameter; got:\n{outline_text}"
            );
            assert!(
                !read_text.contains("--max-heading-level"),
                "read help should not list the Markdown catalog parameter; got:\n{read_text}"
            );
        }
        commands => panic!("expected help commands, got {commands:?}"),
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

#[test]
fn non_document_surfaces_keep_their_own_command_shapes() {
    for args in [
        vec!["--help"],
        vec!["version"],
        vec!["config", "inspect"],
        vec!["adapter", "list"],
        vec!["init"],
        vec!["doctor"],
    ] {
        parse(args).expect("non-document parsing succeeds");
    }
}
