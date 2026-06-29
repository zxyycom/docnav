use std::env;
use std::path::PathBuf;

use docnav_protocol::{Operation, PositiveInteger};
use docnav_standard_parameters::StandardParameterResolution;
use serde_json::{Map, Value};

use super::super::cli::DirectCliConfig;
use super::super::config::{adapter_direct_cli_config_source_descriptors, ConfigPathOverrides};
use super::super::native_options::NativeOptionSpec;
use super::super::warnings::DirectCliWarning;
use super::loose::{collect_operation_args, LooseArgs};
use super::resolved::{
    collect_diagnostics, merged_native_options, optional_string_value, required_json_value,
    required_string_value, resolved_limit, resolved_page,
};
use super::spec::{arg_ids, operation_about, operation_command};
use super::standard::{resolve_operation_parameters, ID_OUTPUT, ID_PATH, ID_QUERY, ID_REF};
use super::{clap_argv, operation_parse_error, required_string};

pub(super) struct DirectCliParameterSources {
    pub(super) path: String,
    pub(super) page: PositiveInteger,
    pub(super) limit: Value,
    pub(super) output: Value,
    pub(super) ref_id: Option<String>,
    pub(super) query: Option<String>,
    pub(super) native_options: Map<String, Value>,
    pub(super) warnings: Vec<DirectCliWarning>,
}

struct ResolvedDirectCliParameters {
    resolution: StandardParameterResolution,
    warnings: Vec<DirectCliWarning>,
}

pub(super) fn direct_cli_parameter_sources(
    operation: Operation,
    args: &[String],
    config: &DirectCliConfig<'_>,
) -> Result<DirectCliParameterSources, String> {
    let resolved = resolve_direct_cli_parameters(operation, args, config)?;
    direct_cli_sources_from_resolution(operation, resolved, config)
}

fn resolve_direct_cli_parameters(
    operation: Operation,
    args: &[String],
    config: &DirectCliConfig<'_>,
) -> Result<ResolvedDirectCliParameters, String> {
    let (matches, mut warnings) = parse_operation_matches(operation, args, config)?;
    let direct_input = direct_input(operation, &matches, config.native_options)?;
    let descriptors = config_source_descriptors(config, &matches)?;
    let resolution = resolve_operation_parameters(
        operation,
        direct_input,
        descriptors.project,
        descriptors.user,
        config.default_limit,
    )?;
    collect_diagnostics(&resolution, &mut warnings)?;

    Ok(ResolvedDirectCliParameters {
        resolution,
        warnings,
    })
}

fn direct_cli_sources_from_resolution(
    operation: Operation,
    resolved: ResolvedDirectCliParameters,
    config: &DirectCliConfig<'_>,
) -> Result<DirectCliParameterSources, String> {
    let resolution = resolved.resolution;
    Ok(DirectCliParameterSources {
        path: required_string_value(&resolution, ID_PATH)?,
        page: resolved_page(operation, &resolution)?,
        limit: resolved_limit(operation, &resolution, config.default_limit)?,
        output: required_json_value(&resolution, ID_OUTPUT)?,
        ref_id: optional_string_value(&resolution, ID_REF)?,
        query: optional_string_value(&resolution, ID_QUERY)?,
        warnings: resolved.warnings,
        native_options: merged_native_options(operation, &resolution, config.native_options),
    })
}

fn parse_operation_matches(
    operation: Operation,
    args: &[String],
    config: &DirectCliConfig<'_>,
) -> Result<(clap::parser::ArgMatches, Vec<DirectCliWarning>), String> {
    let LooseArgs {
        clap_args,
        warnings,
    } = collect_operation_args(operation, args, config.native_options)?;
    let matches = operation_command(
        operation,
        operation_about(operation),
        config.native_options,
        config.default_limit,
    )
    .try_get_matches_from(clap_argv(operation.as_str(), clap_args))
    .map_err(|_| operation_parse_error(operation, args, config.native_options))?;

    Ok((matches, warnings))
}

