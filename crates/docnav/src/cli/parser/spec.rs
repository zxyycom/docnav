use std::fmt;

use clap::builder::{NonEmptyStringValueParser, Str};
use clap::Id;
use clap::{Arg, ArgAction, Command};
use cli_config_resolution::ProcessingId;
use docnav_navigation::{document_adapter_routing_fields, DocumentParameterCatalog};
use docnav_protocol::Operation;
use docnav_typed_fields::{
    CliBooleanEncoding, DefaultMetadata, FieldIdentity, ProcessingLocator, ProcessingMetadataView,
    ValueKind,
};

use crate::error::{AppError, AppResult};
use crate::parameter_catalog::document_parameter_catalog;

pub(in crate::cli) mod command_names {
    pub(in crate::cli) const ADAPTER: &str = "adapter";
    pub(in crate::cli) const ADAPTER_LIST: &str = "list";
    pub(in crate::cli) const CONFIG: &str = "config";
    pub(in crate::cli) const CONFIG_INSPECT: &str = "inspect";
    pub(in crate::cli) const DOCTOR: &str = "doctor";
    pub(in crate::cli) const FIND: &str = "find";
    pub(in crate::cli) const INFO: &str = "info";
    pub(in crate::cli) const INIT: &str = "init";
    pub(in crate::cli) const OUTLINE: &str = "outline";
    pub(in crate::cli) const READ: &str = "read";
    pub(in crate::cli) const VERSION: &str = "version";
}

pub(in crate::cli) mod arg_ids {
    pub(in crate::cli) const INVOCATION_LOG: &str = "invocation-log";
    pub(in crate::cli) const INVOCATION_LOG_CONTENT_ROOT: &str = "invocation-log-content-root";
    pub(in crate::cli) const PATH: &str = "path";
    pub(in crate::cli) const PROJECT_CONFIG: &str = "project-config";
    pub(in crate::cli) const QUERY: &str = "query";
    pub(in crate::cli) const REF: &str = "ref";
    pub(in crate::cli) const USER_CONFIG: &str = "user-config";
}

#[derive(Clone, Copy)]
enum ConfigPathSupport {
    None,
    ProjectOnly,
    ProjectAndUser,
}

pub(in crate::cli) fn cli_command() -> Command {
    Command::new("docnav")
        .about("Structured document navigation CLI")
        .disable_help_subcommand(true)
        .subcommand(static_document_clap_command(Operation::Outline))
        .subcommand(static_document_clap_command(Operation::Read))
        .subcommand(static_document_clap_command(Operation::Find))
        .subcommand(static_document_clap_command(Operation::Info))
        .subcommand(adapter_command())
        .subcommand(config_command())
        .subcommand(utility_clap_command(
            command_names::INIT,
            "Initialize .docnav project configuration",
        ))
        .subcommand(utility_clap_command(
            command_names::DOCTOR,
            "Check Docnav project and adapter health",
        ))
        .subcommand(utility_clap_command(
            command_names::VERSION,
            "Print docnav version",
        ))
}

pub(in crate::cli) fn is_known_root_command(command: &str) -> bool {
    cli_command().find_subcommand(command).is_some()
}

pub(in crate::cli) struct DocumentCliSpec {
    pub(in crate::cli) command: Command,
    pub(in crate::cli) routing_fields: docnav_typed_fields::FieldDefSet,
    pub(in crate::cli) parameters: DocumentParameterCatalog,
}

pub(in crate::cli) fn document_clap_command(operation: Operation) -> AppResult<DocumentCliSpec> {
    let routing_fields = document_adapter_routing_fields().map_err(|error| {
        AppError::internal(format!(
            "document-routing-field-set-build-failed:{}:{error}",
            operation.as_str()
        ))
    })?;
    let parameters = document_parameter_catalog().map_err(|error| {
        AppError::internal(format!(
            "document-parameter-catalog-build-failed:{}:{error}",
            operation.as_str()
        ))
    })?;
    let processing_id = ProcessingId::new("cli").expect("document CLI processing id is valid");
    let mut command = static_document_clap_command(operation);
    for metadata in routing_fields.processing_metadata(&processing_id) {
        let argument = parameter_argument(&command, &metadata)
            .map_err(|error| document_projection_error(operation, "navigation", error))?;
        command = command.arg(argument);
    }
    let command = augment_parameter_command(command, &parameters, operation, &processing_id)
        .map_err(|error| document_projection_error(operation, "core-catalog", error))?;
    Ok(DocumentCliSpec {
        command,
        routing_fields,
        parameters,
    })
}

