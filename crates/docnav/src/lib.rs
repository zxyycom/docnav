mod adapter_output_contract;
mod adapter_process;
mod cli;
mod config;
mod error;
mod invoke;
mod output;
mod project_context;
mod project_paths;
mod registry;
mod routing;
mod runtime;
mod standard_parameters;

use std::io::{Read, Write};

use cli::{CliCommand, OutputMode};
use error::AppResult;
use runtime::{AdapterRuntime, DocnavRuntime};

pub fn run<I, S, R, W, E>(args: I, stdin: R, stdout: W, stderr: E) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
    R: Read,
    W: Write,
    E: Write,
{
    run_with_runtime(args, stdin, stdout, stderr, &AdapterRuntime)
}

fn run_with_runtime<I, S, R, W, E, T>(
    args: I,
    _stdin: R,
    mut stdout: W,
    mut stderr: E,
    runtime: &T,
) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
    R: Read,
    W: Write,
    E: Write,
    T: DocnavRuntime,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    let output_context = cli::output_context(&args);
    let parsed = match cli::parse(args) {
        Ok(parsed) => parsed,
        Err(error) => {
            return output::write_error(output::ErrorOutput {
                error: &error,
                output_mode: output_context.output_mode,
                operation: output_context.operation,
                warnings: &[],
                stdout: &mut stdout,
                stderr: &mut stderr,
            })
        }
    };

    let cli::ParsedCli { command, warnings } = parsed;
    let output_mode = command.output_mode().unwrap_or(OutputMode::ReadableView);
    let operation = command.operation();
    match execute(command, runtime) {
        Ok(outcome) => output::write_outcome(outcome, &warnings, &mut stdout, &mut stderr),
        Err(error) => output::write_error(output::ErrorOutput {
            error: &error,
            output_mode,
            operation,
            warnings: &warnings,
            stdout: &mut stdout,
            stderr: &mut stderr,
        }),
    }
}

fn execute<T: DocnavRuntime>(
    command: CliCommand,
    runtime: &T,
) -> AppResult<output::CommandOutcome> {
    match command {
        CliCommand::Document(command) => {
            let context = config::load_context()?;
            let request = runtime::DocumentRequest::from_command(command, &context)?;
            runtime.execute_document(request)
        }
        CliCommand::Config(command) => config::execute(command, runtime),
        CliCommand::Init => config::init_project(),
        CliCommand::Doctor => config::doctor(),
        CliCommand::Version => Ok(output::CommandOutcome::plain_text(format!(
            "docnav {}",
            env!("CARGO_PKG_VERSION")
        ))),
        CliCommand::Help(text) => Ok(output::CommandOutcome::plain_text(text)),
    }
}
