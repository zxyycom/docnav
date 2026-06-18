use clap::builder::NonEmptyStringValueParser;
use clap::{Arg, Command};
use docnav_cli_args::{
    scan_loose_args, IgnoredArg, KnownValueFlag as LooseKnownValueFlag, LooseArgScan, MissingValue,
};
use docnav_protocol::{positive_result, Operation, Options, PositiveInteger};
use serde_json::Value;

use super::native_options::{NativeOptionDefault, NativeOptionSpec};
use super::output::DirectOutputMode;
use super::warnings::DirectCliWarning;

// 直接 CLI flag 来自 CLI/adapter-contract 主规范；只在 SDK direct CLI 边界解析使用。
mod flags {
    pub(super) const LIMIT_CHARS: &str = "--limit-chars";
    pub(super) const OUTPUT: &str = "--output";
    pub(super) const PAGE: &str = "--page";
    pub(super) const QUERY: &str = "--query";
    pub(super) const REF: &str = "--ref";
}

mod arg_ids {
    pub(super) const LIMIT_CHARS: &str = "limit_chars";
    pub(super) const OUTPUT: &str = "output";
    pub(super) const PAGE: &str = "page";
    pub(super) const PATH: &str = "path";
    pub(super) const QUERY: &str = "query";
    pub(super) const REF: &str = "ref";
}

