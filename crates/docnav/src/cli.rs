mod flags;
mod parser;
mod types;
mod warning;

pub use parser::parse;
pub use types::{
    CliCommand, ConfigCommand, ConfigGet, ConfigList, ConfigSet, ConfigUnset, DocumentCommand,
    OutputMode, ParsedCli,
};
pub use warning::CliWarning;
