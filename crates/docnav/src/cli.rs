use std::num::NonZeroU32;
use std::str::FromStr;

use docnav_protocol::{Operation, PositiveInteger};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

mod flags {
    pub(super) const ADAPTER: &str = "--adapter";
    pub(super) const LIMIT_CHARS: &str = "--limit-chars";
    pub(super) const OPERATION: &str = "--operation";
    pub(super) const OUTPUT: &str = "--output";
    pub(super) const PAGE: &str = "--page";
    pub(super) const PATH: &str = "--path";
    pub(super) const QUERY: &str = "--query";
    pub(super) const REF: &str = "--ref";
    pub(super) const USER: &str = "--user";
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputMode {
    Text,
    ReadableJson,
    ProtocolJson,
}

impl OutputMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::ReadableJson => "readable-json",
            Self::ProtocolJson => "protocol-json",
        }
    }
}

impl FromStr for OutputMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "text" => Ok(Self::Text),
            "readable-json" => Ok(Self::ReadableJson),
            "protocol-json" => Ok(Self::ProtocolJson),
            _ => Err(format!("invalid --output {value:?}")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedCli {
    pub command: CliCommand,
    pub warnings: Vec<CliWarning>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliCommand {
    Document(DocumentCommand),
    Config(ConfigCommand),
    Init,
    Doctor,
    Version,
}

impl CliCommand {
    pub const fn operation(&self) -> Option<Operation> {
        match self {
            Self::Document(command) => Some(command.operation),
            Self::Config(_) | Self::Init | Self::Doctor | Self::Version => None,
        }
    }

    pub const fn output_mode(&self) -> Option<OutputMode> {
        match self {
            Self::Document(command) => command.output,
            Self::Config(_) | Self::Init | Self::Doctor | Self::Version => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentCommand {
    pub operation: Operation,
    pub path: String,
    pub ref_id: Option<String>,
    pub query: Option<String>,
    pub page: Option<PositiveInteger>,
    pub limit_chars: Option<PositiveInteger>,
    pub output: Option<OutputMode>,
    pub adapter: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigCommand {
    Get(ConfigGet),
    Set(ConfigSet),
    Unset(ConfigUnset),
    List(ConfigList),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigGet {
    pub key: String,
    pub user: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigSet {
    pub key: String,
    pub value: String,
    pub user: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigUnset {
    pub key: String,
    pub user: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigList {
    pub user: bool,
    pub path: Option<String>,
    pub operation: Option<Operation>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CliWarning {
    pub ignored_tokens: Vec<String>,
    pub kind: CliWarningKind,
    pub reason: String,
}

impl CliWarning {
    fn unknown_flag(token: &str) -> Self {
        Self {
            ignored_tokens: vec![token.to_owned()],
            kind: CliWarningKind::UnknownFlag,
            reason: "unknown CLI flag ignored".to_owned(),
        }
    }

    fn extra_positional(token: &str) -> Self {
        Self {
            ignored_tokens: vec![token.to_owned()],
            kind: CliWarningKind::ExtraPositional,
            reason: "extra positional argument ignored".to_owned(),
        }
    }

    fn unused_operation_flag(flag: &str, value: Option<&str>, command: &str) -> Self {
        let mut ignored_tokens = vec![flag.to_owned()];
        if let Some(value) = value {
            ignored_tokens.push(value.to_owned());
        }
        Self {
            ignored_tokens,
            kind: CliWarningKind::UnusedOperationFlag,
            reason: format!("flag is not used by {command} command"),
        }
    }

    pub fn adapter_candidate_failure(adapter_id: &str, reason: &str) -> Self {
        Self {
            ignored_tokens: vec![flags::ADAPTER.to_owned(), adapter_id.to_owned()],
            kind: CliWarningKind::AdapterCandidateFailure,
            reason: format!("preselected adapter was not used: {reason}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CliWarningKind {
    UnknownFlag,
    ExtraPositional,
    UnusedOperationFlag,
    AdapterCandidateFailure,
}

impl CliWarningKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownFlag => "unknown_flag",
            Self::ExtraPositional => "extra_positional",
            Self::UnusedOperationFlag => "unused_operation_flag",
            Self::AdapterCandidateFailure => "adapter_candidate_failure",
        }
    }
}

#[derive(Clone, Copy)]
enum ValueFlag {
    Adapter,
    LimitChars,
    Operation,
    Output,
    Page,
    Path,
    Query,
    Ref,
}

pub fn parse<I, S>(args: I) -> AppResult<ParsedCli>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    let Some((command, rest)) = args.split_first() else {
        return Err(AppError::invalid_request("command", "missing command"));
    };

    match command.as_str() {
        "outline" => parse_document_command(Operation::Outline, rest),
        "read" => parse_document_command(Operation::Read, rest),
        "find" => parse_document_command(Operation::Find, rest),
        "info" => parse_document_command(Operation::Info, rest),
        "config" => parse_config_command(rest),
        "init" => parse_nullary_command(CliCommand::Init, "init", rest),
        "doctor" => parse_nullary_command(CliCommand::Doctor, "doctor", rest),
        "version" => parse_nullary_command(CliCommand::Version, "version", rest),
        _ => Err(AppError::invalid_request(
            "command",
            format!("unknown command {command:?}"),
        )),
    }
}

fn parse_document_command(operation: Operation, args: &[String]) -> AppResult<ParsedCli> {
    let mut path = None;
    let mut ref_id = None;
    let mut query = None;
    let mut page = None;
    let mut limit_chars = None;
    let mut output = None;
    let mut adapter = None;
    let mut warnings = Vec::new();

    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if let Some(flag) = known_value_flag(token) {
            let value = args
                .get(index + 1)
                .ok_or_else(|| AppError::invalid_request(token, "flag requires a value"))?;
            if document_uses_flag(operation, flag) {
                match flag {
                    ValueFlag::Adapter => adapter = Some(non_empty_value(token, value)?),
                    ValueFlag::LimitChars => limit_chars = Some(parse_positive(value, token)?),
                    ValueFlag::Output => {
                        output =
                            Some(value.parse().map_err(|reason: String| {
                                AppError::invalid_request(token, reason)
                            })?)
                    }
                    ValueFlag::Page => page = Some(parse_positive(value, token)?),
                    ValueFlag::Query => query = Some(non_empty_value(token, value)?),
                    ValueFlag::Ref => ref_id = Some(non_empty_value(token, value)?),
                    ValueFlag::Operation | ValueFlag::Path => {
                        warnings.push(CliWarning::unused_operation_flag(
                            token,
                            Some(value),
                            operation.as_str(),
                        ));
                    }
                }
            } else {
                warnings.push(CliWarning::unused_operation_flag(
                    token,
                    Some(value),
                    operation.as_str(),
                ));
            }
            index += 2;
        } else if is_flag(token) {
            warnings.push(CliWarning::unknown_flag(token));
            index += 1;
        } else {
            if path.is_none() {
                path = Some(token.clone());
            } else {
                warnings.push(CliWarning::extra_positional(token));
            }
            index += 1;
        }
    }

    let path = path.ok_or_else(|| {
        AppError::invalid_request("path", format!("{} requires <path>", operation.as_str()))
    })?;
    if operation == Operation::Read && ref_id.is_none() {
        return Err(AppError::invalid_request(
            "--ref",
            "read requires --ref <ref>",
        ));
    }
    if operation == Operation::Find && query.is_none() {
        return Err(AppError::invalid_request(
            "--query",
            "find requires --query <text>",
        ));
    }

    Ok(ParsedCli {
        command: CliCommand::Document(DocumentCommand {
            operation,
            path,
            ref_id,
            query,
            page,
            limit_chars,
            output,
            adapter,
        }),
        warnings,
    })
}

fn parse_config_command(args: &[String]) -> AppResult<ParsedCli> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(AppError::invalid_request(
            "config",
            "missing config subcommand",
        ));
    };

    match subcommand.as_str() {
        "get" => parse_config_get(rest),
        "set" => parse_config_set(rest),
        "unset" => parse_config_unset(rest),
        "list" => parse_config_list(rest),
        _ => Err(AppError::invalid_request(
            "config",
            format!("unknown config subcommand {subcommand:?}"),
        )),
    }
}

fn parse_config_get(args: &[String]) -> AppResult<ParsedCli> {
    let parsed = parse_config_common("config get", args, ConfigFlagUsage::KeyOnly, 1)?;
    let key = required_positional(&parsed.positionals, "key", "config get requires <key>")?;
    Ok(ParsedCli {
        command: CliCommand::Config(ConfigCommand::Get(ConfigGet {
            key,
            user: parsed.user,
        })),
        warnings: parsed.warnings,
    })
}

fn parse_config_set(args: &[String]) -> AppResult<ParsedCli> {
    let parsed = parse_config_common("config set", args, ConfigFlagUsage::KeyOnly, 2)?;
    let key = required_positional(&parsed.positionals, "key", "config set requires <key>")?;
    let value = parsed
        .positionals
        .get(1)
        .cloned()
        .ok_or_else(|| AppError::invalid_request("value", "config set requires <value>"))?;
    Ok(ParsedCli {
        command: CliCommand::Config(ConfigCommand::Set(ConfigSet {
            key,
            value,
            user: parsed.user,
        })),
        warnings: parsed.warnings,
    })
}

fn parse_config_unset(args: &[String]) -> AppResult<ParsedCli> {
    let parsed = parse_config_common("config unset", args, ConfigFlagUsage::KeyOnly, 1)?;
    let key = required_positional(&parsed.positionals, "key", "config unset requires <key>")?;
    Ok(ParsedCli {
        command: CliCommand::Config(ConfigCommand::Unset(ConfigUnset {
            key,
            user: parsed.user,
        })),
        warnings: parsed.warnings,
    })
}

fn parse_config_list(args: &[String]) -> AppResult<ParsedCli> {
    let mut parsed = parse_config_common("config list", args, ConfigFlagUsage::PathContext, 0)?;
    if parsed.path.is_none() {
        if let Some(raw) = parsed.operation_raw.take() {
            parsed.warnings.push(CliWarning::unused_operation_flag(
                flags::OPERATION,
                Some(&raw),
                "config list",
            ));
        }
    }
    let operation = match parsed.operation_raw {
        Some(raw) => Some(parse_operation(&raw)?),
        None => None,
    };

    Ok(ParsedCli {
        command: CliCommand::Config(ConfigCommand::List(ConfigList {
            user: parsed.user,
            path: parsed.path,
            operation,
        })),
        warnings: parsed.warnings,
    })
}

#[derive(Clone, Copy)]
enum ConfigFlagUsage {
    KeyOnly,
    PathContext,
}

struct ParsedConfigCommon {
    user: bool,
    path: Option<String>,
    operation_raw: Option<String>,
    positionals: Vec<String>,
    warnings: Vec<CliWarning>,
}

fn parse_config_common(
    command: &str,
    args: &[String],
    usage: ConfigFlagUsage,
    expected_positionals: usize,
) -> AppResult<ParsedConfigCommon> {
    let mut parsed = ParsedConfigCommon {
        user: false,
        path: None,
        operation_raw: None,
        positionals: Vec::new(),
        warnings: Vec::new(),
    };

    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if token == flags::USER {
            parsed.user = true;
            index += 1;
        } else if let Some(flag) = known_value_flag(token) {
            let value = args
                .get(index + 1)
                .ok_or_else(|| AppError::invalid_request(token, "flag requires a value"))?;
            match (usage, flag) {
                (ConfigFlagUsage::PathContext, ValueFlag::Path) => {
                    parsed.path = Some(value.clone())
                }
                (ConfigFlagUsage::PathContext, ValueFlag::Operation) => {
                    parsed.operation_raw = Some(value.clone())
                }
                _ => parsed.warnings.push(CliWarning::unused_operation_flag(
                    token,
                    Some(value),
                    command,
                )),
            }
            index += 2;
        } else if is_flag(token) {
            parsed.warnings.push(CliWarning::unknown_flag(token));
            index += 1;
        } else {
            parsed.positionals.push(token.clone());
            index += 1;
        }
    }

    if parsed.positionals.len() > expected_positionals {
        for token in parsed.positionals.drain(expected_positionals..) {
            parsed.warnings.push(CliWarning::extra_positional(&token));
        }
    }

    Ok(parsed)
}

fn parse_nullary_command(
    command: CliCommand,
    label: &str,
    args: &[String],
) -> AppResult<ParsedCli> {
    let mut warnings = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if let Some(_flag) = known_value_flag(token) {
            let value = args
                .get(index + 1)
                .ok_or_else(|| AppError::invalid_request(token, "flag requires a value"))?;
            warnings.push(CliWarning::unused_operation_flag(token, Some(value), label));
            index += 2;
        } else if token == flags::USER || is_flag(token) {
            warnings.push(CliWarning::unknown_flag(token));
            index += 1;
        } else {
            warnings.push(CliWarning::extra_positional(token));
            index += 1;
        }
    }

    Ok(ParsedCli { command, warnings })
}

fn known_value_flag(token: &str) -> Option<ValueFlag> {
    match token {
        flags::ADAPTER => Some(ValueFlag::Adapter),
        flags::LIMIT_CHARS => Some(ValueFlag::LimitChars),
        flags::OPERATION => Some(ValueFlag::Operation),
        flags::OUTPUT => Some(ValueFlag::Output),
        flags::PAGE => Some(ValueFlag::Page),
        flags::PATH => Some(ValueFlag::Path),
        flags::QUERY => Some(ValueFlag::Query),
        flags::REF => Some(ValueFlag::Ref),
        _ => None,
    }
}

fn document_uses_flag(operation: Operation, flag: ValueFlag) -> bool {
    match flag {
        ValueFlag::Adapter | ValueFlag::Output => true,
        ValueFlag::Page | ValueFlag::LimitChars => operation != Operation::Info,
        ValueFlag::Ref => operation == Operation::Read,
        ValueFlag::Query => operation == Operation::Find,
        ValueFlag::Operation | ValueFlag::Path => false,
    }
}

fn is_flag(token: &str) -> bool {
    token.starts_with("--")
}

fn parse_positive(value: &str, flag: &str) -> AppResult<PositiveInteger> {
    let parsed = value.parse::<u32>().map_err(|_| {
        AppError::invalid_request(flag, format!("{flag} must be a positive integer"))
    })?;
    NonZeroU32::new(parsed).ok_or_else(|| {
        AppError::invalid_request(flag, format!("{flag} must be a positive integer"))
    })
}

fn non_empty_value(flag: &str, value: &str) -> AppResult<String> {
    if value.is_empty() {
        Err(AppError::invalid_request(
            flag,
            format!("{flag} value must not be empty"),
        ))
    } else {
        Ok(value.to_owned())
    }
}

fn required_positional(positionals: &[String], field: &str, reason: &str) -> AppResult<String> {
    positionals
        .first()
        .cloned()
        .ok_or_else(|| AppError::invalid_request(field, reason))
}

fn parse_operation(value: &str) -> AppResult<Operation> {
    value.parse().map_err(|_| {
        AppError::invalid_request("--operation", "expected outline, read, find, or info")
    })
}
