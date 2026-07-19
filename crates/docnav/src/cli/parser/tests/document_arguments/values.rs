use cli_config_resolution::{CandidateInput, SourceLocator};
use serde_json::json;

use crate::cli::CliCommand;

use super::{candidate, parse};

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
fn auto_read_modes_keep_the_canonical_identity_and_exact_tokens() {
    for (args, expected) in [
        (
            vec!["outline", "doc.md", "--auto-read", "unique-ref"],
            "unique-ref",
        ),
        (
            vec![
                "find",
                "doc.md",
                "--query",
                "needle",
                "--auto-read",
                "disabled",
            ],
            "disabled",
        ),
    ] {
        let parsed = parse(args).expect("parse supported auto-read mode");
        let CliCommand::Document(command) = parsed.command else {
            panic!("expected document command");
        };
        let candidate = candidate(&command, "docnav.defaults.auto_read");
        assert_eq!(
            candidate.locator(),
            &SourceLocator::CliFlag("--auto-read".to_owned())
        );
        assert_eq!(candidate.input(), &CandidateInput::Value(json!(expected)));
    }
}

#[test]
fn invalid_auto_read_token_is_preserved_for_selected_validation() {
    let parsed = parse(["outline", "doc.md", "--auto-read", "sometimes"])
        .expect("structural parse preserves the canonical candidate");
    let CliCommand::Document(command) = parsed.command else {
        panic!("expected document command");
    };

    assert_eq!(
        candidate(&command, "docnav.defaults.auto_read").input(),
        &CandidateInput::Value(json!("sometimes"))
    );
}

#[test]
fn explicit_max_heading_level_value_is_parsed_for_supported_operations() {
    for args in [
        vec!["outline", "doc.md", "--max-heading-level", "2"],
        vec![
            "find",
            "doc.md",
            "--query",
            "needle",
            "--max-heading-level",
            "2",
        ],
    ] {
        let parsed = parse(args).expect("parse max heading level");

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
