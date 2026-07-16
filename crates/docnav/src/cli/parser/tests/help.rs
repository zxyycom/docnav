use super::super::{parse, parse_with_registry};
use crate::cli::CliCommand;
use docnav_adapter_contracts::{
    AdapterDefinition, AdapterOptionSpec, CliBooleanEncoding, CliProcessingMetadata,
    FieldValidation, ProcessStrategy,
};
use docnav_navigation::{NavigationAdapterRef, NavigationAdapterRegistry};
use docnav_protocol::Operation;

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
fn help_text_scopes_native_options_to_supported_operations() {
    let outline = parse(["outline", "--help"]).expect("parse outline help");
    let read = parse(["read", "--help"]).expect("parse read help");

    match (outline.command, read.command) {
        (CliCommand::Help(outline_text), CliCommand::Help(read_text)) => {
            assert!(
                outline_text.contains("--max-heading-level"),
                "outline help should list markdown outline native option; got:\n{outline_text}"
            );
            assert!(
                !read_text.contains("--max-heading-level"),
                "read help should not list outline-only native option; got:\n{read_text}"
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
fn non_document_surfaces_do_not_evaluate_document_projection() {
    for args in [
        vec!["--help"],
        vec!["version"],
        vec!["config", "inspect"],
        vec!["adapter", "list"],
        vec!["init"],
        vec!["doctor"],
    ] {
        parse_with_registry(args, &PanickingRegistry)
            .expect("non-document parsing must not inspect document declarations");
    }
}

#[test]
fn document_projection_conflict_reports_adapter_and_field_attribution() {
    let error = parse_with_registry(["outline", "--help"], &StaticConflictRegistry)
        .expect_err("generated/static conflict must fail before parsing");
    let error_id = error
        .diagnostic()
        .details()
        .to_value()
        .get("error_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned();

    assert!(
        error_id.contains("owner=adapter:docnav-markdown"),
        "{error_id}"
    );
    assert!(
        error_id.contains("docnav.adapters.docnav-markdown.options.project_config_conflict"),
        "{error_id}"
    );
}

#[test]
fn unrelated_operation_projection_conflict_does_not_block_current_parse() {
    let parsed = parse_with_registry(
        ["read", "doc.md", "--ref", "doc:full"],
        &StaticConflictRegistry,
    )
    .expect("outline-only declaration conflict must not block read parsing");

    assert!(matches!(parsed.command, CliCommand::Document(_)));
    parse_with_registry(["outline", "doc.md"], &StaticConflictRegistry)
        .expect_err("the conflicted outline projection remains blocking for outline");
}

#[test]
fn generated_presence_boolean_switch_is_retained_and_captured() {
    let parsed = parse_with_registry(
        ["outline", "doc.md", "--show-details"],
        &PresenceSwitchRegistry,
    )
    .expect("generated presence switch parses");
    let CliCommand::Document(command) = parsed.command else {
        panic!("expected document command");
    };
    let candidate = command
        .cli_source
        .candidates()
        .iter()
        .find(|candidate| {
            candidate.field().as_str() == "docnav.adapters.docnav-markdown.options.show_details"
        })
        .expect("presence switch candidate");

    assert_eq!(
        candidate.input(),
        &cli_config_resolution::CandidateInput::Value(true.into())
    );
}

struct PanickingRegistry;

impl NavigationAdapterRegistry for PanickingRegistry {
    fn adapters(&self) -> Vec<NavigationAdapterRef<'_>> {
        panic!("document projection was evaluated")
    }
}

struct StaticConflictRegistry;

struct PresenceSwitchRegistry;

impl NavigationAdapterRegistry for StaticConflictRegistry {
    fn adapters(&self) -> Vec<NavigationAdapterRef<'_>> {
        let base = docnav_markdown::markdown_adapter_definition();
        let conflict = AdapterOptionSpec::builder(
            "docnav.adapters.docnav-markdown.options.project_config_conflict",
        )
        .owner("docnav-markdown")
        .operations(&[Operation::Outline])
        .path(["options", "project_config_conflict"])
        .process(
            "cli",
            ProcessStrategy::cli_flag("--project-config").cli_metadata(
                CliProcessingMetadata::new()
                    .help("Conflict with core project config path")
                    .value_name("value"),
            ),
        )
        .validation(FieldValidation::string())
        .build();
        let definition = AdapterDefinition::builder(base.id())
            .adapter(base.adapter())
            .manifest(base.manifest().clone())
            .required_operation_handlers()
            .native_option(conflict)
            .build()
            .expect("conflict is valid at the adapter declaration boundary");
        vec![NavigationAdapterRef::new(definition)]
    }
}

impl NavigationAdapterRegistry for PresenceSwitchRegistry {
    fn adapters(&self) -> Vec<NavigationAdapterRef<'_>> {
        let base = docnav_markdown::markdown_adapter_definition();
        let switch =
            AdapterOptionSpec::builder("docnav.adapters.docnav-markdown.options.show_details")
                .owner("docnav-markdown")
                .operations(&[Operation::Outline])
                .path(["options", "show_details"])
                .process(
                    "cli",
                    ProcessStrategy::cli_flag("--show-details").cli_metadata(
                        CliProcessingMetadata::new()
                            .help("Show outline details")
                            .boolean_encoding(CliBooleanEncoding::PresenceMeansTrue),
                    ),
                )
                .validation(FieldValidation::boolean())
                .build();
        let definition = AdapterDefinition::builder(base.id())
            .adapter(base.adapter())
            .manifest(base.manifest().clone())
            .required_operation_handlers()
            .native_option(switch)
            .build()
            .expect("presence-switch adapter definition");
        vec![NavigationAdapterRef::new(definition)]
    }
}
