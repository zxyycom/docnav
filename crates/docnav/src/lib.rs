mod cli;
mod config;
mod error;
mod invocation_log;
mod navigation_defaults;
mod output;
mod pipeline;
mod project_context;
mod project_paths;
mod registry;
mod runtime;

use std::io::{Read, Write};

use cli::OutputMode;
use docnav_protocol::Operation;
use error::{AppError, AppResult};
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
    let invocation = RunInvocation::collect(args);
    let invocation = match invocation.parse() {
        Ok(invocation) => invocation,
        Err(failure) => {
            return write_invocation_error(
                &failure.error,
                failure.output_context,
                &mut stdout,
                &mut stderr,
            )
        }
    };

    let execution = invocation.execute(runtime);
    match execution.result {
        Ok(outcome) => output::write_outcome(outcome, &mut stdout, &mut stderr),
        Err(error) => {
            write_invocation_error(&error, execution.output_context, &mut stdout, &mut stderr)
        }
    }
}

struct RunInvocation {
    args: Vec<String>,
    output_context: InvocationOutputContext,
}

impl RunInvocation {
    fn collect<I, S>(args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let args: Vec<String> = args.into_iter().map(Into::into).collect();
        let output_context = InvocationOutputContext::preflight(&args);
        Self {
            args,
            output_context,
        }
    }

    fn parse(self) -> Result<CliInvocation, InvocationParseFailure> {
        match cli::parse(self.args) {
            Ok(parsed) => Ok(CliInvocation::from_parsed(parsed)),
            Err(error) => Err(InvocationParseFailure {
                error,
                output_context: self.output_context,
            }),
        }
    }
}

struct InvocationParseFailure {
    error: AppError,
    output_context: InvocationOutputContext,
}

struct CliInvocation {
    command: cli::CliCommand,
    output_context: InvocationOutputContext,
}

impl CliInvocation {
    fn from_parsed(parsed: cli::ParsedCli) -> Self {
        let cli::ParsedCli { command } = parsed;
        let output_context = InvocationOutputContext::parsed_command(&command);
        Self {
            command,
            output_context,
        }
    }

    fn execute<T: DocnavRuntime>(self, runtime: &T) -> InvocationExecution {
        let Self {
            command,
            output_context,
        } = self;
        InvocationExecution {
            result: pipeline::execute(command, runtime),
            output_context,
        }
    }
}

struct InvocationExecution {
    result: AppResult<output::CommandOutcome>,
    output_context: InvocationOutputContext,
}

#[derive(Clone, Copy)]
struct InvocationOutputContext {
    output_mode: OutputMode,
    operation: Option<Operation>,
}

impl InvocationOutputContext {
    fn preflight(args: &[String]) -> Self {
        let output_context = cli::output_context(args);
        Self {
            output_mode: output_context.output_mode,
            operation: output_context.operation,
        }
    }

    fn parsed_command(command: &cli::CliCommand) -> Self {
        Self {
            output_mode: command.output_mode().unwrap_or(OutputMode::ReadableView),
            operation: command.operation(),
        }
    }
}

fn write_invocation_error<W: Write, E: Write>(
    error: &AppError,
    output_context: InvocationOutputContext,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    output::write_error(output::ErrorOutput {
        error,
        output_mode: output_context.output_mode,
        operation: output_context.operation,
        stdout,
        stderr,
    })
}
