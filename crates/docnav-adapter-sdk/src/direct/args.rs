use docnav_protocol::{positive_result, Operation, Options, PositiveInteger};
use serde_json::Value;

use super::native_options::{NativeOptionDefault, NativeOptionSpec};
use super::output::DirectOutputMode;
use super::warnings::DirectCliWarning;

mod diagnostics;
mod loose;
mod spec;

use diagnostics::{operation_parse_error, probe_parse_error, protocol_only_parse_error};
use loose::{collect_operation_args, collect_probe_args, collect_protocol_only_args, LooseArgs};
use spec::{
    arg_ids, command_labels, flags, operation_about, operation_command, parse_output,
    probe_command, protocol_only_command,
};

pub(super) use spec::{command_names, direct_cli_command};

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

    let common = parse_common_operation_options(operation, &matches, default_limit_chars)?;
    let specific = parse_specific_operation_fields(operation, &matches)?;
    let native_options = parsed_native_options(operation, &matches, native_options)?;

    Ok(DirectOperationOptions {
        path: common.path,
        page: common.page,
        limit_chars: common.limit_chars,
        output: common.output,
        ref_id: specific.ref_id,
        query: specific.query,
        warnings,
        native_options,
    })
}

struct CommonOperationOptions {
    path: String,
    page: PositiveInteger,
    limit_chars: PositiveInteger,
    output: DirectOutputMode,
}

struct SpecificOperationFields {
    ref_id: Option<String>,
    query: Option<String>,
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

fn positive_from_u32(value: u32, flag: &str) -> Result<PositiveInteger, String> {
    positive_result(value).map_err(|_| format!("{flag} must be a positive integer"))
}

fn parse_common_operation_options(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
    default_limit_chars: u32,
) -> Result<CommonOperationOptions, String> {
    let path = required_string(
        matches,
        arg_ids::PATH,
        &format!("{operation} requires <path>"),
    )?;
    let page = parse_operation_page(operation, matches)?;
    let limit_chars = parse_operation_limit_chars(operation, matches, default_limit_chars)?;
    let output = parse_output(&required_string(
        matches,
        arg_ids::OUTPUT,
        "missing output mode",
    )?)?;

    Ok(CommonOperationOptions {
        path,
        page,
        limit_chars,
        output,
    })
}

fn parse_operation_page(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
) -> Result<PositiveInteger, String> {
    if operation == Operation::Info {
        return Ok(positive_result(1).expect("static positive integer"));
    }

    let raw = matches.get_one::<u32>(arg_ids::PAGE).copied().unwrap_or(1);
    positive_from_u32(raw, flags::PAGE)
}

fn parse_operation_limit_chars(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
    default_limit_chars: u32,
) -> Result<PositiveInteger, String> {
    if operation == Operation::Info {
        return Ok(positive_result(default_limit_chars).expect("static positive integer"));
    }

    let raw = matches
        .get_one::<u32>(arg_ids::LIMIT_CHARS)
        .copied()
        .unwrap_or(default_limit_chars);
    positive_from_u32(raw, flags::LIMIT_CHARS)
}

fn parse_specific_operation_fields(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
) -> Result<SpecificOperationFields, String> {
    let ref_id = if operation == Operation::Read {
        Some(required_string(
            matches,
            arg_ids::REF,
            "read requires --ref <ref>",
        )?)
    } else {
        None
    };
    let query = if operation == Operation::Find {
        Some(required_string(
            matches,
            arg_ids::QUERY,
            "find requires --query <text>",
        )?)
    } else {
        None
    };

    Ok(SpecificOperationFields { ref_id, query })
}

fn clap_argv(command: &str, args: Vec<String>) -> Vec<String> {
    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push(command.to_owned());
    argv.extend(args);
    argv
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

#[cfg(test)]
mod tests;
