use std::io::{Read, Write};

use crate::output::{write_manifest_json, write_probe_json};
use crate::{invoke_once, Adapter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SdkCommand {
    Manifest,
    Probe { path: String },
    Invoke,
}

pub fn run_command<A, R, W, E>(
    adapter: &A,
    command: SdkCommand,
    stdin: R,
    stdout: W,
    stderr: E,
) -> i32
where
    A: Adapter,
    R: Read,
    W: Write,
    E: Write,
{
    match command {
        SdkCommand::Manifest => write_manifest_json(adapter.manifest(), stdout, stderr),
        SdkCommand::Probe { path } => write_probe_json(adapter.probe(&path), stdout, stderr),
        SdkCommand::Invoke => invoke_once(adapter, stdin, stdout, stderr),
    }
}
