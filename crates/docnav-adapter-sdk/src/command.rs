use std::io::{Read, Write};

use crate::adapter::{validated_manifest, validated_probe};
use crate::output::{write_adapter_boundary_error, write_manifest_json, write_probe_json};
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
    mut stderr: E,
) -> i32
where
    A: Adapter,
    R: Read,
    W: Write,
    E: Write,
{
    match command {
        SdkCommand::Manifest => match validated_manifest(adapter) {
            Ok(manifest) => write_manifest_json(manifest, stdout, stderr),
            Err(error) => write_adapter_boundary_error(&error, &mut stderr),
        },
        SdkCommand::Probe { path } => {
            let manifest = match validated_manifest(adapter) {
                Ok(manifest) => manifest,
                Err(error) => return write_adapter_boundary_error(&error, &mut stderr),
            };
            match validated_probe(adapter, &manifest, &path) {
                Ok(probe) => write_probe_json(probe, stdout, stderr),
                Err(error) => write_adapter_boundary_error(&error, &mut stderr),
            }
        }
        SdkCommand::Invoke => invoke_once(adapter, stdin, stdout, stderr),
    }
}