pub(super) mod command_names {
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
mod command_labels {
    pub(super) const MANIFEST: &str = "manifest";
    pub(super) const PROBE: &str = "probe";
}

// 输入错误文案属于直接 CLI 边界诊断，不进入 protocol schema。
mod input_errors {
    pub(super) const PROTOCOL_OUTPUT_ONLY: &str =
        "only --output protocol-json is supported for this command";
}

#[derive(Clone, Debug)]
pub(super) struct DirectOperationOptions {
    pub(super) path: String,
    pub(super) page: PositiveInteger,
    pub(super) limit_chars: PositiveInteger,
    pub(super) output: DirectOutputMode,
    pub(super) ref_id: Option<String>,
    pub(super) query: Option<String>,
    pub(super) warnings: Vec<DirectCliWarning>,
    native_options: Options,
}

impl DirectOperationOptions {
    pub(super) fn protocol_options(&self) -> Option<Options> {
        if self.native_options.is_empty() {
            None
        } else {
            Some(self.native_options.clone())
        }
    }
}

pub(super) struct DirectProbeOptions {
    pub(super) path: String,
    pub(super) warnings: Vec<DirectCliWarning>,
}

pub(super) fn direct_cli_command(
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

pub(super) fn parse_protocol_only_options(
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<Vec<DirectCliWarning>, String> {
    let LooseArgs {
        clap_args,
        warnings,
    } = collect_protocol_only_args(command_labels::MANIFEST, args, native_options)?;
    protocol_only_command(command_names::MANIFEST, "Emit adapter manifest")
        .try_get_matches_from(clap_argv(command_names::MANIFEST, clap_args))
        .map_err(|_| protocol_only_parse_error(args))?;
    Ok(warnings)
}

pub(super) fn parse_probe(
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<DirectProbeOptions, String> {
    let LooseArgs {
        clap_args,
        warnings,
    } = collect_probe_args(args, native_options)?;
    let matches = probe_command(native_options)
        .try_get_matches_from(clap_argv(command_names::PROBE, clap_args))
        .map_err(|_| probe_parse_error(args))?;
    let path = required_string(&matches, arg_ids::PATH, "probe requires <path>")?;

    Ok(DirectProbeOptions { path, warnings })
}

pub(super) fn parse_operation_options(
    operation: Operation,
    args: &[String],
    default_limit_chars: u32,
    native_options: &[NativeOptionSpec],
) -> Result<DirectOperationOptions, String> {
    let LooseArgs {
        clap_args,
        warnings,
    } = collect_operation_args(operation, args, native_options)?;
    let matches = operation_command(
        operation,
        operation_about(operation),
        native_options,
        default_limit_chars,
    )
    .try_get_matches_from(clap_argv(operation.as_str(), clap_args))
    .map_err(|_| operation_parse_error(operation, args, native_options))?;

    let path = required_string(
        &matches,
        arg_ids::PATH,
        &format!("{operation} requires <path>"),
    )?;
    let page = if operation == Operation::Info {
        positive_result(1).expect("static positive integer")
    } else {
        let raw = matches.get_one::<u32>(arg_ids::PAGE).copied().unwrap_or(1);
        positive_from_u32(raw, flags::PAGE)?
    };
    let limit_chars = if operation == Operation::Info {
        positive_result(default_limit_chars).expect("static positive integer")
    } else {
        let raw = matches
            .get_one::<u32>(arg_ids::LIMIT_CHARS)
            .copied()
            .unwrap_or(default_limit_chars);
        positive_from_u32(raw, flags::LIMIT_CHARS)?
    };
    let output = parse_output(&required_string(
        &matches,
        arg_ids::OUTPUT,
        "missing output mode",
    )?)?;
    let ref_id = if operation == Operation::Read {
        Some(required_string(
            &matches,
            arg_ids::REF,
            "read requires --ref <ref>",
        )?)
    } else {
        None
    };
    let query = if operation == Operation::Find {
        Some(required_string(
            &matches,
            arg_ids::QUERY,
            "find requires --query <text>",
        )?)
    } else {
        None
    };
    let native_options = parsed_native_options(operation, &matches, native_options)?;

    Ok(DirectOperationOptions {
        path,
        page,
        limit_chars,
        output,
        ref_id,
        query,
        warnings,
        native_options,
    })
}

struct LooseArgs {
    clap_args: Vec<String>,
    warnings: Vec<DirectCliWarning>,
}

#[derive(Clone, Copy)]
enum LooseArgContext<'a> {
    ProtocolOnly { command: &'a str },
    Probe,
    Operation(Operation),
}

impl LooseArgContext<'_> {
    fn accepts_path(self) -> bool {
        !matches!(self, Self::ProtocolOnly { .. })
    }

    fn command_label(&self) -> &str {
        match self {
            Self::ProtocolOnly { command } => command,
            Self::Probe => command_labels::PROBE,
            Self::Operation(operation) => operation.as_str(),
        }
    }

    fn uses_flag(self, flag: KnownValueFlag<'_>) -> bool {
        match self {
            Self::ProtocolOnly { .. } | Self::Probe => matches!(flag, KnownValueFlag::Output),
            Self::Operation(operation) => operation_uses_flag(operation, flag),
        }
    }
}

#[derive(Clone, Copy)]
enum KnownValueFlag<'a> {
    Page,
    LimitChars,
    Ref,
    Query,
    Output,
    Native(&'a NativeOptionSpec),
}

fn known_value_flag<'a>(
    token: &str,
    native_options: &'a [NativeOptionSpec],
) -> Option<KnownValueFlag<'a>> {
    let (flag, _value) = split_equals(token);
    match flag {
        flags::PAGE => Some(KnownValueFlag::Page),
        flags::LIMIT_CHARS => Some(KnownValueFlag::LimitChars),
        flags::REF => Some(KnownValueFlag::Ref),
        flags::QUERY => Some(KnownValueFlag::Query),
        flags::OUTPUT => Some(KnownValueFlag::Output),
        _ => native_options
            .iter()
            .find(|spec| spec.flag == token)
            .map(KnownValueFlag::Native),
    }
}

fn operation_uses_flag(operation: Operation, flag: KnownValueFlag<'_>) -> bool {
    match flag {
        KnownValueFlag::Page | KnownValueFlag::LimitChars => operation != Operation::Info,
        KnownValueFlag::Ref => operation == Operation::Read,
        KnownValueFlag::Query => operation == Operation::Find,
        KnownValueFlag::Output => true,
        KnownValueFlag::Native(spec) => spec.supports(operation),
    }
}

fn default_native_options(operation: Operation, specs: &[NativeOptionSpec]) -> Options {
    let mut options = Options::new();
    for spec in specs.iter().filter(|spec| spec.supports(operation)) {
        let Some(default) = spec.default else {
            continue;
        };
        let value = match default {
            NativeOptionDefault::Integer(value) => Value::from(value),
        };
        options.insert(spec.option_key.to_owned(), value);
    }
    options
}

fn parse_protocol_output(value: &str) -> Result<(), String> {
    if parse_output(value)? == DirectOutputMode::ProtocolJson {
        Ok(())
    } else {
        Err(input_errors::PROTOCOL_OUTPUT_ONLY.to_owned())
    }
}

fn parse_output(value: &str) -> Result<DirectOutputMode, String> {
    match value {
        output_values::READABLE_VIEW => Ok(DirectOutputMode::ReadableView),
        output_values::READABLE_JSON => Ok(DirectOutputMode::ReadableJson),
        output_values::PROTOCOL_JSON => Ok(DirectOutputMode::ProtocolJson),
        _ => Err(format!("invalid {} {value:?}", flags::OUTPUT)),
    }
}

fn positive_from_u32(value: u32, flag: &str) -> Result<PositiveInteger, String> {
    positive_result(value).map_err(|_| format!("{flag} must be a positive integer"))
}

fn is_flag(value: &str) -> bool {
    value.starts_with("--")
}

fn split_equals(token: &str) -> (&str, Option<&str>) {
    token
        .split_once('=')
        .map_or((token, None), |(flag, value)| (flag, Some(value)))
}

fn clap_argv(command: &str, args: Vec<String>) -> Vec<String> {
    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push(command.to_owned());
    argv.extend(args);
    argv
}

fn collect_protocol_only_args(
    command: &str,
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<LooseArgs, String> {
    let context = LooseArgContext::ProtocolOnly { command };
    collect_loose_args(context, args, native_options)
}

fn collect_probe_args(
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<LooseArgs, String> {
    collect_loose_args(LooseArgContext::Probe, args, native_options)
}

fn collect_operation_args(
    operation: Operation,
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<LooseArgs, String> {
    collect_loose_args(LooseArgContext::Operation(operation), args, native_options)
}

fn collect_loose_args(
    context: LooseArgContext,
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<LooseArgs, String> {
    let known_value_flags = loose_known_value_flags(context, native_options);
    let positional_limit = usize::from(context.accepts_path());
    let scan = scan_loose_args(
        args,
        &LooseArgScan::new(
            context.command_label(),
            positional_limit,
            &known_value_flags,
        ),
    )
    .map_err(loose_missing_value_error)?;
    Ok(LooseArgs {
        clap_args: scan.retained_args,
        warnings: scan
            .ignored
            .into_iter()
            .map(warning_from_ignored_arg)
            .collect(),
    })
}

fn loose_known_value_flags<'a>(
    context: LooseArgContext<'a>,
    native_options: &'a [NativeOptionSpec],
) -> Vec<LooseKnownValueFlag<'a>> {
    let mut flags = vec![
        LooseKnownValueFlag {
            flag: flags::PAGE,
            used: context.uses_flag(KnownValueFlag::Page),
        },
        LooseKnownValueFlag {
            flag: flags::LIMIT_CHARS,
            used: context.uses_flag(KnownValueFlag::LimitChars),
        },
        LooseKnownValueFlag {
            flag: flags::REF,
            used: context.uses_flag(KnownValueFlag::Ref),
        },
        LooseKnownValueFlag {
            flag: flags::QUERY,
            used: context.uses_flag(KnownValueFlag::Query),
        },
        LooseKnownValueFlag {
            flag: flags::OUTPUT,
            used: context.uses_flag(KnownValueFlag::Output),
        },
    ];
    flags.extend(native_options.iter().map(|spec| LooseKnownValueFlag {
        flag: spec.flag,
        used: context.uses_flag(KnownValueFlag::Native(spec)),
    }));
    flags
}

fn warning_from_ignored_arg(ignored: IgnoredArg) -> DirectCliWarning {
    match ignored {
        IgnoredArg::UnknownFlag { token } => DirectCliWarning::unknown_flag(&token),
        IgnoredArg::ExtraPositional { token } => DirectCliWarning::extra_positional(&token),
        IgnoredArg::UnusedValueFlag {
            flag,
            value,
            command,
        } => DirectCliWarning::unused_operation_flag(&flag, value.as_deref(), &command),
    }
}

fn loose_missing_value_error(error: MissingValue) -> String {
    format!("{} requires a value", error.flag())
}

fn protocol_only_command(name: &'static str, about: &'static str) -> Command {
    Command::new(name).about(about).arg(protocol_output_arg())
}

fn probe_command(native_options: &[NativeOptionSpec]) -> Command {
    let mut command = Command::new(command_names::PROBE)
        .about("Probe document format support")
        .arg(path_arg())
        .arg(protocol_output_arg());
    for spec in native_options {
        command = command.arg(native_arg(spec));
    }
    command
}

fn operation_command(
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
        super::native_options::NativeOptionValueSpec::IntegerRange { .. } => "integer",
    }
}

fn native_help(spec: &NativeOptionSpec) -> String {
    let range = match spec.value {
        super::native_options::NativeOptionValueSpec::IntegerRange { min, max } => {
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

fn operation_about(operation: Operation) -> &'static str {
    match operation {
        Operation::Outline => "Return compact document outline entries",
        Operation::Read => "Read a document region by adapter ref",
        Operation::Find => "Find matching document regions",
        Operation::Info => "Return adapter document summary",
    }
}

fn required_string(
    matches: &clap::parser::ArgMatches,
    id: &str,
    message: &str,
) -> Result<String, String> {
    matches
        .get_one::<String>(id)
        .cloned()
        .ok_or_else(|| message.to_owned())
}

fn parsed_native_options(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
    specs: &[NativeOptionSpec],
) -> Result<Options, String> {
    let mut options = default_native_options(operation, specs);
    for spec in specs.iter().filter(|spec| spec.supports(operation)) {
        if let Some(value) = matches.get_one::<String>(spec.option_key) {
            options.insert(spec.option_key.to_owned(), spec.parse_value(value)?);
        }
    }
    Ok(options)
}

fn protocol_only_parse_error(args: &[String]) -> String {
    missing_value_error(args)
        .or_else(|| invalid_protocol_output_error(args))
        .unwrap_or_else(|| "invalid protocol-only command arguments".to_owned())
}

fn probe_parse_error(args: &[String]) -> String {
    if !has_path_candidate(args, &[]) {
        return "probe requires <path>".to_owned();
    }
    missing_value_error(args)
        .or_else(|| invalid_protocol_output_error(args))
        .unwrap_or_else(|| "invalid probe arguments".to_owned())
}

fn operation_parse_error(
    operation: Operation,
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> String {
    if !has_path_candidate(args, native_options) {
        return format!("{operation} requires <path>");
    }
    if operation == Operation::Read && !has_value_flag(args, flags::REF) {
        return "read requires --ref <ref>".to_owned();
    }
    if operation == Operation::Find && !has_value_flag(args, flags::QUERY) {
        return "find requires --query <text>".to_owned();
    }
    first_invalid_used_flag(operation, args, native_options)
        .unwrap_or_else(|| "invalid operation arguments".to_owned())
}

fn missing_value_error(args: &[String]) -> Option<String> {
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if known_value_flag(token, &[]).is_some() {
            let (flag, inline_value) = split_equals(token);
            if inline_value.is_none() && args.get(index + 1).is_none() {
                return Some(format!("{flag} requires a value"));
            }
            index += if inline_value.is_some() { 1 } else { 2 };
        } else {
            index += 1;
        }
    }
    None
}

fn invalid_protocol_output_error(args: &[String]) -> Option<String> {
    flag_value(args, flags::OUTPUT).and_then(|value| {
        parse_protocol_output(value)
            .err()
            .map(|_| input_errors::PROTOCOL_OUTPUT_ONLY.to_owned())
    })
}

fn has_path_candidate(args: &[String], native_options: &[NativeOptionSpec]) -> bool {
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if known_value_flag(token, native_options).is_some() {
            let (_flag, inline_value) = split_equals(token);
            index += if inline_value.is_some() { 1 } else { 2 };
        } else if is_flag(token) {
            index += 1;
        } else {
            return true;
        }
    }
    false
}

fn has_value_flag(args: &[String], expected: &str) -> bool {
    flag_value(args, expected).is_some()
}

fn flag_value<'a>(args: &'a [String], expected: &str) -> Option<&'a str> {
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        let (flag, inline_value) = split_equals(token);
        if flag == expected {
            return inline_value.or_else(|| args.get(index + 1).map(String::as_str));
        }
        index += 1;
    }
    None
}

