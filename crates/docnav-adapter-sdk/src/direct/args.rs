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

// 直接 CLI 输出模式字符串来自 CLI 主规范；protocol 层不复用这些阅读输出标签。
mod output_values {
    pub(super) const PROTOCOL_JSON: &str = "protocol-json";
    pub(super) const READABLE_JSON: &str = "readable-json";
    pub(super) const TEXT: &str = "text";
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

pub(super) fn parse_protocol_only_options(
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<Vec<DirectCliWarning>, String> {
    let mut warnings = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if let Some(flag) = known_value_flag(token, native_options) {
            let value = args
                .get(index + 1)
                .ok_or_else(|| format!("{token} requires a value"))?;
            match flag {
                KnownValueFlag::Output => parse_protocol_output(value)?,
                _ => {
                    push_unused_warning(&mut warnings, token, Some(value), command_labels::MANIFEST)
                }
            }
            index += 2;
        } else if is_flag(token) {
            push_unknown_warning(&mut warnings, token);
            index += 1;
        } else {
            push_extra_positional_warning(&mut warnings, token);
            index += 1;
        }
    }

    Ok(warnings)
}

pub(super) fn parse_probe(
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<DirectProbeOptions, String> {
    let mut path = None;
    let mut warnings = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if let Some(flag) = known_value_flag(token, native_options) {
            let value = args
                .get(index + 1)
                .ok_or_else(|| format!("{token} requires a value"))?;
            match flag {
                KnownValueFlag::Output => parse_protocol_output(value)?,
                _ => push_unused_warning(&mut warnings, token, Some(value), command_labels::PROBE),
            }
            index += 2;
        } else if is_flag(token) {
            push_unknown_warning(&mut warnings, token);
            index += 1;
        } else {
            if path.is_none() {
                path = Some(token.clone());
            } else {
                push_extra_positional_warning(&mut warnings, token);
            }
            index += 1;
        }
    }

    let Some(path) = path else {
        return Err("probe requires <path>".to_owned());
    };

    Ok(DirectProbeOptions { path, warnings })
}

pub(super) fn parse_operation_options(
    operation: Operation,
    args: &[String],
    default_limit_chars: u32,
    native_options: &[NativeOptionSpec],
) -> Result<DirectOperationOptions, String> {
    let mut options = DirectOperationOptionAccumulator {
        page: positive_result(1).expect("static positive integer"),
        limit_chars: positive_result(default_limit_chars).expect("static positive integer"),
        output: DirectOutputMode::Text,
        ref_id: None,
        query: None,
        native_options: default_native_options(operation, native_options),
    };

    let mut path = None;
    let mut warnings = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if let Some(flag) = known_value_flag(token, native_options) {
            let value = args
                .get(index + 1)
                .ok_or_else(|| format!("{token} requires a value"))?;
            if !apply_operation_flag(operation, flag, value, &mut options)? {
                push_unused_warning(&mut warnings, token, Some(value), operation.as_str());
            }
            index += 2;
        } else if is_flag(token) {
            push_unknown_warning(&mut warnings, token);
            index += 1;
        } else {
            if path.is_none() {
                path = Some(token.clone());
            } else {
                push_extra_positional_warning(&mut warnings, token);
            }
            index += 1;
        }
    }

    let Some(path) = path else {
        return Err(format!("{operation} requires <path>"));
    };
    Ok(DirectOperationOptions {
        path,
        page: options.page,
        limit_chars: options.limit_chars,
        output: options.output,
        ref_id: options.ref_id,
        query: options.query,
        warnings,
        native_options: options.native_options,
    })
}

