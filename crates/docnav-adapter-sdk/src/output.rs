use docnav_diagnostics::{
    typed_codes, BoundaryDetails, BoundaryDiagnosticCode, DiagnosticRecordDraft, DiagnosticSource,
    DiagnosticStack,
};
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
        let _ = emit_boundary_diagnostic(
            &mut stderr,
            BoundaryDiagnosticCode::ManifestSchemaValidationFailed,
            format!(
                "{}: {error}",
                diagnostics::MANIFEST_SCHEMA_VALIDATION_FAILED
            ),
        );
        return AdapterExitCode::ProtocolError.code();
    }
    if let Err(error) = manifest.validate_semantics() {
        let _ = emit_boundary_diagnostic(
            &mut stderr,
            BoundaryDiagnosticCode::ManifestSemanticValidationFailed,
            format!(
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
        let _ = emit_boundary_diagnostic(
            &mut stderr,
            BoundaryDiagnosticCode::ProbeResultSchemaValidationFailed,
            format!(
                "{}: {error}",
                diagnostics::PROBE_RESULT_SCHEMA_VALIDATION_FAILED
            ),
        );
        return AdapterExitCode::ProtocolError.code();
    }
    if let Err(error) = probe.validate_semantics() {
        let _ = emit_boundary_diagnostic(
            &mut stderr,
            BoundaryDiagnosticCode::ProbeResultSemanticValidationFailed,
            format!(
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
            let _ = emit_boundary_diagnostic(
                &mut *stderr,
                BoundaryDiagnosticCode::FailedToSerialize,
                format!("{} {label}: {error}", diagnostics::FAILED_TO_SERIALIZE),
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
            let _ = emit_boundary_diagnostic(
                &mut stderr,
                BoundaryDiagnosticCode::FailedToWriteJson,
                format!("{}: {error}", diagnostics::FAILED_TO_WRITE_JSON),
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
        let _ = emit_boundary_diagnostic(
            stderr,
            BoundaryDiagnosticCode::ProtocolResponseSemanticValidationFailed,
            format!(
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
        let _ = emit_boundary_diagnostic(
            stderr,
            BoundaryDiagnosticCode::ProtocolResponseSchemaValidationFailed,
            format!(
                "{}: {error}",
                diagnostics::PROTOCOL_RESPONSE_SCHEMA_VALIDATION_FAILED
            ),
        );
        return AdapterExitCode::ProtocolError.code();
    }
    if let Err(error) = serde_json::to_writer(stdout, &value) {
        let _ = emit_boundary_diagnostic(
            stderr,
            BoundaryDiagnosticCode::FailedToWriteProtocolResponse,
            format!(
                "{}: {error}",
                diagnostics::FAILED_TO_WRITE_PROTOCOL_RESPONSE
            ),
        );
        return AdapterExitCode::IoError.code();
    }
    exit_code.code()
}

fn write_record_summary_to_stderr<W: Write>(stderr: &mut W, message: &str) -> std::io::Result<()> {
    writeln!(stderr, "{message}")
}

pub(crate) fn emit_boundary_diagnostic<W: Write>(
    stderr: &mut W,
    code: BoundaryDiagnosticCode,
    message: impl Into<String>,
) -> std::io::Result<()> {
    let message = message.into();
    let mut diagnostics = DiagnosticStack::new();
    let id = diagnostics
        .push(boundary_diagnostic_record_draft(
            code,
            message.clone(),
            DiagnosticSource::with_stage("docnav-adapter-sdk", "stderr"),
        ))
        .map_err(std::io::Error::other)?;
    let Some(record) = diagnostics.get(id) else {
        return Err(std::io::Error::other(
            "pushed adapter boundary diagnostic record not found",
        ));
    };
    write_record_summary_to_stderr(stderr, record.summary())
}

fn boundary_diagnostic_record_draft(
    code: BoundaryDiagnosticCode,
    message: String,
    source: DiagnosticSource,
) -> DiagnosticRecordDraft {
    let details = BoundaryDetails::new(message.clone());
    match code {
        BoundaryDiagnosticCode::AdapterErrorExitCodeCannotBe => DiagnosticRecordDraft::new::<
            typed_codes::boundary::AdapterErrorExitCodeCannotBe,
        >(message, details, source),
        BoundaryDiagnosticCode::FailedToReadRequest => DiagnosticRecordDraft::new::<
            typed_codes::boundary::FailedToReadRequest,
        >(message, details, source),
        BoundaryDiagnosticCode::FailedToSerialize => DiagnosticRecordDraft::new::<
            typed_codes::boundary::FailedToSerialize,
        >(message, details, source),
        BoundaryDiagnosticCode::FailedToWriteCliWarning => DiagnosticRecordDraft::new::<
            typed_codes::boundary::FailedToWriteCliWarning,
        >(message, details, source),
        BoundaryDiagnosticCode::FailedToWriteJson => DiagnosticRecordDraft::new::<
            typed_codes::boundary::FailedToWriteJson,
        >(message, details, source),
        BoundaryDiagnosticCode::FailedToWriteProtocolResponse => DiagnosticRecordDraft::new::<
            typed_codes::boundary::FailedToWriteProtocolResponse,
        >(message, details, source),
        BoundaryDiagnosticCode::FailedToWriteReadableView => DiagnosticRecordDraft::new::<
            typed_codes::boundary::FailedToWriteReadableView,
        >(message, details, source),
        BoundaryDiagnosticCode::InvalidRequestJson => DiagnosticRecordDraft::new::<
            typed_codes::boundary::InvalidRequestJson,
        >(message, details, source),
        BoundaryDiagnosticCode::ManifestAdapterIdMismatch => DiagnosticRecordDraft::new::<
            typed_codes::boundary::ManifestAdapterIdMismatch,
        >(message, details, source),
        BoundaryDiagnosticCode::ManifestSchemaValidationFailed => {
            DiagnosticRecordDraft::new::<typed_codes::boundary::ManifestSchemaValidationFailed>(
                message, details, source,
            )
        }
        BoundaryDiagnosticCode::ManifestSemanticValidationFailed => {
            DiagnosticRecordDraft::new::<typed_codes::boundary::ManifestSemanticValidationFailed>(
                message, details, source,
            )
        }
        BoundaryDiagnosticCode::ProbeResultAdapterIdMismatch => DiagnosticRecordDraft::new::<
            typed_codes::boundary::ProbeResultAdapterIdMismatch,
        >(message, details, source),
        BoundaryDiagnosticCode::ProbeResultSchemaValidationFailed => {
            DiagnosticRecordDraft::new::<typed_codes::boundary::ProbeResultSchemaValidationFailed>(
                message, details, source,
            )
        }
        BoundaryDiagnosticCode::ProbeResultSemanticValidationFailed => {
            DiagnosticRecordDraft::new::<typed_codes::boundary::ProbeResultSemanticValidationFailed>(
                message, details, source,
            )
        }
        BoundaryDiagnosticCode::ProtocolResponseSchemaValidationFailed => {
            DiagnosticRecordDraft::new::<
                typed_codes::boundary::ProtocolResponseSchemaValidationFailed,
            >(message, details, source)
        }
        BoundaryDiagnosticCode::ProtocolResponseSemanticValidationFailed => {
            DiagnosticRecordDraft::new::<
                typed_codes::boundary::ProtocolResponseSemanticValidationFailed,
            >(message, details, source)
        }
        BoundaryDiagnosticCode::ReadableViewRenderFailed => DiagnosticRecordDraft::new::<
            typed_codes::boundary::ReadableViewRenderFailed,
        >(message, details, source),
        BoundaryDiagnosticCode::RequestDeserializationFailed => DiagnosticRecordDraft::new::<
            typed_codes::boundary::RequestDeserializationFailed,
        >(message, details, source),
        BoundaryDiagnosticCode::RequestSchemaValidationFailed => DiagnosticRecordDraft::new::<
            typed_codes::boundary::RequestSchemaValidationFailed,
        >(message, details, source),
    }
}

pub(crate) fn write_adapter_boundary_error<E: Write>(
    error: &AdapterBoundaryError,
    stderr: &mut E,
) -> i32 {
    let _ = emit_boundary_diagnostic(
        stderr,
        error.diagnostic_code(),
        format!("{}: {error}", error.diagnostic()),
    );
    AdapterExitCode::ProtocolError.code()
}
