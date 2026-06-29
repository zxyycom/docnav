use std::io::{Read, Write};

use docnav_adapter_sdk::{
    run_direct_cli, DirectCliConfig, DirectCliInvocation, NativeOptionDefault, NativeOptionSpec,
    NativeOptionValueSpec,
};
use docnav_protocol::Operation;

use crate::adapter::{
    MarkdownAdapter, DEFAULT_LIMIT, DEFAULT_MAX_HEADING_LEVEL, MAX_HEADING_LEVEL_OPTION,
};

const REQUEST_ID: &str = "docnav-markdown-cli";
const USAGE: &str = "usage: docnav-markdown <outline|read|find|info|manifest|probe|invoke> ...";
const MAX_HEADING_LEVEL_OPERATIONS: &[Operation] = &[Operation::Outline, Operation::Find];
const NATIVE_OPTIONS: &[NativeOptionSpec] = &[NativeOptionSpec {
    flag: "--max-heading-level",
    option_key: MAX_HEADING_LEVEL_OPTION,
    operations: MAX_HEADING_LEVEL_OPERATIONS,
    value: NativeOptionValueSpec::IntegerRange { min: 1, max: 6 },
    default: Some(NativeOptionDefault::Integer(
        DEFAULT_MAX_HEADING_LEVEL as u64,
    )),
}];

pub fn run<I, R, W, E>(args: I, stdin: R, stdout: W, stderr: E) -> i32
where
    I: IntoIterator<Item = String>,
    R: Read,
    W: Write,
    E: Write,
{
    let adapter = MarkdownAdapter;
    run_direct_cli(DirectCliInvocation {
        adapter: &adapter,
        args,
        stdin,
        stdout,
        stderr,
        config: DirectCliConfig {
            adapter_id: "docnav-markdown",
            program_name: "docnav-markdown",
            usage: USAGE,
            request_id: REQUEST_ID,
            default_limit: DEFAULT_LIMIT,
            default_user_config_dir: None,
            native_options: NATIVE_OPTIONS,
        },
    })
}

#[cfg(test)]
mod tests;
