use docnav_protocol::Operation;

use crate::error::{AppError, AppResult};

use super::super::types::{CliCommand, DocumentCommand, ParsedCli};
use super::super::warning::CliWarning;
use super::common::{known_value_flag, non_empty_value, parse_positive, ValueFlag};

pub(super) fn parse_document_command(
    operation: Operation,
    args: &[String],
) -> AppResult<ParsedCli> {
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
        } else if super::common::is_flag(token) {
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

fn document_uses_flag(operation: Operation, flag: ValueFlag) -> bool {
    match flag {
        ValueFlag::Adapter | ValueFlag::Output => true,
        ValueFlag::Page | ValueFlag::LimitChars => operation != Operation::Info,
        ValueFlag::Ref => operation == Operation::Read,
        ValueFlag::Query => operation == Operation::Find,
        ValueFlag::Operation | ValueFlag::Path => false,
    }
}
