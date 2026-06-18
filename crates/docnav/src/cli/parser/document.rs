use docnav_cli_args::{scan_loose_args, LooseArgScan};
use docnav_protocol::Operation;

use crate::error::{AppError, AppResult};

use super::super::flags;
use super::super::types::{CliCommand, DocumentCommand, ParsedCli};
use super::common::{
    clap_argv, is_flag, known_value_flag, loose_value_flags, missing_value_error,
    optional_explicit_output, optional_explicit_positive, optional_explicit_string,
    required_string, split_equals, warning_from_ignored_arg, ValueFlag,
};
use super::{arg_ids, document_clap_command};

pub(super) fn parse_document_command(
    operation: Operation,
    args: &[String],
) -> AppResult<ParsedCli> {
    let LooseDocumentArgs {
        clap_args,
        warnings,
    } = collect_document_args(operation, args)?;
    let matches = document_clap_command(operation)
        .try_get_matches_from(clap_argv(operation.as_str(), clap_args))
        .map_err(|_| document_parse_error(operation, args))?;

    let path = required_string(&matches, arg_ids::PATH, "path")?;
    let ref_id = if operation == Operation::Read {
        optional_explicit_string(&matches, arg_ids::REF)
    } else {
        None
    };
    let query = if operation == Operation::Find {
        optional_explicit_string(&matches, arg_ids::QUERY)
    } else {
        None
    };
    let page = if operation == Operation::Info {
        None
    } else {
        optional_explicit_positive(&matches, arg_ids::PAGE, flags::PAGE)?
    };
    let limit_chars = if operation == Operation::Info {
        None
    } else {
        optional_explicit_positive(&matches, arg_ids::LIMIT_CHARS, flags::LIMIT_CHARS)?
    };
    let output = optional_explicit_output(&matches)?;
    let adapter = optional_explicit_string(&matches, arg_ids::ADAPTER);

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

struct LooseDocumentArgs {
    clap_args: Vec<String>,
    warnings: Vec<super::super::warning::CliWarning>,
}

fn collect_document_args(operation: Operation, args: &[String]) -> AppResult<LooseDocumentArgs> {
    let known_value_flags = loose_value_flags(|flag| document_uses_flag(operation, flag));
    let scan = scan_loose_args(
        args,
        &LooseArgScan::new(operation.as_str(), 1, &known_value_flags),
    )
    .map_err(missing_value_error)?;

    Ok(LooseDocumentArgs {
        clap_args: scan.retained_args,
        warnings: scan
            .ignored
            .into_iter()
            .map(warning_from_ignored_arg)
            .collect(),
    })
}

fn document_parse_error(operation: Operation, args: &[String]) -> AppError {
    if !has_path_candidate(args) {
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
        _ => first_invalid_used_flag(operation, args)
            .unwrap_or_else(|| AppError::invalid_request("argv", "invalid command line arguments")),
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

fn has_path_candidate(args: &[String]) -> bool {
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if known_value_flag(token).is_some() {
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
    args.iter().any(|token| {
        let (flag, value) = split_equals(token);
        flag == expected && value.is_some()
    }) || args
        .windows(2)
        .any(|window| window.first().is_some_and(|token| token == expected))
}

fn first_invalid_used_flag(operation: Operation, args: &[String]) -> Option<AppError> {
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        let Some(flag) = known_value_flag(token) else {
            index += 1;
            continue;
        };
        let (flag_token, inline_value) = split_equals(token);
        let value = inline_value.or_else(|| args.get(index + 1).map(String::as_str));
        if document_uses_flag(operation, flag) {
            match (flag, value) {
                (_, None) => {
                    return Some(AppError::invalid_request(
                        flag_token,
                        "flag requires a value",
                    ))
                }
                (ValueFlag::Page, Some(value))
                    if value.parse::<u32>().ok().filter(|v| *v > 0).is_none() =>
                {
                    return Some(AppError::invalid_request(
                        flags::PAGE,
                        format!("{} must be a positive integer", flags::PAGE),
                    ));
                }
                (ValueFlag::LimitChars, Some(value))
                    if value.parse::<u32>().ok().filter(|v| *v > 0).is_none() =>
                {
                    return Some(AppError::invalid_request(
                        flags::LIMIT_CHARS,
                        format!("{} must be a positive integer", flags::LIMIT_CHARS),
                    ));
                }
                (ValueFlag::Output, Some(value)) => {
                    if let Err(reason) = value.parse::<super::super::types::OutputMode>() {
                        return Some(AppError::invalid_request(
                            flags::OUTPUT,
                            format!("invalid {}: {reason}", flags::OUTPUT),
                        ));
                    }
                }
                (ValueFlag::Ref, Some("")) => {
                    return Some(AppError::invalid_request(
                        flags::REF,
                        format!("{} value must not be empty", flags::REF),
                    ));
                }
                (ValueFlag::Query, Some("")) => {
                    return Some(AppError::invalid_request(
                        flags::QUERY,
                        format!("{} value must not be empty", flags::QUERY),
                    ));
                }
                _ => {}
            }
        }
        index += if inline_value.is_some() { 1 } else { 2 };
    }
    None
}