struct DirectOperationOptionAccumulator {
    page: PositiveInteger,
    limit_chars: PositiveInteger,
    output: DirectOutputMode,
    ref_id: Option<String>,
    query: Option<String>,
    native_options: Options,
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
    match token {
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

fn apply_operation_flag(
    operation: Operation,
    flag: KnownValueFlag<'_>,
    value: &str,
    options: &mut DirectOperationOptionAccumulator,
) -> Result<bool, String> {
    match flag {
        KnownValueFlag::Page if operation != Operation::Info => {
            options.page = parse_positive(value, flags::PAGE)?;
            Ok(true)
        }
        KnownValueFlag::LimitChars if operation != Operation::Info => {
            options.limit_chars = parse_positive(value, flags::LIMIT_CHARS)?;
            Ok(true)
        }
        KnownValueFlag::Ref if operation == Operation::Read => {
            if value.is_empty() {
                return Err(format!("{} must not be empty", flags::REF));
            }
            options.ref_id = Some(value.to_owned());
            Ok(true)
        }
        KnownValueFlag::Query if operation == Operation::Find => {
            if value.is_empty() {
                return Err(format!("{} must not be empty", flags::QUERY));
            }
            options.query = Some(value.to_owned());
            Ok(true)
        }
        KnownValueFlag::Output => {
            options.output = parse_output(value)?;
            Ok(true)
        }
        KnownValueFlag::Native(spec) if spec.supports(operation) => {
            options
                .native_options
                .insert(spec.option_key.to_owned(), spec.parse_value(value)?);
            Ok(true)
        }
        _ => Ok(false),
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
        output_values::TEXT => Ok(DirectOutputMode::Text),
        output_values::READABLE_JSON => Ok(DirectOutputMode::ReadableJson),
        output_values::PROTOCOL_JSON => Ok(DirectOutputMode::ProtocolJson),
        _ => Err(format!("invalid {} {value:?}", flags::OUTPUT)),
    }
}

fn parse_positive(value: &str, flag: &str) -> Result<PositiveInteger, String> {
    let raw = value
        .parse::<u32>()
        .map_err(|_| format!("{flag} must be a positive integer"))?;
    positive_result(raw).map_err(|_| format!("{flag} must be a positive integer"))
}

fn is_flag(value: &str) -> bool {
    value.starts_with("--")
}

fn push_unknown_warning(warnings: &mut Vec<DirectCliWarning>, token: &str) {
    warnings.push(DirectCliWarning::unknown_flag(token));
}

fn push_extra_positional_warning(warnings: &mut Vec<DirectCliWarning>, token: &str) {
    warnings.push(DirectCliWarning::extra_positional(token));
}

fn push_unused_warning(
    warnings: &mut Vec<DirectCliWarning>,
    flag: &str,
    value: Option<&String>,
    command: &str,
) {
    warnings.push(DirectCliWarning::unused_operation_flag(
        flag,
        value.map(String::as_str),
        command,
    ));
}

#[cfg(test)]
mod tests {
    use super::super::warnings::{DirectCliWarningEffect, DirectCliWarningId};
    use super::super::NativeOptionValueSpec;
    use super::*;

    const MAX_HEADING_LEVEL_OPERATIONS: &[Operation] = &[Operation::Outline, Operation::Find];
    const MAX_HEADING_LEVEL: NativeOptionSpec = NativeOptionSpec {
        flag: "--max-heading-level",
        option_key: "max_heading_level",
        operations: MAX_HEADING_LEVEL_OPERATIONS,
        value: NativeOptionValueSpec::IntegerRange { min: 1, max: 6 },
        default: Some(NativeOptionDefault::Integer(3)),
    };

    #[test]
    fn unknown_flag_does_not_consume_following_positional() {
        let options = parse_operation_options(
            Operation::Outline,
            &args(&["--future", "doc.md"]),
            6000,
            &[],
        )
        .expect("parse options");

        assert_eq!(options.path, "doc.md");
        assert_eq!(
            options.warnings,
            vec![DirectCliWarning::unknown_flag("--future")]
        );
    }

    #[test]
    fn unknown_flag_with_equals_is_one_ignored_token() {
        let options = parse_operation_options(
            Operation::Outline,
            &args(&["--future=value", "doc.md"]),
            6000,
            &[],
        )
        .expect("parse options");

        assert_eq!(options.path, "doc.md");
        assert_eq!(options.warnings.len(), 1);
        assert_eq!(options.warnings[0].id, DirectCliWarningId::CliArgvIgnored);
        assert_eq!(
            options.warnings[0].effect,
            DirectCliWarningEffect::OperationContinued
        );
        assert_eq!(options.warnings[0].details.tokens, ["--future=value"]);
    }

    #[test]
    fn unknown_flag_does_not_consume_following_known_flag() {
        let options = parse_operation_options(
            Operation::Outline,
            &args(&["doc.md", "--future", "--output", "protocol-json"]),
            6000,
            &[],
        )
        .expect("parse options");

        assert_eq!(options.output, DirectOutputMode::ProtocolJson);
        assert_eq!(options.warnings.len(), 1);
        assert_eq!(options.warnings[0].id, DirectCliWarningId::CliArgvIgnored);
        assert_eq!(options.warnings[0].details.tokens, ["--future"]);
    }

    #[test]
    fn extra_positional_warns_after_path_slot_is_filled() {
        let options =
            parse_operation_options(Operation::Outline, &args(&["doc.md", "extra"]), 6000, &[])
                .expect("parse options");

        assert_eq!(options.path, "doc.md");
        assert_eq!(options.warnings.len(), 1);
        assert_eq!(options.warnings[0].id, DirectCliWarningId::CliArgvIgnored);
        assert_eq!(options.warnings[0].details.tokens, ["extra"]);
    }

    #[test]
    fn unused_known_value_flag_consumes_value_and_warns() {
        let options = parse_operation_options(
            Operation::Read,
            &args(&["doc.md", "--ref", "L1:Guide", "--max-heading-level", "nope"]),
            6000,
            &[MAX_HEADING_LEVEL],
        )
        .expect("parse options");

        assert_eq!(options.ref_id.as_deref(), Some("L1:Guide"));
        assert_eq!(options.warnings.len(), 1);
        assert_eq!(
            options.warnings[0].details.tokens,
            ["--max-heading-level", "nope"]
        );
        assert_eq!(options.warnings[0].id, DirectCliWarningId::CliArgvIgnored);
    }

    #[test]
    fn unused_core_value_flag_with_invalid_value_does_not_fail_info() {
        let options = parse_operation_options(
            Operation::Info,
            &args(&["doc.md", "--limit-chars", "nope"]),
            6000,
            &[],
        )
        .expect("parse options");

        assert_eq!(options.path, "doc.md");
        assert_eq!(options.limit_chars.get(), 6000);
        assert_eq!(options.warnings.len(), 1);
        assert_eq!(
            options.warnings[0].details.tokens,
            ["--limit-chars", "nope"]
        );
        assert_eq!(options.warnings[0].id, DirectCliWarningId::CliArgvIgnored);
    }

    #[test]
    fn unused_known_value_flag_consumes_value_that_looks_like_flag() {
        let options = parse_operation_options(
            Operation::Read,
            &args(&["doc.md", "--ref", "L1:Guide", "--query", "--future-value"]),
            6000,
            &[],
        )
        .expect("parse options");

        assert_eq!(options.ref_id.as_deref(), Some("L1:Guide"));
        assert_eq!(options.warnings.len(), 1);
        assert_eq!(
            options.warnings[0].details.tokens,
            ["--query", "--future-value"]
        );
        assert_eq!(options.warnings[0].id, DirectCliWarningId::CliArgvIgnored);
    }

    #[test]
    fn known_value_flag_accepts_token_that_looks_like_flag() {
        let options = parse_operation_options(
            Operation::Read,
            &args(&["doc.md", "--ref", "--future-value"]),
            6000,
            &[],
        )
        .expect("parse options");

        assert_eq!(options.ref_id.as_deref(), Some("--future-value"));
        assert!(options.warnings.is_empty());
    }

    #[test]
    fn protocol_only_commands_warn_but_keep_protocol_output() {
        let warnings = parse_protocol_only_options(
            &args(&["--future", "extra", "--output", "protocol-json"]),
            &[],
        )
        .expect("parse protocol-only options");

        assert_eq!(warnings.len(), 2);
        assert_eq!(warnings[0].id, DirectCliWarningId::CliArgvIgnored);
        assert_eq!(warnings[0].details.tokens, ["--future"]);
        assert_eq!(warnings[1].id, DirectCliWarningId::CliArgvIgnored);
        assert_eq!(warnings[1].details.tokens, ["extra"]);
    }

    #[test]
    fn probe_path_can_follow_unknown_flag() {
        let parsed = parse_probe(
            &args(&["--future", "doc.md", "--output", "protocol-json"]),
            &[],
        )
        .expect("parse probe options");

        assert_eq!(parsed.path, "doc.md");
        assert_eq!(parsed.warnings.len(), 1);
        assert_eq!(parsed.warnings[0].details.tokens, ["--future"]);
    }

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }
}