pub(in crate::cli) fn static_document_clap_command(operation: Operation) -> Command {
    match operation {
        Operation::Outline => document_command(
            command_names::OUTLINE,
            "Return compact document outline entries",
        ),
        Operation::Read => {
            document_command(command_names::READ, "Read a document region by adapter ref")
                .arg(ref_arg())
        }
        Operation::Find => {
            document_command(command_names::FIND, "Find matching document regions").arg(query_arg())
        }
        Operation::Info => document_command(command_names::INFO, "Return adapter document summary"),
    }
}

fn document_command(name: &'static str, about: &'static str) -> Command {
    with_config_path_args(
        Command::new(name)
            .about(about)
            .arg(path_arg())
            .arg(invocation_log_arg())
            .arg(invocation_log_content_root_arg()),
        ConfigPathSupport::ProjectAndUser,
    )
}

fn config_command() -> Command {
    Command::new(command_names::CONFIG)
        .about("Inspect docnav configuration sources")
        .subcommand(config_inspect_command())
}

pub(in crate::cli) fn adapter_command() -> Command {
    Command::new(command_names::ADAPTER)
        .about("Inspect core release built-in adapters")
        .subcommand(
            Command::new(command_names::ADAPTER_LIST)
                .about("List adapters registered in the current core release"),
        )
}

pub(in crate::cli) fn config_inspect_command() -> Command {
    with_config_path_args(
        Command::new(command_names::CONFIG_INSPECT).about("Inspect selected configuration sources"),
        ConfigPathSupport::ProjectAndUser,
    )
}

pub(in crate::cli) fn utility_clap_command(name: &'static str, about: &'static str) -> Command {
    let command = Command::new(name).about(about);
    let support = match name {
        command_names::INIT => ConfigPathSupport::ProjectOnly,
        command_names::DOCTOR => ConfigPathSupport::ProjectAndUser,
        _ => ConfigPathSupport::None,
    };
    with_config_path_args(command, support)
}

fn path_arg() -> Arg {
    Arg::new(arg_ids::PATH)
        .value_name("path")
        .required(true)
        .value_parser(NonEmptyStringValueParser::new())
}

fn invocation_log_arg() -> Arg {
    value_arg(arg_ids::INVOCATION_LOG, "invocation-log", "path")
}

fn invocation_log_content_root_arg() -> Arg {
    value_arg(
        arg_ids::INVOCATION_LOG_CONTENT_ROOT,
        "invocation-log-content-root",
        "path",
    )
}

fn project_config_arg() -> Arg {
    value_arg(arg_ids::PROJECT_CONFIG, "project-config", "path")
}

fn user_config_arg() -> Arg {
    value_arg(arg_ids::USER_CONFIG, "user-config", "path")
}

fn with_config_path_args(command: Command, support: ConfigPathSupport) -> Command {
    match support {
        ConfigPathSupport::None => command,
        ConfigPathSupport::ProjectOnly => command.arg(project_config_arg()),
        ConfigPathSupport::ProjectAndUser => {
            command.arg(project_config_arg()).arg(user_config_arg())
        }
    }
}

fn query_arg() -> Arg {
    value_arg(arg_ids::QUERY, "query", "text").required(true)
}

fn ref_arg() -> Arg {
    value_arg(arg_ids::REF, "ref", "ref").required(true)
}

fn value_arg(id: impl Into<Id>, long: impl Into<Str>, value_name: &'static str) -> Arg {
    Arg::new(id)
        .long(long)
        .value_name(value_name)
        .num_args(1)
        .value_parser(NonEmptyStringValueParser::new())
}

pub(in crate::cli) fn document_projection_error(
    operation: Operation,
    owner: &str,
    error: DocumentProjectionError,
) -> AppError {
    let field = error
        .field()
        .map_or("unknown", |identity| identity.as_str());
    AppError::internal(format!(
        "document-cli-projection-failed:{}:owner={owner}:field={field}:{error}",
        operation.as_str()
    ))
}

#[derive(Debug)]
pub(super) struct DocumentProjectionError {
    field: Option<FieldIdentity>,
    message: String,
}

