use clap::builder::NonEmptyStringValueParser;
use clap::{Arg, Command};
use docnav_protocol::Operation;

use super::super::native_options::{NativeOptionDefault, NativeOptionSpec, NativeOptionValueSpec};
use super::super::output::DirectOutputMode;

// 直接 CLI flag 来自 CLI/adapter-contract 主规范；只在 SDK direct CLI 边界解析使用。
pub(in crate::direct::args) mod flags {
    pub(in crate::direct::args) const LIMIT_CHARS: &str = "--limit-chars";
    pub(in crate::direct::args) const OUTPUT: &str = "--output";
    pub(in crate::direct::args) const PAGE: &str = "--page";
    pub(in crate::direct::args) const QUERY: &str = "--query";
    pub(in crate::direct::args) const REF: &str = "--ref";
}

pub(in crate::direct::args) mod arg_ids {
    pub(in crate::direct::args) const LIMIT_CHARS: &str = "limit_chars";
    pub(in crate::direct::args) const OUTPUT: &str = "output";
    pub(in crate::direct::args) const PAGE: &str = "page";
    pub(in crate::direct::args) const PATH: &str = "path";
    pub(in crate::direct::args) const QUERY: &str = "query";
    pub(in crate::direct::args) const REF: &str = "ref";
}

pub(in crate::direct) mod command_names {
    pub(crate) const FIND: &str = "find";
    pub(crate) const INFO: &str = "info";
    pub(crate) const INVOKE: &str = "invoke";
    pub(crate) const MANIFEST: &str = "manifest";
    pub(crate) const OUTLINE: &str = "outline";
    pub(crate) const PROBE: &str = "probe";
    pub(crate) const READ: &str = "read";
}

mod defaults {
    pub(super) const LIMIT_CHARS: &str = "6000";
    pub(super) const LIMIT_CHARS_VALUE: u32 = 6000;
    pub(super) const OUTPUT: &str = super::output_values::READABLE_VIEW;
    pub(super) const PAGE: &str = "1";
    pub(super) const PROTOCOL_OUTPUT: &str = super::output_values::PROTOCOL_JSON;
}

// 直接 CLI 输出模式字符串来自 CLI 主规范；protocol 层不复用这些阅读输出标签。
mod output_values {
    pub(super) const PROTOCOL_JSON: &str = "protocol-json";
    pub(super) const READABLE_JSON: &str = "readable-json";
    pub(super) const READABLE_VIEW: &str = "readable-view";
}

// 这些命令标签只用于直接 CLI warning reason，不参与 protocol operation 枚举。
pub(in crate::direct::args) mod command_labels {
    pub(in crate::direct::args) const MANIFEST: &str = "manifest";
    pub(in crate::direct::args) const PROBE: &str = "probe";
}

// 输入错误文案属于直接 CLI 边界诊断，不进入 protocol schema。
pub(in crate::direct::args) mod input_errors {
    pub(in crate::direct::args) const PROTOCOL_OUTPUT_ONLY: &str =
        "only --output protocol-json is supported for this command";
}

pub(in crate::direct) fn direct_cli_command(
    program_name: &'static str,
    native_options: &[NativeOptionSpec],
    default_limit_chars: u32,
) -> Command {
    Command::new(program_name)
        .about("Docnav adapter direct CLI")
        .disable_help_subcommand(true)
        .subcommand(protocol_only_command(
            command_names::MANIFEST,
            "Emit adapter manifest",
        ))
        .subcommand(probe_command(native_options))
        .subcommand(
            Command::new(command_names::INVOKE).about("Read one protocol request from stdin"),
        )
        .subcommand(operation_command(
            Operation::Outline,
            "Return compact document outline entries",
            native_options,
            default_limit_chars,
        ))
        .subcommand(operation_command(
            Operation::Read,
            "Read a document region by adapter ref",
            native_options,
            default_limit_chars,
        ))
        .subcommand(operation_command(
            Operation::Find,
            "Find matching document regions",
            native_options,
            default_limit_chars,
        ))
        .subcommand(operation_command(
            Operation::Info,
            "Return adapter document summary",
            native_options,
            default_limit_chars,
        ))
}

pub(in crate::direct::args) fn protocol_only_command(
    name: &'static str,
    about: &'static str,
) -> Command {
    Command::new(name).about(about).arg(protocol_output_arg())
}

pub(in crate::direct::args) fn probe_command(native_options: &[NativeOptionSpec]) -> Command {
    let mut command = Command::new(command_names::PROBE)
        .about("Probe document format support")
        .arg(path_arg())
        .arg(protocol_output_arg());
    for spec in native_options {
        command = command.arg(native_arg(spec));
    }
    command
}

pub(in crate::direct::args) fn operation_command(
    operation: Operation,
    about: &'static str,
    native_options: &[NativeOptionSpec],
    default_limit_chars: u32,
) -> Command {
    let mut command = Command::new(operation_name(operation))
        .about(about)
        .arg(path_arg())
        .arg(output_arg());

    if operation != Operation::Info {
        command = command
            .arg(page_arg())
            .arg(limit_chars_arg(default_limit_chars));
    }
    if operation == Operation::Read {
        command = command.arg(ref_arg());
    }
    if operation == Operation::Find {
        command = command.arg(query_arg());
    }
    for spec in native_options
        .iter()
        .filter(|spec| spec.supports(operation))
    {
        command = command.arg(native_arg(spec));
    }
    command
}