fn config_source_descriptors(
    config: &DirectCliConfig<'_>,
    matches: &clap::parser::ArgMatches,
) -> Result<super::super::config::AdapterDirectCliConfigSourceDescriptors, String> {
    let cwd =
        env::current_dir().map_err(|error| format!("failed to read current directory: {error}"))?;
    Ok(adapter_direct_cli_config_source_descriptors(
        config.adapter_id,
        config.default_user_config_dir,
        &cwd,
        ConfigPathOverrides {
            project: config_path_override(matches, arg_ids::PROJECT_CONFIG_PATH),
            user: config_path_override(matches, arg_ids::USER_CONFIG_PATH),
        },
    ))
}

fn direct_input(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
    native_specs: &[NativeOptionSpec],
) -> Result<Value, String> {
    let mut input = Map::new();
    insert_path_input(operation, matches, &mut input)?;
    insert_window_input(operation, matches, &mut input);
    insert_output_input(matches, &mut input);
    insert_operation_input(operation, matches, &mut input)?;
    insert_native_options_input(operation, matches, native_specs, &mut input);
    Ok(Value::Object(input))
}

fn insert_path_input(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
    input: &mut Map<String, Value>,
) -> Result<(), String> {
    input.insert(
        "path".to_owned(),
        Value::from(required_string(
            matches,
            arg_ids::PATH,
            &format!("{operation} requires <path>"),
        )?),
    );
    Ok(())
}

fn insert_window_input(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
    input: &mut Map<String, Value>,
) {
    if operation != Operation::Info {
        if let Some(value) = command_line_string_value(matches, arg_ids::PAGINATION) {
            input.insert("pagination".to_owned(), pagination_value(value));
        }
        if let Some(value) = command_line_u32_value(matches, arg_ids::PAGE) {
            input.insert("page".to_owned(), value);
        }
        if let Some(value) = command_line_u32_value(matches, arg_ids::LIMIT) {
            input.insert("limit".to_owned(), value);
        }
    }
}

fn pagination_value(value: Value) -> Value {
    Value::Bool(value.as_str() == Some(super::spec::pagination_values::ENABLED))
}

fn insert_output_input(matches: &clap::parser::ArgMatches, input: &mut Map<String, Value>) {
    if let Some(value) = command_line_string_value(matches, arg_ids::OUTPUT) {
        input.insert("output".to_owned(), value);
    }
}

fn insert_operation_input(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
    input: &mut Map<String, Value>,
) -> Result<(), String> {
    match operation {
        Operation::Read => {
            input.insert(
                "ref".to_owned(),
                Value::from(required_string(
                    matches,
                    arg_ids::REF,
                    "read requires --ref <ref>",
                )?),
            );
        }
        Operation::Find => {
            input.insert(
                "query".to_owned(),
                Value::from(required_string(
                    matches,
                    arg_ids::QUERY,
                    "find requires --query <text>",
                )?),
            );
        }
        Operation::Outline | Operation::Info => {}
    }
    Ok(())
}

fn insert_native_options_input(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
    native_specs: &[NativeOptionSpec],
    input: &mut Map<String, Value>,
) {
    let native_options = direct_native_options(operation, matches, native_specs);
    if !native_options.is_empty() {
        input.insert("options".to_owned(), Value::Object(native_options));
    }
}

fn direct_native_options(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
    specs: &[NativeOptionSpec],
) -> Map<String, Value> {
    let mut options = Map::new();
    for spec in specs.iter().filter(|spec| spec.supports(operation)) {
        if let Some(value) = command_line_string_value(matches, spec.option_key) {
            options.insert(spec.option_key.to_owned(), value);
        }
    }
    options
}

fn command_line_u32_value(matches: &clap::parser::ArgMatches, id: &str) -> Option<Value> {
    if !is_command_line_value(matches, id) {
        return None;
    }
    matches.get_one::<u32>(id).copied().map(Value::from)
}

fn command_line_string_value(matches: &clap::parser::ArgMatches, id: &str) -> Option<Value> {
    if !is_command_line_value(matches, id) {
        return None;
    }
    matches.get_one::<String>(id).cloned().map(Value::from)
}

fn config_path_override(matches: &clap::parser::ArgMatches, id: &str) -> Option<PathBuf> {
    matches.get_one::<String>(id).map(PathBuf::from)
}

fn is_command_line_value(matches: &clap::parser::ArgMatches, id: &str) -> bool {
    matches.value_source(id) == Some(clap::parser::ValueSource::CommandLine)
}
