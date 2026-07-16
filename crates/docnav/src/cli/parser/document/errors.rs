use clap::error::ErrorKind;
use docnav_protocol::Operation;

use crate::error::AppError;

use super::super::super::flags;
use super::super::argument_helpers::{
    error_from_rejected_arg, is_flag, missing_value_flag_error, split_equals,
};
use super::value_flags::DocumentValueFlags;

pub(super) fn document_parse_error(
    operation: Operation,
    args: &[String],
    value_flags: &DocumentValueFlags,
    clap_error: &clap::Error,
) -> AppError {
    if clap_error.kind() == ErrorKind::UnknownArgument {
        return unknown_or_unused_argument(operation, args, value_flags);
    }
    if !has_path_candidate(args, value_flags) {
        return AppError::invalid_request(
            "path",
            format!("{} requires <path>", operation.as_str()),
        );
    }
    match operation {
        Operation::Read if !has_value_flag(args, flags::REF) => {
            AppError::invalid_request(flags::REF, "read requires --ref <ref>")
        }
        Operation::Find if !has_value_flag(args, flags::QUERY) => {
            AppError::invalid_request(flags::QUERY, "find requires --query <text>")
        }
        _ => missing_value_arg(args, value_flags).map_or_else(
            || AppError::invalid_request("argv", "invalid command line arguments"),
            missing_value_flag_error,
        ),
    }
}

fn unknown_or_unused_argument(
    operation: Operation,
    args: &[String],
    value_flags: &DocumentValueFlags,
) -> AppError {
    let token = args
        .iter()
        .find(|token| {
            value_flags.is_unused(token) || (is_flag(token) && !value_flags.contains(token))
        })
        .cloned()
        .unwrap_or_else(|| "--unknown".to_owned());
    if value_flags.is_unused(&token) {
        return error_from_rejected_arg(docnav_cli_args::RejectedArg::UnusedValueFlag {
            flag: token,
            value: None,
            command: operation.as_str().to_owned(),
        });
    }
    error_from_rejected_arg(docnav_cli_args::RejectedArg::UnknownFlag { token })
}

fn missing_value_arg<'a>(args: &'a [String], value_flags: &DocumentValueFlags) -> Option<&'a str> {
    args.iter().enumerate().find_map(|(index, token)| {
        let (flag, inline_value) = split_equals(token);
        (inline_value.is_none()
            && value_flags.takes_value(flag) == Some(true)
            && !value_flags.is_unused(flag)
            && args.get(index + 1).is_none_or(|next| is_flag(next)))
        .then_some(flag)
    })
}

fn has_path_candidate(args: &[String], value_flags: &DocumentValueFlags) -> bool {
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if let Some(takes_value) = value_flags.takes_value(token) {
            let (_, inline_value) = split_equals(token);
            index += if takes_value && inline_value.is_none() {
                2
            } else {
                1
            };
        } else if is_flag(token) {
            index += 1;
        } else {
            return true;
        }
    }
    false
}

fn has_value_flag(args: &[String], expected: &str) -> bool {
    args.iter().any(|token| {
        let (flag, value) = split_equals(token);
        flag == expected && value.is_some()
    }) || args
        .windows(2)
        .any(|window| window.first().is_some_and(|token| token == expected))
}
