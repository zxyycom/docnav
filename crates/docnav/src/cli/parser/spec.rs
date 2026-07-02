use clap::builder::NonEmptyStringValueParser;
use clap::{Arg, ArgAction, Command};
use docnav_protocol::Operation;

use crate::registry;

pub(in crate::cli) mod command_names {
    pub(in crate::cli) const ADAPTER: &str = "adapter";
    pub(in crate::cli) const ADAPTER_LIST: &str = "list";
    pub(in crate::cli) const CONFIG: &str = "config";
    pub(in crate::cli) const CONFIG_GET: &str = "get";
    pub(in crate::cli) const CONFIG_LIST: &str = "list";
    pub(in crate::cli) const CONFIG_SET: &str = "set";
    pub(in crate::cli) const CONFIG_UNSET: &str = "unset";
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
    pub(in crate::cli) const KEY: &str = "key";
    pub(in crate::cli) const LIMIT: &str = "limit";
    pub(in crate::cli) const OPERATION: &str = "operation";
    pub(in crate::cli) const OUTPUT: &str = "output";
    pub(in crate::cli) const PAGE: &str = "page";
    pub(in crate::cli) const PAGINATION: &str = "pagination";
    pub(in crate::cli) const PATH: &str = "path";
    pub(in crate::cli) const QUERY: &str = "query";
    pub(in crate::cli) const REF: &str = "ref";
    pub(in crate::cli) const USER: &str = "user";
    pub(in crate::cli) const VALUE: &str = "value";
}

pub(in crate::cli) mod defaults {
    pub(in crate::cli) const LIMIT: &str = crate::standard_parameters::DEFAULT_LIMIT_TEXT;
    pub(in crate::cli) const OUTPUT: &str = crate::standard_parameters::DEFAULT_OUTPUT_TEXT;
    pub(in crate::cli) const PAGE: &str = crate::standard_parameters::DEFAULT_PAGE_TEXT;
    pub(in crate::cli) const PAGINATION: &str = "enabled";
}

pub(in crate::cli) mod output_values {
    pub(in crate::cli) const PROTOCOL_JSON: &str = "protocol-json";
    pub(in crate::cli) const READABLE_JSON: &str = "readable-json";
    pub(in crate::cli) const READABLE_VIEW: &str = "readable-view";
}

pub(in crate::cli) mod operation_values {
    pub(in crate::cli) const FIND: &str = "find";
    pub(in crate::cli) const INFO: &str = "info";
    pub(in crate::cli) const OUTLINE: &str = "outline";
    pub(in crate::cli) const READ: &str = "read";
}

pub(in crate::cli) mod pagination_values {
    pub(in crate::cli) const DISABLED: &str = "disabled";
    pub(in crate::cli) const ENABLED: &str = "enabled";
}

pub(in crate::cli) fn cli_command() -> Command {
    Command::new("docnav")
        .about("Structured document navigation CLI")
        .disable_help_subcommand(true)
        .subcommand(document_clap_command(Operation::Outline))
        .subcommand(document_clap_command(Operation::Read))
        .subcommand(document_clap_command(Operation::Find))
        .subcommand(document_clap_command(Operation::Info))
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

pub(in crate::cli) fn document_clap_command(operation: Operation) -> Command {
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
    )
}

fn document_command(name: &'static str, about: &'static str) -> Command {
    Command::new(name)
        .about(about)
        .arg(path_arg())
        .arg(adapter_arg())
        .arg(output_arg())
}

fn paged_document_command(name: &'static str, about: &'static str) -> Command {
    document_command(name, about)
        .arg(pagination_arg())
        .arg(page_arg())
        .arg(limit_arg())
}

fn config_command() -> Command {
    Command::new(command_names::CONFIG)
        .about("Read and write docnav configuration")
        .subcommand(config_get_command())
        .subcommand(config_set_command())
        .subcommand(config_unset_command())
        .subcommand(config_list_command())
}

pub(in crate::cli) fn adapter_command() -> Command {
    Command::new(command_names::ADAPTER)
        .about("Inspect core release built-in adapters")
        .subcommand(
            Command::new(command_names::ADAPTER_LIST)
                .about("List adapters registered in the current core release"),
        )
}

pub(in crate::cli) fn config_get_command() -> Command {
    Command::new(command_names::CONFIG_GET)
        .about("Read an effective configuration key")
        .arg(key_arg())
        .arg(user_arg())
}

pub(in crate::cli) fn config_set_command() -> Command {
    Command::new(command_names::CONFIG_SET)
        .about("Set a project or user configuration key")
        .arg(key_arg())
        .arg(positional_value_arg(arg_ids::VALUE, "value"))
        .arg(user_arg())
}

pub(in crate::cli) fn config_unset_command() -> Command {
    Command::new(command_names::CONFIG_UNSET)
        .about("Remove a project or user configuration key")
        .arg(key_arg())
        .arg(user_arg())
}

pub(in crate::cli) fn config_list_command() -> Command {
    Command::new(command_names::CONFIG_LIST)
        .about("List effective configuration")
        .arg(user_arg())
        .arg(value_arg(arg_ids::PATH, "path", "path"))
        .arg(operation_arg())
}

pub(in crate::cli) fn utility_clap_command(name: &'static str, about: &'static str) -> Command {
    Command::new(name).about(about)
}

fn path_arg() -> Arg {
    Arg::new(arg_ids::PATH)
        .value_name("path")
        .required(true)
        .value_parser(NonEmptyStringValueParser::new())
}

fn key_arg() -> Arg {
    positional_value_arg(arg_ids::KEY, "key")
}

fn adapter_arg() -> Arg {
    value_arg(arg_ids::ADAPTER, "adapter", "adapter-id")
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

fn add_native_option_args(mut command: Command, operation: Operation) -> Command {
    let mut arg_ids = Vec::new();
    for option in registry::native_options_for(operation) {
        let Some(arg_id) = option.cli_arg_id() else {
            continue;
        };
        if arg_ids.contains(&arg_id) {
            continue;
        }
        arg_ids.push(arg_id);
        command = command.arg(value_arg(arg_id, arg_id, "value"));
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

fn operation_arg() -> Arg {
    value_arg(arg_ids::OPERATION, "operation", "outline|read|find|info").value_parser([
        operation_values::OUTLINE,
        operation_values::READ,
        operation_values::FIND,
        operation_values::INFO,
    ])
}

fn user_arg() -> Arg {
    Arg::new(arg_ids::USER)
        .long("user")
        .action(ArgAction::SetTrue)
}

fn value_arg(id: &'static str, long: &'static str, value_name: &'static str) -> Arg {
    Arg::new(id)
        .long(long)
        .value_name(value_name)
        .num_args(1)
        .allow_hyphen_values(true)
        .value_parser(NonEmptyStringValueParser::new())
}

fn positional_value_arg(id: &'static str, value_name: &'static str) -> Arg {
    Arg::new(id)
        .value_name(value_name)
        .required(true)
        .value_parser(NonEmptyStringValueParser::new())
}
