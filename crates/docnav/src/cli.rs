mod command_model;
mod flags;
mod parser;
mod preflight;

pub use command_model::{
    AdapterCommand, CliCommand, ConfigCommand, ConfigGet, ConfigList, ConfigSet, ConfigUnset,
    DocumentCommand, OutputMode, ParsedCli,
};
pub use parser::parse;
pub use preflight::output_context;
