mod adapter;
mod command;
mod constants;
mod direct_args;
mod direct_cli;
mod direct_output;
mod error;
mod invoke;
mod output;

pub use adapter::{Adapter, AdapterResult};
pub use command::{run_command, SdkCommand};
pub use direct_args::{NativeOptionDefault, NativeOptionSpec, NativeOptionValueSpec};
pub use direct_cli::{run_direct_cli, DirectCliConfig};
pub use direct_output::{DirectOutputMode, DirectTextFormatter};
pub use error::{exit_code_for_error, AdapterError, AdapterExitCode, AdapterExitCodeError};
pub use invoke::{execute_operation, invoke_once};
pub use output::emit_diagnostic;

#[cfg(test)]
mod tests;
