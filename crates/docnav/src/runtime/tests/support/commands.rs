use cli_config_resolution::{
    FieldIdentity, Source, SourceCandidate, SourceId, SourceKind, SourceLocator,
};
use docnav_navigation::{DOCUMENT_CLI_SOURCE_ID, DOCUMENT_CLI_SOURCE_PRIORITY};
use docnav_protocol::Operation;
use serde_json::{json, Value};

use crate::cli::DocumentCommand;

pub(in crate::runtime::tests) fn outline_command(
    max_heading_level: Option<u32>,
    adapter: Option<&str>,
) -> DocumentCommand {
    let mut candidates = vec![
        cli_value_candidate("docnav.defaults.pagination.limit", "--limit", json!(80)),
        cli_value_candidate("docnav.defaults.output", "--output", json!("protocol-json")),
    ];
    if let Some(value) = max_heading_level {
        candidates.push(cli_value_candidate(
            "docnav.adapters.docnav-markdown.options.max_heading_level",
            "--max-heading-level",
            json!(value),
        ));
    }
    if let Some(adapter) = adapter {
        candidates.push(cli_value_candidate(
            "docnav.defaults.adapter",
            "--adapter",
            json!(adapter),
        ));
    }
    DocumentCommand {
        operation: Operation::Outline,
        path: "docs/guide.md".to_owned(),
        ref_id: None,
        query: None,
        cli_source: cli_source(candidates),
        invocation_log: None,
        invocation_log_content_root: None,
        config_paths: Default::default(),
    }
}

pub(in crate::runtime::tests) fn read_command(ref_id: &str) -> DocumentCommand {
    DocumentCommand {
        operation: Operation::Read,
        path: "docs/guide.md".to_owned(),
        ref_id: Some(ref_id.to_owned()),
        query: None,
        cli_source: cli_source(vec![
            cli_value_candidate("docnav.defaults.pagination.limit", "--limit", json!(80)),
            cli_value_candidate("docnav.defaults.output", "--output", json!("protocol-json")),
        ]),
        invocation_log: None,
        invocation_log_content_root: None,
        config_paths: Default::default(),
    }
}

pub(in crate::runtime::tests) fn set_cli_value(
    command: &mut DocumentCommand,
    identity: &str,
    flag: &str,
    value: Value,
) {
    let mut candidates = command
        .cli_source
        .candidates()
        .iter()
        .filter(|candidate| candidate.field().as_str() != identity)
        .cloned()
        .collect::<Vec<_>>();
    candidates.push(cli_value_candidate(identity, flag, value));
    command.cli_source = cli_source(candidates);
}

pub(in crate::runtime::tests) fn cli_source(candidates: Vec<SourceCandidate>) -> Box<Source> {
    Box::new(
        Source::new(
            SourceId::new(DOCUMENT_CLI_SOURCE_ID).unwrap(),
            SourceKind::Cli,
            DOCUMENT_CLI_SOURCE_PRIORITY,
            candidates,
        )
        .unwrap(),
    )
}

fn cli_value_candidate(identity: &str, flag: &str, value: Value) -> SourceCandidate {
    SourceCandidate::value(
        FieldIdentity::new(identity).unwrap(),
        SourceLocator::CliFlag(flag.to_owned()),
        value,
    )
}
