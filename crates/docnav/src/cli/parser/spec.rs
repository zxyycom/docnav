use clap::builder::{NonEmptyStringValueParser, Str};
use clap::Id;
use clap::{Arg, Command};
use docnav_protocol::Operation;

use super::ParserContext;

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
    pub(in crate::cli) const ADAPTER: &str = "adapter";
    pub(in crate::cli) const INVOCATION_LOG: &str = "invocation-log";
    pub(in crate::cli) const INVOCATION_LOG_CONTENT_ROOT: &str = "invocation-log-content-root";
    pub(in crate::cli) const LIMIT: &str = "limit";
    pub(in crate::cli) const OUTPUT: &str = "output";
    pub(in crate::cli) const PAGE: &str = "page";
    pub(in crate::cli) const PAGINATION: &str = "pagination";
    pub(in crate::cli) const PATH: &str = "path";
    pub(in crate::cli) const PROJECT_CONFIG: &str = "project-config";
    pub(in crate::cli) const QUERY: &str = "query";
    pub(in crate::cli) const REF: &str = "ref";
    pub(in crate::cli) const USER_CONFIG: &str = "user-config";
}

pub(in crate::cli) mod defaults {
    pub(in crate::cli) const LIMIT: &str = crate::navigation_defaults::DEFAULT_LIMIT_TEXT;
    pub(in crate::cli) const OUTPUT: &str = crate::navigation_defaults::DEFAULT_OUTPUT_TEXT;
    pub(in crate::cli) const PAGE: &str = crate::navigation_defaults::DEFAULT_PAGE_TEXT;
    pub(in crate::cli) const PAGINATION: &str = "enabled";
}

pub(in crate::cli) mod output_values {
    pub(in crate::cli) const PROTOCOL_JSON: &str = "protocol-json";
    pub(in crate::cli) const READABLE_JSON: &str = "readable-json";
    pub(in crate::cli) const READABLE_VIEW: &str = "readable-view";
}

pub(in crate::cli) mod pagination_values {
    pub(in crate::cli) const DISABLED: &str = "disabled";
    pub(in crate::cli) const ENABLED: &str = "enabled";
}

#[derive(Clone, Copy)]
enum ConfigPathSupport {
    None,
    ProjectOnly,
    ProjectAndUser,
}

pub(in crate::cli) fn cli_command(context: &ParserContext) -> Command {
    Command::new("docnav")
        .about("Structured document navigation CLI")
        .disable_help_subcommand(true)
        .subcommand(document_clap_command(Operation::Outline, context))
        .subcommand(document_clap_command(Operation::Read, context))
        .subcommand(document_clap_command(Operation::Find, context))
        .subcommand(document_clap_command(Operation::Info, context))
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

pub(in crate::cli) fn is_known_root_command(command: &str, context: &ParserContext) -> bool {
    cli_command(context).find_subcommand(command).is_some()
}

pub(in crate::cli) fn document_clap_command(
    operation: Operation,
    context: &ParserContext,
) -> Command {
    add_native_option_args(
        match operation {
            Operation::Outline => paged_document_command(
                command_names::OUTLINE,
                "Return compact document outline entries",
            ),
            Operation::Read => {
                paged_document_command(command_names::READ, "Read a document region by adapter ref")
                    .arg(ref_arg())
            }
            Operation::Find => {
                paged_document_command(command_names::FIND, "Find matching document regions")
                    .arg(query_arg())
            }
            Operation::Info => {
                document_command(command_names::INFO, "Return adapter document summary")
            }
        },
        operation,
        context,
    )
}

fn document_command(name: &'static str, about: &'static str) -> Command {
    with_config_path_args(
        Command::new(name)
            .about(about)
            .arg(path_arg())
            .arg(adapter_arg())
            .arg(invocation_log_arg())
            .arg(invocation_log_content_root_arg())
            .arg(output_arg()),
        ConfigPathSupport::ProjectAndUser,
    )
}

fn paged_document_command(name: &'static str, about: &'static str) -> Command {
    document_command(name, about)
        .arg(pagination_arg())
        .arg(page_arg())
        .arg(limit_arg())
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

fn adapter_arg() -> Arg {
    value_arg(arg_ids::ADAPTER, "adapter", "adapter-id")
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

fn page_arg() -> Arg {
    value_arg(arg_ids::PAGE, "page", "positive integer")
        .default_value(defaults::PAGE)
        .value_parser(clap::value_parser!(u32))
}

fn limit_arg() -> Arg {
    value_arg(arg_ids::LIMIT, "limit", "positive integer")
        .default_value(defaults::LIMIT)
        .value_parser(clap::value_parser!(u32))
}

fn add_native_option_args(
    mut command: Command,
    operation: Operation,
    context: &ParserContext,
) -> Command {
    for arg_id in context.native_options().arg_ids_for_operation(operation) {
        command = command.arg(value_arg(*arg_id, *arg_id, "value"));
    }
    command
}

fn pagination_arg() -> Arg {
    value_arg(arg_ids::PAGINATION, "pagination", "enabled|disabled")
        .default_value(defaults::PAGINATION)
        .value_parser([pagination_values::ENABLED, pagination_values::DISABLED])
}

fn output_arg() -> Arg {
    value_arg(
        arg_ids::OUTPUT,
        "output",
        "readable-view|readable-json|protocol-json",
    )
    .default_value(defaults::OUTPUT)
    .value_parser([
        output_values::READABLE_VIEW,
        output_values::READABLE_JSON,
        output_values::PROTOCOL_JSON,
    ])
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
        .allow_hyphen_values(true)
        .value_parser(NonEmptyStringValueParser::new())
}