fn path_arg() -> Arg {
    Arg::new(arg_ids::PATH)
        .value_name("path")
        .required(true)
        .value_parser(NonEmptyStringValueParser::new())
}

fn page_arg() -> Arg {
    Arg::new(arg_ids::PAGE)
        .long("page")
        .value_name("positive integer")
        .num_args(1)
        .allow_hyphen_values(true)
        .default_value(defaults::PAGE)
        .value_parser(clap::value_parser!(u32))
}

fn limit_chars_arg(default_limit_chars: u32) -> Arg {
    let arg = Arg::new(arg_ids::LIMIT_CHARS)
        .long("limit-chars")
        .value_name("positive integer")
        .num_args(1)
        .allow_hyphen_values(true)
        .value_parser(clap::value_parser!(u32));
    if default_limit_chars == defaults::LIMIT_CHARS_VALUE {
        arg.default_value(defaults::LIMIT_CHARS)
    } else {
        arg.help("positive integer; default: adapter configured value")
    }
}

fn output_arg() -> Arg {
    Arg::new(arg_ids::OUTPUT)
        .long("output")
        .value_name("readable-view|readable-json|protocol-json")
        .num_args(1)
        .allow_hyphen_values(true)
        .default_value(defaults::OUTPUT)
        .value_parser([
            output_values::READABLE_VIEW,
            output_values::READABLE_JSON,
            output_values::PROTOCOL_JSON,
        ])
}

fn protocol_output_arg() -> Arg {
    Arg::new(arg_ids::OUTPUT)
        .long("output")
        .value_name("protocol-json")
        .num_args(1)
        .allow_hyphen_values(true)
        .default_value(defaults::PROTOCOL_OUTPUT)
        .value_parser([output_values::PROTOCOL_JSON])
}

fn ref_arg() -> Arg {
    Arg::new(arg_ids::REF)
        .long("ref")
        .value_name("ref")
        .num_args(1)
        .required(true)
        .allow_hyphen_values(true)
        .value_parser(NonEmptyStringValueParser::new())
}

fn query_arg() -> Arg {
    Arg::new(arg_ids::QUERY)
        .long("query")
        .value_name("text")
        .num_args(1)
        .required(true)
        .allow_hyphen_values(true)
        .value_parser(NonEmptyStringValueParser::new())
}

fn native_arg(spec: &NativeOptionSpec) -> Arg {
    Arg::new(spec.option_key)
        .long(strip_long_prefix(spec.flag))
        .value_name(native_value_name(spec))
        .num_args(1)
        .allow_hyphen_values(true)
        .help(native_help(spec))
        .value_parser(NonEmptyStringValueParser::new())
}

fn native_value_name(spec: &NativeOptionSpec) -> &'static str {
    match spec.value {
        NativeOptionValueSpec::IntegerRange { .. } => "integer",
    }
}

fn native_help(spec: &NativeOptionSpec) -> String {
    let range = match spec.value {
        NativeOptionValueSpec::IntegerRange { min, max } => {
            format!("integer from {min} to {max}")
        }
    };
    match spec.default {
        Some(NativeOptionDefault::Integer(value)) => {
            format!("{range}; default: {value}")
        }
        None => range,
    }
}

fn strip_long_prefix(flag: &'static str) -> &'static str {
    flag.strip_prefix("--").unwrap_or(flag)
}

fn operation_name(operation: Operation) -> &'static str {
    match operation {
        Operation::Outline => command_names::OUTLINE,
        Operation::Read => command_names::READ,
        Operation::Find => command_names::FIND,
        Operation::Info => command_names::INFO,
    }
}

pub(in crate::direct::args) fn operation_about(operation: Operation) -> &'static str {
    match operation {
        Operation::Outline => "Return compact document outline entries",
        Operation::Read => "Read a document region by adapter ref",
        Operation::Find => "Find matching document regions",
        Operation::Info => "Return adapter document summary",
    }
}

pub(in crate::direct::args) fn parse_protocol_output(value: &str) -> Result<(), String> {
    if parse_output(value)? == DirectOutputMode::ProtocolJson {
        Ok(())
    } else {
        Err(input_errors::PROTOCOL_OUTPUT_ONLY.to_owned())
    }
}

pub(in crate::direct::args) fn parse_output(value: &str) -> Result<DirectOutputMode, String> {
    match value {
        output_values::READABLE_VIEW => Ok(DirectOutputMode::ReadableView),
        output_values::READABLE_JSON => Ok(DirectOutputMode::ReadableJson),
        output_values::PROTOCOL_JSON => Ok(DirectOutputMode::ProtocolJson),
        _ => Err(format!("invalid {} {value:?}", flags::OUTPUT)),
    }
}
