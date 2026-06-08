mod flags;
mod parser;
mod preflight;
mod types;
mod warning;

pub use parser::parse;
pub use preflight::output_context;
pub use types::{
    CliCommand, ConfigCommand, ConfigGet, ConfigList, ConfigSet, ConfigUnset, DocumentCommand,
    OutputMode, ParsedCli,
};
pub use warning::CliWarning;
