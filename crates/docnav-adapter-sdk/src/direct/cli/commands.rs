use std::io::{Read, Write};

use docnav_protocol::Operation;

use super::super::args::{command_names, parse_probe, parse_protocol_only_options};
use super::super::config::{adapter_direct_cli_config_source_descriptors, ConfigPathOverrides};
use super::operation::run_operation;
use super::{input_error, DirectCliContext, DirectCommandInvocation};
use crate::standard_parameters::InvokeStandardParameterConfig;
use crate::{invoke::invoke_once_with_standard_parameter_config, run_command, Adapter, SdkCommand};

pub(super) fn run_direct_command<A, R, W, E>(
    context: &DirectCliContext<'_, A>,
    command: DirectCommandInvocation<'_>,
    stdin: R,
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    A: Adapter,
    R: Read,
    W: Write,
    E: Write,
{
    match command.name {
        command_names::MANIFEST => run_manifest_command(context, command.args, stdout, stderr),
        command_names::PROBE => run_probe_command(context, command.args, stdout, stderr),
        command_names::INVOKE => run_invoke_command(context, command.args, stdin, stdout, stderr),
        command_names::OUTLINE => {
            run_operation(context, Operation::Outline, command.args, stdout, stderr)
        }
        command_names::READ => {
            run_operation(context, Operation::Read, command.args, stdout, stderr)
        }
        command_names::FIND => {
            run_operation(context, Operation::Find, command.args, stdout, stderr)
        }
        command_names::INFO => {
            run_operation(context, Operation::Info, command.args, stdout, stderr)
        }
        _ => unreachable!("known direct CLI commands are handled above"),
    }
}

fn run_manifest_command<A, W, E>(
    context: &DirectCliContext<'_, A>,
    args: &[String],
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    A: Adapter,
    W: Write,
    E: Write,
{
    match parse_protocol_only_options(args, context.config.native_options) {
        Ok(()) => run_command(
            context.adapter,
            SdkCommand::Manifest,
            std::io::empty(),
            stdout,
            stderr,
        ),
        Err(message) => input_error(stderr, &message),
    }
}

fn run_probe_command<A, W, E>(
    context: &DirectCliContext<'_, A>,
    args: &[String],
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    A: Adapter,
    W: Write,
    E: Write,
{
    match parse_probe(args, context.config.native_options) {
        Ok(options) => run_command(
            context.adapter,
            SdkCommand::Probe { path: options.path },
            std::io::empty(),
            stdout,
            stderr,
        ),
        Err(message) => input_error(stderr, &message),
    }
}

fn run_invoke_command<A, R, W, E>(
    context: &DirectCliContext<'_, A>,
    args: &[String],
    stdin: R,
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    A: Adapter,
    R: Read,
    W: Write,
    E: Write,
{
    if !args.is_empty() {
        return input_error(stderr, "invoke does not accept positional arguments");
    }
    let standard_parameters = match invoke_standard_parameter_config(&context.config) {
        Ok(config) => config,
        Err(message) => return input_error(stderr, &message),
    };
    invoke_once_with_standard_parameter_config(
        context.adapter,
        standard_parameters,
        stdin,
        stdout,
        stderr,
    )
}

fn invoke_standard_parameter_config(
    config: &super::DirectCliConfig<'_>,
) -> Result<InvokeStandardParameterConfig, String> {
    let cwd = std::env::current_dir()
        .map_err(|error| format!("failed to read current directory: {error}"))?;
    let descriptors = adapter_direct_cli_config_source_descriptors(
        config.adapter_id,
        config.default_user_config_dir,
        &cwd,
        ConfigPathOverrides::default(),
    );
    Ok(InvokeStandardParameterConfig {
        default_limit: config.default_limit,
        project_config: Some(descriptors.project),
        user_config: Some(descriptors.user),
    })
}
