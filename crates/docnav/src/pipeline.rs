mod adapter;
mod configuration;
mod document;
mod meta;
mod project;

use crate::cli::{
    AdapterCommand, CliCommand, ConfigCommand, ConfigPathArgs, DocumentCommand, OutputMode,
};
use crate::error::AppResult;
use crate::output::CommandOutcome;
use crate::runtime::DocnavRuntime;

pub(crate) fn execute<T: DocnavRuntime>(
    command: CliCommand,
    runtime: &T,
    error_output_mode: &mut OutputMode,
) -> AppResult<CommandOutcome> {
    PipelineContext::from_runtime(runtime).execute(command, error_output_mode)
}

struct PipelineContext<'a, T: DocnavRuntime> {
    services: PipelineServices<'a, T>,
}

impl<'a, T: DocnavRuntime> PipelineContext<'a, T> {
    fn from_runtime(runtime: &'a T) -> Self {
        Self {
            services: PipelineServices { runtime },
        }
    }

    fn execute(
        &self,
        command: CliCommand,
        error_output_mode: &mut OutputMode,
    ) -> AppResult<CommandOutcome> {
        match CommandFamily::from(command) {
            CommandFamily::Document(command) => document::execute(command, self, error_output_mode),
            CommandFamily::Config(command) => configuration::execute(command, self),
            CommandFamily::Adapter(command) => adapter::execute(command),
            CommandFamily::Project(command) => project::execute(command),
            CommandFamily::Meta(command) => meta::execute(command),
        }
    }

    fn services(&self) -> &PipelineServices<'a, T> {
        &self.services
    }
}

struct PipelineServices<'a, T: DocnavRuntime> {
    runtime: &'a T,
}

impl<T: DocnavRuntime> PipelineServices<'_, T> {
    fn runtime(&self) -> &T {
        self.runtime
    }
}

enum CommandFamily {
    Document(DocumentCommand),
    Config(ConfigCommand),
    Adapter(AdapterCommand),
    Project(ProjectCommand),
    Meta(MetaCommand),
}

enum ProjectCommand {
    Init(ConfigPathArgs),
    Doctor(ConfigPathArgs),
}

enum MetaCommand {
    Version,
    Help(String),
}

impl From<CliCommand> for CommandFamily {
    fn from(command: CliCommand) -> Self {
        match command {
            CliCommand::Document(command) => Self::Document(command),
            CliCommand::Config(command) => Self::Config(command),
            CliCommand::Adapter(command) => Self::Adapter(command),
            CliCommand::Init(config_paths) => Self::Project(ProjectCommand::Init(config_paths)),
            CliCommand::Doctor(config_paths) => Self::Project(ProjectCommand::Doctor(config_paths)),
            CliCommand::Version => Self::Meta(MetaCommand::Version),
            CliCommand::Help(text) => Self::Meta(MetaCommand::Help(text)),
        }
    }
}
