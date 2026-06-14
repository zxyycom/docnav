mod adapter;
mod boundary;
mod command;
mod constants;
mod direct;
mod error;
mod invoke;
mod output;

pub use adapter::{Adapter, AdapterResult};
pub use command::{run_command, SdkCommand};
pub use direct::{
    run_direct_cli, DirectCliConfig, DirectOutputMode, NativeOptionDefault, NativeOptionSpec,
    NativeOptionValueSpec,
};
pub use error::{exit_code_for_error, AdapterError, AdapterExitCode, AdapterExitCodeError};
pub use invoke::{execute_operation, invoke_once};
pub use output::emit_diagnostic;

#[cfg(test)]
mod tests;
