mod command_model;
mod flags;
mod parser;
mod preflight;
pub mod warning;

pub use command_model::{
    CliCommand, ConfigCommand, ConfigGet, ConfigList, ConfigSet, ConfigUnset, DocumentCommand,
    OutputMode, ParsedCli,
};
pub use parser::parse;
pub use preflight::output_context;
pub use warning::CliWarning;