fn first_invalid_used_flag(
    operation: Operation,
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Option<String> {
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        let Some(flag) = known_value_flag(token, native_options) else {
            index += 1;
            continue;
        };
        let (flag_token, inline_value) = split_equals(token);
        let value = inline_value.or_else(|| args.get(index + 1).map(String::as_str));
        if operation_uses_flag(operation, flag) {
            match (flag, value) {
                (_, None) => return Some(format!("{flag_token} requires a value")),
                (KnownValueFlag::Page, Some(value))
                    if value.parse::<u32>().ok().filter(|v| *v > 0).is_none() =>
                {
                    return Some(format!("{} must be a positive integer", flags::PAGE));
                }
                (KnownValueFlag::LimitChars, Some(value))
                    if value.parse::<u32>().ok().filter(|v| *v > 0).is_none() =>
                {
                    return Some(format!("{} must be a positive integer", flags::LIMIT_CHARS));
                }
                (KnownValueFlag::Output, Some(value)) if parse_output(value).is_err() => {
                    return Some(format!("invalid {} {value:?}", flags::OUTPUT));
                }
                (KnownValueFlag::Ref, Some("")) => {
                    return Some(format!("{} must not be empty", flags::REF));
                }
                (KnownValueFlag::Query, Some("")) => {
                    return Some(format!("{} must not be empty", flags::QUERY));
                }
                (KnownValueFlag::Native(spec), Some(value)) if spec.parse_value(value).is_err() => {
                    return spec.parse_value(value).err();
                }
                _ => {}
            }
        }
        index += if inline_value.is_some() { 1 } else { 2 };
    }
    None
}

#[cfg(test)]
mod tests;
