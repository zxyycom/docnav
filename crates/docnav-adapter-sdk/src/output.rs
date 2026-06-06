use docnav_protocol::{
    validate_manifest_value, validate_probe_result_value, validate_protocol_response_value,
    Manifest, ProbeResult, ProtocolResponse,
};
use serde::Serialize;
use serde_json::Value;
use std::io::Write;

use crate::boundary::AdapterBoundaryError;
use crate::constants::{diagnostics, json_labels};
use crate::AdapterExitCode;

pub(crate) fn write_manifest_json<W, E>(manifest: Manifest, stdout: W, mut stderr: E) -> i32
where
    W: Write,
    E: Write,
{
    let value = match to_json_value(&manifest, &mut stderr, json_labels::MANIFEST) {
        Ok(value) => value,
        Err(exit_code) => return exit_code.code(),
    };
    if let Err(error) = validate_manifest_value(&value) {
        let _ = emit_diagnostic(
            &mut stderr,
            &format!(
                "{}: {error}",
                diagnostics::MANIFEST_SCHEMA_VALIDATION_FAILED
            ),
        );
        return AdapterExitCode::ProtocolError.code();
    }
    if let Err(error) = manifest.validate_semantics() {
        let _ = emit_diagnostic(
            &mut stderr,
            &format!(
                "{}: {error}",
                diagnostics::MANIFEST_SEMANTIC_VALIDATION_FAILED
            ),
        );
        return AdapterExitCode::ProtocolError.code();
    }
    write_json_value(&value, stdout, stderr)
}

pub(crate) fn write_probe_json<W, E>(probe: ProbeResult, stdout: W, mut stderr: E) -> i32
where
    W: Write,
    E: Write,
{
    let value = match to_json_value(&probe, &mut stderr, json_labels::PROBE_RESULT) {
        Ok(value) => value,
        Err(exit_code) => return exit_code.code(),
    };
    if let Err(error) = validate_probe_result_value(&value) {
        let _ = emit_diagnostic(
            &mut stderr,
            &format!(
                "{}: {error}",
                diagnostics::PROBE_RESULT_SCHEMA_VALIDATION_FAILED
            ),
        );
        return AdapterExitCode::ProtocolError.code();
    }
    if let Err(error) = probe.validate_semantics() {
        let _ = emit_diagnostic(
            &mut stderr,
            &format!(
                "{}: {error}",
                diagnostics::PROBE_RESULT_SEMANTIC_VALIDATION_FAILED
            ),
        );
        return AdapterExitCode::ProtocolError.code();
    }
    write_json_value(&value, stdout, stderr)
}

fn to_json_value<T, E>(value: &T, stderr: &mut E, label: &str) -> Result<Value, AdapterExitCode>
where
    T: Serialize,
    E: Write,
{
    match serde_json::to_value(value) {
        Ok(value) => Ok(value),
        Err(error) => {
            let _ = emit_diagnostic(
                &mut *stderr,
                &format!("{} {label}: {error}", diagnostics::FAILED_TO_SERIALIZE),
            );
            Err(AdapterExitCode::IoError)
        }
    }
}

fn write_json_value<W, E>(value: &Value, mut stdout: W, mut stderr: E) -> i32
where
    W: Write,
    E: Write,
{
    match serde_json::to_writer(&mut stdout, value) {
        Ok(()) => AdapterExitCode::Success.code(),
        Err(error) => {
            let _ = emit_diagnostic(
                &mut stderr,
                &format!("{}: {error}", diagnostics::FAILED_TO_WRITE_JSON),
            );
            AdapterExitCode::IoError.code()
        }
    }
}

pub(crate) fn write_protocol_response<W, E>(
    response: &ProtocolResponse,
    stdout: &mut W,
    stderr: &mut E,
    exit_code: AdapterExitCode,
) -> i32
where
    W: Write,
    E: Write,
{
    if let Err(error) = response.validate() {
        let _ = emit_diagnostic(
            stderr,
            &format!(
                "{}: {error}",
                diagnostics::PROTOCOL_RESPONSE_SEMANTIC_VALIDATION_FAILED
            ),
        );
        return AdapterExitCode::ProtocolError.code();
    }
    let value = match to_json_value(response, stderr, json_labels::PROTOCOL_RESPONSE) {
        Ok(value) => value,
        Err(exit_code) => return exit_code.code(),
    };
    if let Err(error) = validate_protocol_response_value(&value) {
        let _ = emit_diagnostic(
            stderr,
            &format!(
                "{}: {error}",
                diagnostics::PROTOCOL_RESPONSE_SCHEMA_VALIDATION_FAILED
            ),
        );
        return AdapterExitCode::ProtocolError.code();
    }
    if let Err(error) = serde_json::to_writer(stdout, &value) {
        let _ = emit_diagnostic(
            stderr,
            &format!(
                "{}: {error}",
                diagnostics::FAILED_TO_WRITE_PROTOCOL_RESPONSE
            ),
        );
        return AdapterExitCode::IoError.code();
    }
    exit_code.code()
}

pub fn emit_diagnostic<W: Write>(stderr: &mut W, message: &str) -> std::io::Result<()> {
    writeln!(stderr, "{message}")
}

pub(crate) fn write_adapter_boundary_error<E: Write>(
    error: &AdapterBoundaryError,
    stderr: &mut E,
) -> i32 {
    let _ = emit_diagnostic(stderr, &format!("{}: {error}", error.diagnostic()));
    AdapterExitCode::ProtocolError.code()
}
