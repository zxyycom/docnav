use std::env;
use std::path::PathBuf;

use docnav_protocol::{positive_result, Operation, PositiveInteger};
use serde_json::{Map, Value};

use super::super::cli::DirectCliConfig;
use super::super::config::{
    load_adapter_direct_cli_config, ConfigPathOverrides, LoadedAdapterDirectCliConfig,
};
use super::super::native_options::{NativeOptionDefault, NativeOptionSpec};
use super::super::warnings::DirectCliWarning;
use super::loose::{collect_operation_args, LooseArgs};
use super::spec::{arg_ids, default_output_value, flags, operation_about, operation_command};
use super::{clap_argv, operation_parse_error, positive_from_u32, required_string};

pub(super) struct DirectCliParameterSources {
    pub(super) path: String,
    pub(super) page: PositiveInteger,
    pub(super) limit_chars: Value,
    pub(super) output: Value,
    pub(super) ref_id: Option<String>,
    pub(super) query: Option<String>,
    pub(super) native_options: Map<String, Value>,
    pub(super) warnings: Vec<DirectCliWarning>,
}

pub(super) fn direct_cli_parameter_sources(
    operation: Operation,
    args: &[String],
    config: &DirectCliConfig<'_>,
) -> Result<DirectCliParameterSources, String> {
    let (matches, mut warnings) = parse_operation_matches(operation, args, config)?;
    let path = required_string(
        &matches,
        arg_ids::PATH,
        &format!("{operation} requires <path>"),
    )?;
    let page = parse_operation_page(operation, &matches)?;
    let ref_id = operation_ref_id(operation, &matches)?;
    let query = operation_query(operation, &matches)?;
    let loaded_config = load_config_sources(config, &matches)?;
    warnings.extend(loaded_config.warnings.iter().cloned());

    Ok(DirectCliParameterSources {
        path,
        page,
        limit_chars: merged_limit_chars(
            operation,
            &matches,
            &loaded_config,
            config.default_limit_chars,
        ),
        output: merged_output(&matches, &loaded_config),
        ref_id,
        query,
        warnings,
        native_options: merged_native_options(
            operation,
            &matches,
            &loaded_config,
            config.native_options,
        ),
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
        config.default_limit_chars,
    )
    .try_get_matches_from(clap_argv(operation.as_str(), clap_args))
    .map_err(|_| operation_parse_error(operation, args, config.native_options))?;

    Ok((matches, warnings))
}

fn load_config_sources(
    config: &DirectCliConfig<'_>,
    matches: &clap::parser::ArgMatches,
) -> Result<LoadedAdapterDirectCliConfig, String> {
    let cwd =
        env::current_dir().map_err(|error| format!("failed to read current directory: {error}"))?;
    Ok(load_adapter_direct_cli_config(
        config.adapter_id,
        config.default_user_config_dir,
        &cwd,
        ConfigPathOverrides {
            project: config_path_override(matches, arg_ids::PROJECT_CONFIG_PATH),
            user: config_path_override(matches, arg_ids::USER_CONFIG_PATH),
        },
    ))
}

fn default_native_option_values(
    operation: Operation,
    specs: &[NativeOptionSpec],
) -> Map<String, Value> {
    let mut options = Map::new();
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

fn operation_ref_id(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
) -> Result<Option<String>, String> {
    if operation == Operation::Read {
        Ok(Some(required_string(
            matches,
            arg_ids::REF,
            "read requires --ref <ref>",
        )?))
    } else {
        Ok(None)
    }
}

fn operation_query(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
) -> Result<Option<String>, String> {
    if operation == Operation::Find {
        Ok(Some(required_string(
            matches,
            arg_ids::QUERY,
            "find requires --query <text>",
        )?))
    } else {
        Ok(None)
    }
}

fn merged_limit_chars(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
    config: &LoadedAdapterDirectCliConfig,
    default_limit_chars: u32,
) -> Value {
    if operation == Operation::Info {
        return Value::from(default_limit_chars);
    }
    command_line_u32_value(matches, arg_ids::LIMIT_CHARS)
        .or_else(|| config.project.limit_chars.clone())
        .or_else(|| config.user.limit_chars.clone())
        .unwrap_or_else(|| Value::from(default_limit_chars))
}

fn merged_output(
    matches: &clap::parser::ArgMatches,
    config: &LoadedAdapterDirectCliConfig,
) -> Value {
    command_line_string_value(matches, arg_ids::OUTPUT)
        .or_else(|| config.project.output.clone())
        .or_else(|| config.user.output.clone())
        .unwrap_or_else(|| Value::from(default_output_value()))
}

fn merged_native_options(
    operation: Operation,
    matches: &clap::parser::ArgMatches,
    config: &LoadedAdapterDirectCliConfig,
    specs: &[NativeOptionSpec],
) -> Map<String, Value> {
    let mut options = default_native_option_values(operation, specs);
    options.extend(config.user.native_options.clone());
    options.extend(config.project.native_options.clone());
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
