use super::super::parse;
use crate::cli::CliCommand;
use cli_config_resolution::CandidateInput;

#[test]
fn parse_without_output_has_none() {
    let parsed = parse(["outline", "doc.md"]).expect("parse with default output");

    match parsed.command {
        CliCommand::Document(command) => {
            assert!(command.cli_source.candidates().is_empty());
        }
        command => panic!("expected document command, got {command:?}"),
    }
}

#[test]
fn parse_explicit_protocol_json() {
    let parsed =
        parse(["outline", "doc.md", "--output", "protocol-json"]).expect("parse protocol-json");

    match parsed.command {
        CliCommand::Document(command) => {
            assert_eq!(
                command
                    .cli_source
                    .candidates()
                    .iter()
                    .find(|candidate| candidate.field().as_str() == "docnav.defaults.output")
                    .map(|candidate| candidate.input()),
                Some(&CandidateInput::Value(serde_json::json!("protocol-json")))
            );
        }
        command => panic!("expected document command, got {command:?}"),
    }
}

#[test]
fn removed_output_value_remains_a_canonical_candidate_for_navigation_validation() {
    let parsed = parse(["outline", "doc.md", "--output", "readable-json"])
        .expect("enum validation belongs to selected canonical resolution");
    let CliCommand::Document(command) = parsed.command else {
        panic!("expected document command");
    };
    assert_eq!(
        command
            .cli_source
            .candidates()
            .iter()
            .find(|candidate| candidate.field().as_str() == "docnav.defaults.output")
            .map(|candidate| candidate.input()),
        Some(&CandidateInput::Value(serde_json::json!("readable-json")))
    );
}
