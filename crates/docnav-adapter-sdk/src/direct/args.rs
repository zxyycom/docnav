use std::fmt;

use docnav_protocol::{positive_result, Operation, Options, PositiveInteger};
use docnav_standard_parameters::StandardParameterConfigSourceIssue;
use serde_json::{Map, Value};

use super::cli::DirectCliConfig;
use super::native_options::NativeOptionSpec;
use super::output::DirectOutputMode;

mod boundaries;
mod diagnostics;
mod resolved;
mod sources;
mod spec;
mod standard;

use boundaries::{collect_probe_args, collect_protocol_only_args, BoundaryArgs};
use diagnostics::{operation_parse_error, probe_parse_error, protocol_only_parse_error};
use sources::direct_cli_parameter_sources;
use spec::{arg_ids, command_labels, flags, parse_output, probe_command, protocol_only_command};

use sources::DirectCliParameterSources;
pub(super) use spec::{command_names, direct_cli_command};

#[derive(Clone, Debug)]
pub(super) struct DirectOperationOptions {
    pub(super) path: String,
    pub(super) page: PositiveInteger,
    pub(super) limit: PositiveInteger,
    pub(super) output: DirectOutputMode,
    pub(super) ref_id: Option<String>,
    pub(super) query: Option<String>,
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

#[derive(Debug)]
pub(super) struct DirectProbeOptions {
    pub(super) path: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum DirectCliInputError {
    Message(String),
    ConfigSource(StandardParameterConfigSourceIssue),
}

impl DirectCliInputError {
    pub(super) fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

impl fmt::Display for DirectCliInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => formatter.write_str(message),
            Self::ConfigSource(issue) => formatter.write_str(&issue.message()),
        }
    }
}

impl From<String> for DirectCliInputError {
    fn from(message: String) -> Self {
        Self::Message(message)
    }
}

impl From<&str> for DirectCliInputError {
    fn from(message: &str) -> Self {
        Self::Message(message.to_owned())
    }
}

pub(super) fn parse_protocol_only_options(
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<(), String> {
    let BoundaryArgs { clap_args } =
        collect_protocol_only_args(command_labels::MANIFEST, args, native_options)?;
    protocol_only_command(command_names::MANIFEST, "Emit adapter manifest")
        .try_get_matches_from(clap_argv(command_names::MANIFEST, clap_args))
        .map_err(|_| protocol_only_parse_error(args))?;
    Ok(())
}

pub(super) fn parse_probe(
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<DirectProbeOptions, String> {
    let BoundaryArgs { clap_args } = collect_probe_args(args, native_options)?;
    let matches = probe_command()
        .try_get_matches_from(clap_argv(command_names::PROBE, clap_args))
        .map_err(|_| probe_parse_error(args))?;
    let path = required_string(&matches, arg_ids::PATH, "probe requires <path>")?;

    Ok(DirectProbeOptions { path })
}

pub(super) fn parse_operation_options(
    operation: Operation,
    args: &[String],
    config: &DirectCliConfig<'_>,
) -> Result<DirectOperationOptions, DirectCliInputError> {
    let sources = direct_cli_parameter_sources(operation, args, config)?;
    operation_options_from_sources(operation, sources, config.native_options)
}

fn positive_from_u32(value: u32, flag: &str) -> Result<PositiveInteger, String> {
    positive_result(value).map_err(|_| format!("{flag} must be a positive integer"))
}

fn parse_operation_limit(value: &Value) -> Result<PositiveInteger, String> {
    let raw = value
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| format!("{} must be a positive integer", flags::LIMIT))?;
    positive_from_u32(raw, flags::LIMIT)
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
            return Err(format!(
                "native option {key:?} is not supported by {}",
                operation.as_str()
            ));
        }
        options.insert(key.clone(), spec.parse_value(value)?);
    }
    Ok(options)
}

fn operation_options_from_sources(
    operation: Operation,
    sources: DirectCliParameterSources,
    native_specs: &[NativeOptionSpec],
) -> Result<DirectOperationOptions, DirectCliInputError> {
    let limit = parse_operation_limit(&sources.limit)?;
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
        limit,
        output,
        ref_id: sources.ref_id,
        query: sources.query,
        native_options,
    })
}

#[cfg(test)]
mod tests;