impl DocumentProjectionError {
    pub(super) fn for_field(field: FieldIdentity, message: impl Into<String>) -> Self {
        Self {
            field: Some(field),
            message: message.into(),
        }
    }

    pub(super) fn source(error: cli_config_resolution::SourceError) -> Self {
        Self {
            field: None,
            message: error.to_string(),
        }
    }

    fn field(&self) -> Option<&FieldIdentity> {
        self.field.as_ref()
    }
}

impl fmt::Display for DocumentProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

fn augment_parameter_command(
    mut command: Command,
    catalog: &DocumentParameterCatalog,
    operation: Operation,
    processing_id: &ProcessingId,
) -> Result<Command, DocumentProjectionError> {
    for field in catalog.operation_fields(operation) {
        let Some(metadata) = field.processing_metadata(processing_id) else {
            continue;
        };
        let argument = parameter_argument(&command, &metadata)?;
        command = command.arg(argument);
    }
    Ok(command)
}

fn parameter_argument(
    command: &Command,
    metadata: &ProcessingMetadataView<'_>,
) -> Result<Arg, DocumentProjectionError> {
    let ProcessingLocator::CliFlag(flag) = &metadata.locator else {
        return Err(DocumentProjectionError::for_field(
            metadata.identity().clone(),
            format!(
                "field {} uses non-CLI processing locator {:?}",
                metadata.identity().as_str(),
                metadata.locator
            ),
        ));
    };
    let mut argument = Arg::new(metadata.identity().as_str().to_owned());
    if let Some(long) = valid_long_flag(flag) {
        if command
            .get_arguments()
            .any(|existing| existing.get_long() == Some(long))
        {
            return Err(DocumentProjectionError::for_field(
                metadata.identity().clone(),
                format!(
                    "CLI flag {flag} for field {} conflicts with another clap argument",
                    metadata.identity().as_str()
                ),
            ));
        }
        argument = argument.long(long.to_owned());
    } else {
        return Err(DocumentProjectionError::for_field(
            metadata.identity().clone(),
            format!(
                "field {} has invalid CLI flag {flag}",
                metadata.identity().as_str()
            ),
        ));
    }

    let cli = metadata.cli.as_ref().cloned().unwrap_or_default();
    if let Some(help) = parameter_help(metadata) {
        argument = argument.help(help);
    }
    if let Some(value_name) = cli.value_name {
        argument = argument.value_name(value_name);
    }
    match (metadata.value_kind(), &cli.boolean_encoding) {
        (
            ValueKind::Boolean,
            Some(CliBooleanEncoding::Explicit {
                true_token: Some(_),
                false_token: Some(_),
            }),
        ) => Ok(argument.action(ArgAction::Set).num_args(1)),
        (ValueKind::String, _) => Ok(argument.action(ArgAction::Set).num_args(1)),
        (ValueKind::Integer, _) => Ok(argument
            .action(ArgAction::Set)
            .num_args(1)
            .allow_negative_numbers(true)),
        _ => Err(DocumentProjectionError::for_field(
            metadata.identity().clone(),
            format!(
                "field {} has unsupported CLI value kind {:?}",
                metadata.identity().as_str(),
                metadata.value_kind()
            ),
        )),
    }
}

fn valid_long_flag(flag: &str) -> Option<&str> {
    let long = flag.strip_prefix("--")?;
    (!long.is_empty()
        && !long.starts_with('-')
        && long
            .chars()
            .all(|character| !character.is_whitespace() && character != '='))
    .then_some(long)
}

fn parameter_help(metadata: &ProcessingMetadataView<'_>) -> Option<String> {
    let mut canonical_facts = Vec::new();
    if let Some(accepted_values) = &metadata.constraints().enum_values {
        canonical_facts.push(format!(
            "possible values: {}",
            accepted_values
                .iter()
                .map(display_json_value)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if let DefaultMetadata::Static(value) = metadata.default() {
        canonical_facts.push(format!("default: {}", display_json_value(value)));
    }
    let help = metadata
        .cli
        .as_ref()
        .and_then(|metadata| metadata.help.as_deref());
    match (help, canonical_facts.is_empty()) {
        (Some(help), true) => Some(help.to_owned()),
        (Some(help), false) => Some(format!("{help} [{}]", canonical_facts.join("; "))),
        (None, false) => Some(format!("[{}]", canonical_facts.join("; "))),
        (None, true) => None,
    }
}

fn display_json_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),
        value => value.to_string(),
    }
}
