use docnav_protocol::Operation;
use serde_json::Value;

use super::super::native_options::NativeOptionSpec;
use super::loose::{is_flag, known_value_flag, operation_uses_flag, split_equals, KnownValueFlag};
use super::spec::{flags, input_errors, parse_output, parse_protocol_output};

pub(super) fn protocol_only_parse_error(args: &[String]) -> String {
    missing_value_error(args)
        .or_else(|| invalid_protocol_output_error(args))
        .unwrap_or_else(|| "invalid protocol-only command arguments".to_owned())
}

pub(super) fn probe_parse_error(args: &[String]) -> String {
    if !has_path_candidate(args, &[]) {
        return "probe requires <path>".to_owned();
    }
    missing_value_error(args)
        .or_else(|| invalid_protocol_output_error(args))
        .unwrap_or_else(|| "invalid probe arguments".to_owned())
}

pub(super) fn operation_parse_error(
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
        let Some(occurrence) = direct_value_flag_occurrence(args, index, native_options) else {
            index += 1;
            continue;
        };
        if operation_uses_flag(operation, occurrence.flag) {
            if let Some(error) = direct_value_flag_error(occurrence) {
                return Some(error);
            }
        }
        index += occurrence.consumed;
    }
    None
}

#[derive(Clone, Copy)]
struct DirectValueFlagOccurrence<'a> {
    flag: KnownValueFlag<'a>,
    flag_token: &'a str,
    value: Option<&'a str>,
    consumed: usize,
}

fn direct_value_flag_occurrence<'a>(
    args: &'a [String],
    index: usize,
    native_options: &'a [NativeOptionSpec],
) -> Option<DirectValueFlagOccurrence<'a>> {
    let token = &args[index];
    let flag = known_value_flag(token, native_options)?;
    let (flag_token, inline_value) = split_equals(token);
    Some(DirectValueFlagOccurrence {
        flag,
        flag_token,
        value: inline_value.or_else(|| args.get(index + 1).map(String::as_str)),
        consumed: if inline_value.is_some() { 1 } else { 2 },
    })
}

fn direct_value_flag_error(occurrence: DirectValueFlagOccurrence<'_>) -> Option<String> {
    match (occurrence.flag, occurrence.value) {
        (_, None) => Some(format!("{} requires a value", occurrence.flag_token)),
        (KnownValueFlag::Page, Some(value)) => positive_flag_error(flags::PAGE, value),
        (KnownValueFlag::LimitChars, Some(value)) => positive_flag_error(flags::LIMIT_CHARS, value),
        (KnownValueFlag::Output, Some(value)) => output_flag_error(value),
        (KnownValueFlag::Ref, Some("")) => Some(format!("{} must not be empty", flags::REF)),
        (KnownValueFlag::Query, Some("")) => Some(format!("{} must not be empty", flags::QUERY)),
        (KnownValueFlag::Native(spec), Some(value)) => spec.parse_value(&Value::from(value)).err(),
        _ => None,
    }
}

fn positive_flag_error(flag: &str, value: &str) -> Option<String> {
    if value
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
        .is_some()
    {
        return None;
    }
    Some(format!("{flag} must be a positive integer"))
}

fn output_flag_error(value: &str) -> Option<String> {
    parse_output(value)
        .err()
        .map(|_| format!("invalid {} {value:?}", flags::OUTPUT))
}
