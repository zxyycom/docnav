use docnav_protocol::{positive_result, Operation, Options, PositiveInteger};
use serde_json::{Map, Value};

use super::cli::DirectCliConfig;
use super::native_options::NativeOptionSpec;
use super::output::DirectOutputMode;
use super::warnings::DirectCliWarning;

mod diagnostics;
mod loose;
mod sources;
mod spec;

use diagnostics::{operation_parse_error, probe_parse_error, protocol_only_parse_error};
use loose::{collect_probe_args, collect_protocol_only_args, LooseArgs};
use sources::direct_cli_parameter_sources;
use spec::{arg_ids, command_labels, flags, parse_output, probe_command, protocol_only_command};

use sources::DirectCliParameterSources;
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
    config: &DirectCliConfig<'_>,
) -> Result<DirectOperationOptions, String> {
    let sources = direct_cli_parameter_sources(operation, args, config)?;
    operation_options_from_sources(operation, sources, config.native_options)
}

fn positive_from_u32(value: u32, flag: &str) -> Result<PositiveInteger, String> {
    positive_result(value).map_err(|_| format!("{flag} must be a positive integer"))
}

fn parse_operation_limit_chars(value: &Value) -> Result<PositiveInteger, String> {
    let raw = value
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| format!("{} must be a positive integer", flags::LIMIT_CHARS))?;
    positive_from_u32(raw, flags::LIMIT_CHARS)
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
    raw_options: &Map<String, Value>,
    specs: &[NativeOptionSpec],
) -> Result<Options, String> {
    let mut options = Options::new();
    for (key, value) in raw_options {
        let Some(spec) = specs.iter().find(|spec| spec.option_key == key) else {
            return Err(format!("unknown native option {key:?}"));
        };
        if !spec.supports(operation) {
            continue;
        }
        options.insert(key.clone(), spec.parse_value(value)?);
    }
    Ok(options)
}

fn operation_options_from_sources(
    operation: Operation,
    sources: DirectCliParameterSources,
    native_specs: &[NativeOptionSpec],
) -> Result<DirectOperationOptions, String> {
    let limit_chars = parse_operation_limit_chars(&sources.limit_chars)?;
    let output = parse_output(
        sources
            .output
            .as_str()
            .ok_or_else(|| format!("invalid {} {:?}", flags::OUTPUT, sources.output))?,
    )?;
    let native_options = parsed_native_options(operation, &sources.native_options, native_specs)?;

    Ok(DirectOperationOptions {
        path: sources.path,
        page: sources.page,
        limit_chars,
        output,
        ref_id: sources.ref_id,
        query: sources.query,
        warnings: sources.warnings,
        native_options,
    })
}

#[cfg(test)]
mod tests;
