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
            for (argument, description) in [
                ("<path>", "Document path to navigate"),
                ("--invocation-log <path>", "Write invocation events"),
                (
                    "--invocation-log-content-root <path>",
                    "Store captured invocation content",
                ),
                ("--project-config <path>", "Read project configuration"),
                ("--user-config <path>", "Read user configuration"),
            ] {
                assert!(
                    text.contains(argument) && text.contains(description),
                    "outline help should explain {argument}; got:\n{text}"
                );
            }
        }
        command => panic!("expected help command, got {command:?}"),
    }
}

#[test]
fn help_text_shows_only_public_output_modes() {
    let parsed = parse(["outline", "--help"]).expect("parse help");

    match parsed.command {
        CliCommand::Help(text) => {
            assert!(
                text.contains("readable-view"),
                "help should list readable-view; got:\n{text}"
            );
            assert!(
                !text.contains("readable-json"),
                "help should not list removed readable-json; got:\n{text}"
            );
            assert!(
                text.contains("protocol-json"),
                "help should list protocol-json; got:\n{text}"
            );
            assert!(
                !text.contains("text|protocol-json"),
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
    let find = parse(["find", "--help"]).expect("parse find help");

    match (outline.command, read.command, find.command) {
        (
            CliCommand::Help(outline_text),
            CliCommand::Help(read_text),
            CliCommand::Help(find_text),
        ) => {
            assert!(
                outline_text.contains("--max-heading-level"),
                "outline help should list the Markdown catalog parameter; got:\n{outline_text}"
            );
            assert!(
                !read_text.contains("--max-heading-level"),
                "read help should not list the Markdown catalog parameter; got:\n{read_text}"
            );
            assert!(
                read_text.contains("--ref <ref>")
                    && read_text.contains("Adapter-owned document region reference"),
                "read help should explain its static ref argument; got:\n{read_text}"
            );
            assert!(
                find_text.contains("--query <text>")
                    && find_text.contains("Text to find in the document"),
                "find help should explain its static query argument; got:\n{find_text}"
            );
            for text in [&outline_text, &find_text] {
                assert!(
                    text.contains("--auto-read <disabled|unique-ref>"),
                    "outline/find help should show exact auto-read tokens; got:\n{text}"
                );
                assert!(
                    text.contains("possible values: disabled, unique-ref")
                        && text.contains("default: unique-ref"),
                    "auto-read help should derive enum and default facts; got:\n{text}"
                );
            }
            assert!(
                !read_text.contains("--auto-read"),
                "read help must not expose auto-read; got:\n{read_text}"
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
fn help_flag_after_terminator_is_parsed_as_document_path() {
    let parsed = parse(["outline", "--", "--help"]).expect("parse terminated help token");

    match parsed.command {
        CliCommand::Document(command) => assert_eq!(command.path, "--help"),
        command => panic!("expected document command, got {command:?}"),
    }
}

#[test]
fn unknown_command_with_help_remains_input_error() {
    let error = parse(["bogus", "--help"]).expect_err("unknown command remains invalid");
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["field"], "command");
    assert!(
        details["reason"]
            .as_str()
            .is_some_and(|reason| reason.contains("unknown command")),
        "expected unknown-command reason, got {details}"
    );
}
